//! Agent mode orchestrator for autonomous desktop control.
//!
//! Runs an agent loop: screenshot → send to Ollama vision model →
//! parse actions → execute → repeat, until the model signals DONE
//! or the iteration limit is reached.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio_util::sync::CancellationToken;

use crate::computer_control::{self, AgentAction};

// ─── Status types ───────────────────────────────────────────────────────────────

/// Current status of the agent loop.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Capturing,
    Analyzing,
    Executing,
    Done,
    Error,
}

/// An event emitted by the agent loop to the frontend.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum AgentEvent {
    StatusChanged(AgentStatus),
    ActionExecuted { action: String, result: String },
    Reasoning(String),
    ScreenshotTaken(String),
    Error(String),
    Done { summary: String },
}

// ─── Agent state ────────────────────────────────────────────────────────────────

/// Shared state for the running agent.
pub struct AgentState {
    cancel: Mutex<CancellationToken>,
    status: Mutex<AgentStatus>,
    history: Mutex<Vec<String>>,
}

impl AgentState {
    pub fn new() -> Self {
        Self {
            cancel: Mutex::new(CancellationToken::new()),
            status: Mutex::new(AgentStatus::Idle),
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn get_status(&self) -> AgentStatus {
        self.status.lock().unwrap().clone()
    }

    fn set_status(&self, status: AgentStatus) {
        *self.status.lock().unwrap() = status;
    }

    fn add_to_history(&self, entry: String) {
        self.history.lock().unwrap().push(entry);
    }

    pub fn cancel(&self) {
        self.cancel.lock().unwrap().cancel();
    }

    fn is_cancelled(&self) -> bool {
        self.cancel.lock().unwrap().is_cancelled()
    }

    fn reset(&self) {
        *self.cancel.lock().unwrap() = CancellationToken::new();
        *self.status.lock().unwrap() = AgentStatus::Idle;
        self.history.lock().unwrap().clear();
    }
}

// ─── System prompt ──────────────────────────────────────────────────────────────

const AGENT_SYSTEM_PROMPT: &str = r#"You are a desktop automation agent. You control a Windows computer by analyzing screenshots and issuing actions.

On each turn you will receive a screenshot. Analyze it and decide what action to take to accomplish the user's task.

Available actions (one per line, must be EXACTLY formatted):
- CLICK x y — Left-click at screen coordinates (x, y)
- DOUBLE_CLICK x y — Double-click at coordinates
- RIGHT_CLICK x y — Right-click at coordinates
- DRAG start_x start_y end_x end_y [duration_ms] — Drag from start to end
- TYPE text — Type the given text character by character
- KEY_PRESS modifiers+key — Press key combo, e.g. "ctrl+c", "ctrl+shift+s"
- SCROLL direction amount — Scroll "up" or "down" by amount (default unit is 3 lines)
- LAUNCH target — Open a program, file, or URL
- SCREENSHOT — Take another screenshot (for checking progress)
- DONE summary — Task is complete, summarize what was accomplished

Important rules:
1. Always provide exactly ONE action per response line starting with the action keyword.
2. You may include brief reasoning before the action line.
3. Coordinates are in absolute screen pixels.
4. Use SCREENSHOT when you need to verify a change before proceeding.
5. Use DONE when the task is complete.
6. Be precise with coordinates — examine the screenshot carefully."#;

// ─── Iteration limits ───────────────────────────────────────────────────────────

const MAX_ITERATIONS: u32 = 50;
const ACTION_DELAY_MS: u64 = 300;
const SCREENSHOT_DELAY_MS: u64 = 500;

// ─── Agent loop ──────────────────────────────────────────────────────────────────

/// Runs the agent loop: screenshot → model → parse → execute → repeat.
pub async fn run_agent_loop(
    app_handle: tauri::AppHandle,
    state: Arc<AgentState>,
    task: String,
    model: String,
    ollama_url: String,
) -> Result<(), String> {
    state.reset();
    state.set_status(AgentStatus::Capturing);
    let _ = app_handle.emit("thuki://agent", AgentEvent::StatusChanged(AgentStatus::Capturing));

    // Build initial prompt with the task description.
    let mut conversation: Vec<serde_json::Value> = vec![
        serde_json::json!({
            "role": "system",
            "content": AGENT_SYSTEM_PROMPT,
        }),
        serde_json::json!({
            "role": "user",
            "content": format!("Task: {}", task),
        }),
    ];

    for iteration in 0..MAX_ITERATIONS {
        if state.is_cancelled() {
            state.set_status(AgentStatus::Done);
            let _ = app_handle.emit("thuki://agent", AgentEvent::StatusChanged(AgentStatus::Done));
            return Ok(());
        }

        // Step 1: Take a screenshot.
        state.set_status(AgentStatus::Capturing);
        let _ = app_handle.emit("thuki://agent", AgentEvent::StatusChanged(AgentStatus::Capturing));

        let screenshot_path = capture_screenshot(&app_handle).await?;

        let _ = app_handle.emit("thuki://agent", AgentEvent::ScreenshotTaken(screenshot_path.clone()));

        // Read screenshot as base64 for the vision model.
        let screenshot_b64 = tokio::task::spawn_blocking({
            let path = screenshot_path.clone();
            move || read_image_as_base64(&path)
        })
        .await
        .map_err(|e| format!("Failed to read screenshot: {e}"))??;

        // Add screenshot to conversation.
        conversation.push(serde_json::json!({
            "role": "user",
            "content": [
                {
                    "type": "text",
                    "text": if iteration == 0 {
                        format!("Here is the current screen. Task: {}", task)
                    } else {
                        "Here is the screen after the last action.".to_string()
                    },
                },
                {
                    "type": "image_url",
                    "image_url": {
                        "url": format!("data:image/png;base64,{}", screenshot_b64),
                    },
                },
            ],
        }));

        // Step 2: Send to Ollama vision model.
        state.set_status(AgentStatus::Analyzing);
        let _ = app_handle.emit("thuki://agent", AgentEvent::StatusChanged(AgentStatus::Analyzing));

        let response = query_ollama(&ollama_url, &model, &conversation).await?;

        // Add model response to conversation history.
        conversation.push(serde_json::json!({
            "role": "assistant",
            "content": response,
        }));

        state.add_to_history(format!("[Model] {}", response));
        let _ = app_handle.emit("thuki://agent", AgentEvent::Reasoning(response.clone()));

        // Step 3: Parse and execute actions from the response.
        state.set_status(AgentStatus::Executing);
        let _ = app_handle.emit("thuki://agent", AgentEvent::StatusChanged(AgentStatus::Executing));

        for line in response.lines() {
            if state.is_cancelled() {
                state.set_status(AgentStatus::Done);
                let _ = app_handle.emit("thuki://agent", AgentEvent::StatusChanged(AgentStatus::Done));
                return Ok(());
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Skip lines that don't start with a known action keyword.
            let upper = line.to_uppercase();
            let is_action = upper.starts_with("CLICK")
                || upper.starts_with("DOUBLE_CLICK")
                || upper.starts_with("RIGHT_CLICK")
                || upper.starts_with("DRAG")
                || upper.starts_with("TYPE")
                || upper.starts_with("KEY_PRESS")
                || upper.starts_with("SCROLL")
                || upper.starts_with("LAUNCH")
                || upper.starts_with("DONE")
                || upper.starts_with("SCREENSHOT");

            if !is_action {
                continue;
            }

            let action = match computer_control::parse_action_line(line) {
                Some(a) => a,
                None => continue,
            };

            // Handle DONE action.
            if let AgentAction::Done { ref summary } = action {
                let _ = app_handle.emit("thuki://agent", AgentEvent::Done { summary: summary.clone() });
                state.set_status(AgentStatus::Done);
                let _ = app_handle.emit("thuki://agent", AgentEvent::StatusChanged(AgentStatus::Done));
                return Ok(());
            }

            // Handle SCREENSHOT action (just retake, don't execute anything).
            if let AgentAction::Screenshot {} = action {
                // Remove the last screenshot+prompt from conversation to replace it.
                if conversation.len() >= 2 {
                    conversation.pop();
                    conversation.pop();
                }
                break; // Restart loop to take a new screenshot.
            }

            // Execute the action.
            let action_desc = format!("{:?}", action);
            let result = tokio::task::spawn_blocking({
                let action = action.clone();
                move || computer_control::execute_action(&action)
            })
            .await
            .map_err(|e| format!("Action execution panicked: {e}"))?;

            let result_msg = match result {
                Ok(()) => "ok".to_string(),
                Err(e) => e,
            };
            state.add_to_history(format!("[Action] {} -> {}", action_desc, result_msg));
            let _ = app_handle.emit("thuki://agent", AgentEvent::ActionExecuted {
                action: action_desc,
                result: result_msg,
            });

            // Small delay between actions for stability.
            tokio::time::sleep(Duration::from_millis(ACTION_DELAY_MS)).await;
        }

        // Delay before next screenshot to let the screen update.
        tokio::time::sleep(Duration::from_millis(SCREENSHOT_DELAY_MS)).await;
    }

    // Reached max iterations.
    let summary = "Agent reached maximum iteration limit".to_string();
    let _ = app_handle.emit("thuki://agent", AgentEvent::Done { summary: summary.clone() });
    state.set_status(AgentStatus::Done);
    let _ = app_handle.emit("thuki://agent", AgentEvent::StatusChanged(AgentStatus::Done));
    Ok(())
}

// ─── Screenshot capture ──────────────────────────────────────────────────────────

async fn capture_screenshot(app_handle: &tauri::AppHandle) -> Result<String, String> {
    crate::windows_screenshot::capture_silent_screenshot_command(app_handle.clone()).await
}

// ─── Ollama query ────────────────────────────────────────────────────────────────

async fn query_ollama(
    ollama_url: &str,
    model: &str,
    conversation: &[serde_json::Value],
) -> Result<String, String> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": model,
        "messages": conversation,
        "stream": false,
    });

    let url = format!("{}/api/chat", ollama_url.trim_end_matches('/'));

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Ollama request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Ollama returned status {}: {}", status, body));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Ollama response: {e}"))?;

    json.get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "No content in Ollama response".to_string())
}

// ─── Image encoding ─────────────────────────────────────────────────────────────

fn read_image_as_base64(path: &str) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read screenshot: {e}"))?;
    Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes))
}

// ─── Tauri commands ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn start_agent_mode(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Arc<AgentState>>,
    task: String,
    model: String,
    ollama_url: String,
) -> Result<(), String> {
    let state = state.inner().clone();
    let app_handle = app_handle.clone();

    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_agent_loop(app_handle, state, task, model, ollama_url).await {
            eprintln!("thuki: [agent] error: {e}");
        }
    });

    Ok(())
}

#[tauri::command]
pub fn stop_agent_mode(state: tauri::State<'_, Arc<AgentState>>) -> Result<(), String> {
    state.cancel();
    Ok(())
}

#[tauri::command]
pub fn get_agent_status(state: tauri::State<'_, Arc<AgentState>>) -> Result<AgentStatus, String> {
    Ok(state.get_status())
}

// ─── Tests ───────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_status_serialization() {
        let status = AgentStatus::Capturing;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"capturing\"");
    }

    #[test]
    fn agent_status_roundtrip() {
        let statuses = vec![
            AgentStatus::Idle,
            AgentStatus::Capturing,
            AgentStatus::Analyzing,
            AgentStatus::Executing,
            AgentStatus::Done,
            AgentStatus::Error,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let deserialized: AgentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, deserialized);
        }
    }

    #[test]
    fn agent_event_serialization() {
        let event = AgentEvent::StatusChanged(AgentStatus::Executing);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("status_changed"));

        let event = AgentEvent::ActionExecuted {
            action: "Click".to_string(),
            result: "ok".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("action_executed"));

        let event = AgentEvent::Done { summary: "done".to_string() };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("done"));
    }

    #[test]
    fn agent_state_new_is_idle() {
        let state = AgentState::new();
        assert_eq!(state.get_status(), AgentStatus::Idle);
    }

    #[test]
    fn agent_state_set_status() {
        let state = AgentState::new();
        state.set_status(AgentStatus::Executing);
        assert_eq!(state.get_status(), AgentStatus::Executing);
    }

    #[test]
    fn agent_state_cancel() {
        let state = AgentState::new();
        assert!(!state.is_cancelled());
        state.cancel();
        assert!(state.is_cancelled());
    }

    #[test]
    fn agent_state_reset() {
        let state = AgentState::new();
        state.set_status(AgentStatus::Executing);
        state.add_to_history("test".to_string());
        state.cancel();
        state.reset();
        assert_eq!(state.get_status(), AgentStatus::Idle);
        assert!(!state.is_cancelled());
        assert!(state.history.lock().unwrap().is_empty());
    }

    #[test]
    fn agent_state_history() {
        let state = AgentState::new();
        state.add_to_history("entry1".to_string());
        state.add_to_history("entry2".to_string());
        let h = state.history.lock().unwrap();
        assert_eq!(h.len(), 2);
        assert_eq!(h[0], "entry1");
        assert_eq!(h[1], "entry2");
    }

    #[test]
    fn max_iterations_constant() {
        assert_eq!(MAX_ITERATIONS, 50);
    }

    #[test]
    fn action_delay_constant() {
        assert_eq!(ACTION_DELAY_MS, 300);
    }

    #[test]
    fn screenshot_delay_constant() {
        assert_eq!(SCREENSHOT_DELAY_MS, 500);
    }

    #[test]
    fn parse_action_integration() {
        let action = computer_control::parse_action_line("CLICK 100 200");
        assert!(matches!(action, Some(AgentAction::Click { x: 100, y: 200 })));

        let action = computer_control::parse_action_line("DONE task complete");
        assert!(matches!(action, Some(AgentAction::Done { ref summary }) if summary == "task complete"));

        let action = computer_control::parse_action_line("SCREENSHOT");
        assert!(matches!(action, Some(AgentAction::Screenshot {})));
    }

    #[test]
    fn system_prompt_not_empty() {
        assert!(!AGENT_SYSTEM_PROMPT.is_empty());
        assert!(AGENT_SYSTEM_PROMPT.contains("CLICK"));
        assert!(AGENT_SYSTEM_PROMPT.contains("DONE"));
    }

    #[test]
    fn read_image_as_base64_file_not_found() {
        let result = read_image_as_base64("/nonexistent/path/to/image.png");
        assert!(result.is_err());
    }
}