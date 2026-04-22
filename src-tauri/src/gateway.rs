//! Local gateway server exposing an OpenAI-compatible API.
//!
//! Runs an HTTP server on a configurable port inside the Tauri process.
//! Proxies requests to the configured Ollama instance, translating between
//! OpenAI and Ollama request/response formats.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::{
    extract::State as AxumState,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

// ─── Shared state ─────────────────────────────────────────────────────────────

pub struct GatewayState {
    pub running: AtomicBool,
    pub port: std::sync::Mutex<u16>,
    pub ollama_url: std::sync::Mutex<String>,
    pub cancel: tokio::sync::Mutex<Option<CancellationToken>>,
}

impl GatewayState {
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            port: std::sync::Mutex::new(18789),
            ollama_url: std::sync::Mutex::new("http://127.0.0.1:11434".to_string()),
            cancel: tokio::sync::Mutex::new(None),
        }
    }
}

// ─── OpenAI-compatible types ──────────────────────────────────────────────────

#[derive(Serialize)]
struct ModelObject {
    id: String,
    object: &'static str,
    created: u64,
    owned_by: &'static str,
}

#[derive(Serialize)]
struct ModelsResponse {
    object: &'static str,
    data: Vec<ModelObject>,
}

#[derive(Deserialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessageOpenAI>,
    #[serde(default)]
    _stream: bool,
    #[serde(default)]
    _temperature: Option<f64>,
}

#[derive(Deserialize)]
struct ChatMessageOpenAI {
    role: String,
    content: Value,
}

#[derive(Serialize)]
struct ChatCompletionResponse {
    id: String,
    object: &'static str,
    created: u64,
    model: String,
    choices: Vec<Choice>,
}

#[derive(Serialize)]
struct Choice {
    index: u32,
    message: ChoiceMessage,
    finish_reason: &'static str,
}

#[derive(Serialize)]
struct ChoiceMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    message: String,
    r#type: String,
}

// ─── Route handlers ────────────────────────────────────────────────────────────

async fn health() -> &'static str {
    "ok"
}

async fn list_models(AxumState(state): AxumState<Arc<GatewayState>>) -> Json<ModelsResponse> {
    let ollama_url = state.ollama_url.lock().unwrap().clone();
    let models = fetch_ollama_models(&ollama_url).await.unwrap_or_default();

    let data = models
        .into_iter()
        .map(|name| ModelObject {
            id: name,
            object: "model",
            created: 0,
            owned_by: "ollama",
        })
        .collect();

    Json(ModelsResponse {
        object: "list",
        data,
    })
}

async fn chat_completions(
    AxumState(state): AxumState<Arc<GatewayState>>,
    Json(req): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, (StatusCode, Json<ErrorResponse>)> {
    let ollama_url = state.ollama_url.lock().unwrap().clone();

    // Convert OpenAI messages to Ollama format.
    let messages: Vec<Value> = req
        .messages
        .into_iter()
        .map(|m| {
            let content_str = match &m.content {
                Value::String(s) => s.clone(),
                Value::Array(arr) => {
                    // Multi-part content (text + image_url).
                    let mut text = String::new();
                    for part in arr {
                        if let Some(t) = part.get("text").and_then(|v| v.as_str()) {
                            text.push_str(t);
                        }
                    }
                    text
                }
                other => other.to_string(),
            };
            json!({
                "role": m.role,
                "content": content_str,
            })
        })
        .collect();

    let body = json!({
        "model": req.model,
        "messages": messages,
        "stream": false,
    });

    let client = reqwest::Client::new();
    let url = format!("{}/api/chat", ollama_url.trim_end_matches('/'));

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| gateway_error(&e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(gateway_error(&format!(
            "Ollama returned {}: {}",
            status, body
        )));
    }

    let json: Value = response
        .json()
        .await
        .map_err(|e| gateway_error(&e.to_string()))?;

    let content = json
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    let id = format!("chatcmpl-{}", uuid::Uuid::new_v4().to_string().replace('-', "")[..8].to_string());

    Ok(Json(ChatCompletionResponse {
        id,
        object: "chat.completion",
        created: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        model: req.model,
        choices: vec![Choice {
            index: 0,
            message: ChoiceMessage {
                role: "assistant".to_string(),
                content,
            },
            finish_reason: "stop",
        }],
    }))
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

async fn fetch_ollama_models(ollama_url: &str) -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/tags", ollama_url.trim_end_matches('/'));

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Ok(vec![]);
    }

    let json: Value = response.json().await.map_err(|e| e.to_string())?;

    let models = json
        .get("models")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Ok(models)
}

fn gateway_error(msg: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: ErrorDetail {
                message: msg.to_string(),
                r#type: "gateway_error".to_string(),
            },
        }),
    )
}

// ─── Server lifecycle ──────────────────────────────────────────────────────────

/// Starts the gateway HTTP server.
pub async fn start_server(state: Arc<GatewayState>, port: u16) -> Result<(), String> {
    if state.running.load(Ordering::SeqCst) {
        return Err("Gateway is already running".to_string());
    }

    let cancel = CancellationToken::new();
    *state.cancel.lock().await = Some(cancel.clone());
    *state.port.lock().unwrap() = port;
    state.running.store(true, Ordering::SeqCst);

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(list_models))
        .route("/v1/chat/completions", post(chat_completions))
        .layer(
            tower_http::cors::CorsLayer::permissive(),
        )
        .with_state(state.clone());

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| {
            state.running.store(false, Ordering::SeqCst);
            format!("Failed to bind gateway port {}: {e}", port)
        })?;

    eprintln!("thuki: [gateway] listening on 127.0.0.1:{}", port);

    // Serve until cancelled.
    axum::serve(listener, app)
        .with_graceful_shutdown(cancel.cancelled_owned())
        .await
        .map_err(|e| format!("Gateway server error: {e}"))?;

    state.running.store(false, Ordering::SeqCst);
    eprintln!("thuki: [gateway] stopped");
    Ok(())
}

/// Stops the gateway HTTP server.
pub async fn stop_server(state: &GatewayState) -> Result<(), String> {
    if !state.running.load(Ordering::SeqCst) {
        return Ok(());
    }

    if let Some(cancel) = state.cancel.lock().await.take() {
        cancel.cancel();
    }

    state.running.store(false, Ordering::SeqCst);
    Ok(())
}

// ─── Tauri commands ────────────────────────────────────────────────────────────

#[cfg_attr(coverage_nightly, coverage(off))]
#[tauri::command]
pub async fn start_gateway(
    port: u16,
    state: tauri::State<'_, Arc<GatewayState>>,
) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = start_server(state, port).await {
            eprintln!("thuki: [gateway] error: {e}");
        }
    });
    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[tauri::command]
pub async fn stop_gateway(state: tauri::State<'_, Arc<GatewayState>>) -> Result<(), String> {
    stop_server(state.inner()).await
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[tauri::command]
pub fn get_gateway_status(state: tauri::State<'_, Arc<GatewayState>>) -> Value {
    let running = state.running.load(Ordering::SeqCst);
    let port = *state.port.lock().unwrap();
    json!({
        "running": running,
        "port": port,
        "url": if running { format!("http://127.0.0.1:{}", port) } else { String::new() },
    })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gateway_state_new_defaults() {
        let state = GatewayState::new();
        assert!(!state.running.load(Ordering::SeqCst));
        assert_eq!(*state.port.lock().unwrap(), 18789);
        assert_eq!(*state.ollama_url.lock().unwrap(), "http://127.0.0.1:11434");
    }

    #[test]
    fn gateway_state_running_flag() {
        let state = GatewayState::new();
        assert!(!state.running.load(Ordering::SeqCst));
        state.running.store(true, Ordering::SeqCst);
        assert!(state.running.load(Ordering::SeqCst));
    }

    #[test]
    fn error_response_format() {
        let (status, body) = gateway_error("test error");
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        let json = serde_json::to_value(&*body).unwrap();
        assert_eq!(json["error"]["message"], "test error");
        assert_eq!(json["error"]["type"], "gateway_error");
    }

    #[test]
    fn models_response_serialization() {
        let resp = ModelsResponse {
            object: "list",
            data: vec![ModelObject {
                id: "test-model".to_string(),
                object: "model",
                created: 0,
                owned_by: "ollama",
            }],
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["object"], "list");
        assert_eq!(json["data"][0]["id"], "test-model");
    }

    #[test]
    fn chat_completion_response_serialization() {
        let resp = ChatCompletionResponse {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion",
            created: 0,
            model: "test-model".to_string(),
            choices: vec![Choice {
                index: 0,
                message: ChoiceMessage {
                    role: "assistant".to_string(),
                    content: "Hello!".to_string(),
                },
                finish_reason: "stop",
            }],
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["object"], "chat.completion");
        assert_eq!(json["choices"][0]["message"]["content"], "Hello!");
    }
}