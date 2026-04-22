//! OpenAI API provider for streaming chat completions with tool use.
//!
//! Implements the OpenAI Chat Completions API with SSE streaming,
//! function calling (tool use), and converts responses to the
//! unified ProviderChunk format.

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use super::{ProviderChunk, ToolCall};

// ─── Request types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<OpenAITool>,
    stream: bool,
}

#[derive(Serialize, Clone)]
struct OpenAIMessage {
    role: String,
    content: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Serialize, Clone)]
struct OpenAIToolCall {
    id: String,
    r#type: String,
    function: OpenAIFunctionCall,
}

#[derive(Serialize, Clone)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Serialize)]
pub struct OpenAITool {
    r#type: String,
    function: OpenAIFunction,
}

#[derive(Serialize)]
pub struct OpenAIFunction {
    name: String,
    description: String,
    parameters: Value,
}

// ─── Response types (non-streaming, kept for future use) ─────────────────────

#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenAIChoice {
    message: Option<OpenAIResponseMessage>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenAIResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIResponseToolCall>>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenAIResponseToolCall {
    id: String,
    r#type: Option<String>,
    function: OpenAIResponseFunction,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenAIResponseFunction {
    name: String,
    arguments: String,
}

// ─── SSE streaming types ───────────────────────────────────────────────────────

#[derive(Deserialize)]
struct OpenAIStreamChunk {
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAIStreamDelta {
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIStreamToolCall>>,
}

#[derive(Deserialize)]
struct OpenAIStreamToolCall {
    index: Option<u32>,
    id: Option<String>,
    #[allow(dead_code)]
    r#type: Option<String>,
    function: Option<OpenAIStreamFunction>,
}

#[derive(Deserialize)]
struct OpenAIStreamFunction {
    name: Option<String>,
    arguments: Option<String>,
}

// ─── Computer use tool definitions ────────────────────────────────────────────

/// Returns the OpenAI-format tool definitions for computer control.
pub fn openai_computer_tools() -> Vec<OpenAITool> {
    vec![
        OpenAITool {
            r#type: "function".to_string(),
            function: OpenAIFunction {
                name: "computer_click".to_string(),
                description: "Left-click at the specified screen coordinates.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "x": { "type": "integer", "description": "X coordinate in pixels" },
                        "y": { "type": "integer", "description": "Y coordinate in pixels" },
                    },
                    "required": ["x", "y"],
                }),
            },
        },
        OpenAITool {
            r#type: "function".to_string(),
            function: OpenAIFunction {
                name: "computer_double_click".to_string(),
                description: "Double-click at the specified screen coordinates.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "x": { "type": "integer", "description": "X coordinate" },
                        "y": { "type": "integer", "description": "Y coordinate" },
                    },
                    "required": ["x", "y"],
                }),
            },
        },
        OpenAITool {
            r#type: "function".to_string(),
            function: OpenAIFunction {
                name: "computer_right_click".to_string(),
                description: "Right-click at the specified screen coordinates.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "x": { "type": "integer" },
                        "y": { "type": "integer" },
                    },
                    "required": ["x", "y"],
                }),
            },
        },
        OpenAITool {
            r#type: "function".to_string(),
            function: OpenAIFunction {
                name: "computer_type".to_string(),
                description: "Type text character by character.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "The text to type" },
                    },
                    "required": ["text"],
                }),
            },
        },
        OpenAITool {
            r#type: "function".to_string(),
            function: OpenAIFunction {
                name: "computer_key_press".to_string(),
                description: "Press a key combination (e.g., ctrl+c, alt+tab).".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "modifiers": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Modifier keys: ctrl, alt, shift",
                        },
                        "key": { "type": "string", "description": "The key to press" },
                    },
                    "required": ["key"],
                }),
            },
        },
        OpenAITool {
            r#type: "function".to_string(),
            function: OpenAIFunction {
                name: "computer_scroll".to_string(),
                description: "Scroll the screen up or down.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "direction": { "type": "string", "enum": ["up", "down"] },
                        "amount": { "type": "integer", "description": "Number of scroll units (default 3)" },
                    },
                    "required": ["direction"],
                }),
            },
        },
        OpenAITool {
            r#type: "function".to_string(),
            function: OpenAIFunction {
                name: "computer_launch".to_string(),
                description: "Open a program, file, or URL.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Program name, file path, or URL to open" },
                    },
                    "required": ["target"],
                }),
            },
        },
        OpenAITool {
            r#type: "function".to_string(),
            function: OpenAIFunction {
                name: "computer_screenshot".to_string(),
                description: "Take a screenshot to check the current screen state.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {},
                }),
            },
        },
    ]
}

// ─── Streaming chat ────────────────────────────────────────────────────────────

/// Streams a chat completion from the OpenAI API.
///
/// Sends the conversation with tool definitions, streams the response,
/// and emits `ProviderChunk` variants via the `on_chunk` callback.
/// If the model makes tool calls, they are collected and emitted as a
/// single `ProviderChunk::ToolCalls` chunk.
pub async fn stream_openai_chat(
    base_url: &str,
    model: &str,
    api_key: &str,
    messages: Vec<crate::commands::ChatMessage>,
    include_tools: bool,
    client: &reqwest::Client,
    cancel_token: CancellationToken,
    mut on_chunk: impl FnMut(ProviderChunk),
) -> Result<String, String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    // Convert Ollama messages to OpenAI format.
    let openai_messages: Vec<OpenAIMessage> = messages
        .into_iter()
        .map(|m| {
            let content = if let Some(ref imgs) = m.images {
                // Multi-modal: text + images.
                let mut parts = vec![serde_json::json!({
                    "type": "text",
                    "text": m.content,
                })];
                for img in imgs {
                    parts.push(serde_json::json!({
                        "type": "image_url",
                        "image_url": { "url": format!("data:image/png;base64,{}", img) },
                    }));
                }
                Value::Array(parts)
            } else {
                Value::String(m.content)
            };
            OpenAIMessage {
                role: m.role,
                content,
                tool_calls: None,
                tool_call_id: None,
            }
        })
        .collect();

    let tools = if include_tools {
        openai_computer_tools()
    } else {
        vec![]
    };

    let request = OpenAIChatRequest {
        model: model.to_string(),
        messages: openai_messages,
        tools,
        stream: true,
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("OpenAI request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("OpenAI returned status {}: {}", status, body));
    }

    let mut stream = response.bytes_stream();
    let mut accumulated = String::new();
    let mut tool_calls_in_progress: Vec<OpenAIToolCall> = Vec::new();

    loop {
        tokio::select! {
            biased;
            _ = cancel_token.cancelled() => {
                on_chunk(ProviderChunk::Cancelled);
                return Ok(accumulated);
            }
            chunk_opt = stream.next() => {
                match chunk_opt {
                    Some(Ok(bytes)) => {
                        let text = String::from_utf8_lossy(&bytes);
                        for line in text.lines() {
                            let line = line.trim();
                            if !line.starts_with("data: ") {
                                continue;
                            }
                            let data = &line[6..];
                            if data == "[DONE]" {
                                // Emit any pending tool calls.
                                if !tool_calls_in_progress.is_empty() {
                                    let converted: Vec<ToolCall> = tool_calls_in_progress
                                        .into_iter()
                                        .map(|tc| ToolCall {
                                            id: tc.id,
                                            name: tc.function.name,
                                            arguments: tc.function.arguments,
                                        })
                                        .collect();
                                    on_chunk(ProviderChunk::ToolCalls(converted));
                                }
                                on_chunk(ProviderChunk::Done);
                                return Ok(accumulated);
                            }

                            let parsed: Result<OpenAIStreamChunk, _> = serde_json::from_str(data);
                            if let Ok(chunk) = parsed {
                                for choice in &chunk.choices {
                                    // Text content.
                                    if let Some(ref content) = choice.delta.content {
                                        if !content.is_empty() {
                                            accumulated.push_str(content);
                                            on_chunk(ProviderChunk::Token(content.clone()));
                                        }
                                    }

                                    // Tool calls — accumulate across stream chunks.
                                    if let Some(ref calls) = choice.delta.tool_calls {
                                        for call in calls {
                                            let idx = call.index.unwrap_or(0) as usize;
                                            // Extend vector if needed.
                                            while tool_calls_in_progress.len() <= idx {
                                                tool_calls_in_progress.push(OpenAIToolCall {
                                                    id: String::new(),
                                                    r#type: "function".to_string(),
                                                    function: OpenAIFunctionCall {
                                                        name: String::new(),
                                                        arguments: String::new(),
                                                    },
                                                });
                                            }
                                            let tc = &mut tool_calls_in_progress[idx];
                                            if let Some(ref id) = call.id {
                                                tc.id = id.clone();
                                            }
                                            if let Some(ref func) = call.function {
                                                if let Some(ref name) = func.name {
                                                    tc.function.name = name.clone();
                                                }
                                                if let Some(ref args) = func.arguments {
                                                    tc.function.arguments.push_str(args);
                                                }
                                            }
                                        }
                                    }

                                    // Check for finish.
                                    if let Some(ref reason) = choice.finish_reason {
                                        if reason == "tool_calls" || reason == "stop" {
                                            if !tool_calls_in_progress.is_empty() {
                                                let converted: Vec<ToolCall> = tool_calls_in_progress
                                                    .into_iter()
                                                    .map(|tc| ToolCall {
                                                        id: tc.id,
                                                        name: tc.function.name,
                                                        arguments: tc.function.arguments,
                                                    })
                                                    .collect();
                                                on_chunk(ProviderChunk::ToolCalls(converted));
                                                tool_calls_in_progress = Vec::new();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        on_chunk(ProviderChunk::Error(format!("Stream error: {e}")));
                        return Err(format!("OpenAI stream error: {e}"));
                    }
                    None => {
                        if !tool_calls_in_progress.is_empty() {
                            let converted: Vec<ToolCall> = tool_calls_in_progress
                                .into_iter()
                                .map(|tc| ToolCall {
                                    id: tc.id,
                                    name: tc.function.name,
                                    arguments: tc.function.arguments,
                                })
                                .collect();
                            on_chunk(ProviderChunk::ToolCalls(converted));
                        }
                        on_chunk(ProviderChunk::Done);
                        return Ok(accumulated);
                    }
                }
            }
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_tool_definitions_count() {
        let tools = openai_computer_tools();
        assert_eq!(tools.len(), 8); // click, double_click, right_click, type, key_press, scroll, launch, screenshot
    }

    #[test]
    fn openai_message_serialization() {
        let msg = OpenAIMessage {
            role: "user".to_string(),
            content: Value::String("Hello".to_string()),
            tool_calls: None,
            tool_call_id: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"Hello\""));
    }

    #[test]
    fn openai_request_serialization() {
        let request = OpenAIChatRequest {
            model: "gpt-4o".to_string(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: Value::String("Test".to_string()),
                tool_calls: None,
                tool_call_id: None,
            }],
            tools: vec![],
            stream: true,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"gpt-4o\""));
        assert!(json.contains("\"stream\":true"));
        // Empty tools array should be skipped.
        assert!(!json.contains("\"tools\""));
    }
}