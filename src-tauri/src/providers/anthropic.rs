//! Anthropic API provider for streaming messages with computer use.
//!
//! Implements the Anthropic Messages API with SSE streaming,
//! computer_use tool type, and converts responses to the
//! unified ProviderChunk format.

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use super::{ProviderChunk, ToolCall};

// ─── Request types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<AnthropicTool>,
    stream: bool,
}

#[derive(Serialize, Clone)]
#[serde(tag = "role")]
enum AnthropicMessage {
    #[serde(rename = "user")]
    User { content: Value },
    #[serde(rename = "assistant")]
    Assistant { content: Value },
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum AnthropicTool {
    #[serde(rename = "computer_20250124")]
    Computer20250124 {
        name: String,
        display_width_px: u32,
        display_height_px: u32,
    },
}

// ─── Response types ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[allow(dead_code)]
struct AnthropicResponse {
    content: Vec<AnthropicContentBlock>,
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
enum AnthropicContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
}

// ─── SSE streaming types ───────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    content_block: Option<AnthropicStreamContentBlock>,
    delta: Option<AnthropicStreamDelta>,
    #[allow(dead_code)]
    message: Option<AnthropicStreamMessage>,
}

#[derive(Deserialize)]
struct AnthropicStreamContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[allow(dead_code)]
    text: Option<String>,
    id: Option<String>,
    name: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicStreamDelta {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    delta_type: Option<String>,
    text: Option<String>,
    partial_json: Option<String>,
    #[allow(dead_code)]
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicStreamMessage {
    #[allow(dead_code)]
    stop_reason: Option<String>,
}

// ─── Computer use tool ─────────────────────────────────────────────────────────

/// Returns the Anthropic computer_use tool definition with standard screen dimensions.
pub fn anthropic_computer_tool(width: u32, height: u32) -> Vec<AnthropicTool> {
    vec![AnthropicTool::Computer20250124 {
        name: "computer".to_string(),
        display_width_px: width,
        display_height_px: height,
    }]
}

// ─── Streaming chat ────────────────────────────────────────────────────────────

/// Streams a message from the Anthropic Messages API.
///
/// Sends the conversation with computer_use tool, streams the response,
/// and emits `ProviderChunk` variants via the `on_chunk` callback.
pub async fn stream_anthropic_chat(
    base_url: &str,
    model: &str,
    api_key: &str,
    system_prompt: &str,
    messages: Vec<crate::commands::ChatMessage>,
    include_tools: bool,
    screenshot_b64: Option<String>,
    screen_width: u32,
    screen_height: u32,
    client: &reqwest::Client,
    cancel_token: CancellationToken,
    mut on_chunk: impl FnMut(ProviderChunk),
) -> Result<String, String> {
    let url = format!("{}/v1/messages", base_url.trim_end_matches('/'));

    // Convert Ollama messages to Anthropic format.
    let mut anthropic_messages: Vec<AnthropicMessage> = Vec::new();

    for m in &messages {
        if m.role == "system" {
            continue; // System prompt handled separately.
        }

        let mut content_parts: Vec<Value> = Vec::new();

        // Add text content.
        if !m.content.is_empty() {
            content_parts.push(serde_json::json!({
                "type": "text",
                "text": m.content,
            }));
        }

        // Add image content.
        if let Some(ref imgs) = m.images {
            for img in imgs {
                content_parts.push(serde_json::json!({
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": "image/png",
                        "data": img,
                    },
                }));
            }
        }

        // If there's a screenshot for computer use, add it.
        if let Some(ref b64) = screenshot_b64 {
            content_parts.push(serde_json::json!({
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": "image/png",
                    "data": b64,
                },
            }));
        }

        if m.role == "user" {
            anthropic_messages.push(AnthropicMessage::User {
                content: if content_parts.len() == 1 {
                    content_parts.into_iter().next().unwrap()
                } else {
                    Value::Array(content_parts)
                },
            });
        } else {
            anthropic_messages.push(AnthropicMessage::Assistant {
                content: if content_parts.len() == 1 {
                    content_parts.into_iter().next().unwrap()
                } else {
                    Value::Array(content_parts)
                },
            });
        }
    }

    let tools = if include_tools {
        anthropic_computer_tool(screen_width, screen_height)
    } else {
        vec![]
    };

    let request = AnthropicRequest {
        model: model.to_string(),
        max_tokens: 4096,
        messages: anthropic_messages,
        tools,
        stream: true,
    };

    let mut req_builder = client
        .post(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json");

    // Anthropic requires system prompt as a top-level field.
    if !system_prompt.is_empty() {
        let mut body = serde_json::to_value(&request)
            .map_err(|e| format!("Serialize error: {e}"))?;
        body.as_object_mut()
            .expect("request serializes to object")
            .insert("system".to_string(), serde_json::Value::String(system_prompt.to_string()));
        req_builder = req_builder.json(&body);
    } else {
        req_builder = req_builder.json(&request);
    }

    let response = req_builder
        .send()
        .await
        .map_err(|e| format!("Anthropic request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Anthropic returned status {}: {}", status, body));
    }

    let mut stream = response.bytes_stream();
    let mut accumulated = String::new();
    let mut current_tool_id = String::new();
    let mut current_tool_name = String::new();
    let mut current_tool_args = String::new();

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
                                // Emit any pending tool call.
                                if !current_tool_id.is_empty() {
                                    on_chunk(ProviderChunk::ToolCalls(vec![ToolCall {
                                        id: current_tool_id.clone(),
                                        name: current_tool_name.clone(),
                                        arguments: current_tool_args.clone(),
                                    }]));
                                    current_tool_id.clear();
                                    current_tool_name.clear();
                                    current_tool_args.clear();
                                }
                                on_chunk(ProviderChunk::Done);
                                return Ok(accumulated);
                            }

                            let parsed: Result<AnthropicStreamEvent, _> = serde_json::from_str(data);
                            if let Ok(event) = parsed {
                                match event.event_type.as_str() {
                                    "content_block_start" => {
                                        if let Some(block) = event.content_block {
                                            if block.block_type == "tool_use" {
                                                current_tool_id = block.id.unwrap_or_default();
                                                current_tool_name = block.name.unwrap_or_default();
                                                current_tool_args.clear();
                                            }
                                        }
                                    }
                                    "content_block_delta" => {
                                        if let Some(delta) = event.delta {
                                            if let Some(text) = delta.text {
                                                if !text.is_empty() {
                                                    accumulated.push_str(&text);
                                                    on_chunk(ProviderChunk::Token(text));
                                                }
                                            }
                                            if let Some(partial_json) = delta.partial_json {
                                                current_tool_args.push_str(&partial_json);
                                            }
                                        }
                                    }
                                    "content_block_stop" => {
                                        // Tool call complete — emit it.
                                        if !current_tool_id.is_empty() {
                                            on_chunk(ProviderChunk::ToolCalls(vec![ToolCall {
                                                id: current_tool_id.clone(),
                                                name: current_tool_name.clone(),
                                                arguments: current_tool_args.clone(),
                                            }]));
                                            current_tool_id.clear();
                                            current_tool_name.clear();
                                            current_tool_args.clear();
                                        }
                                    }
                                    "message_stop" => {
                                        on_chunk(ProviderChunk::Done);
                                        return Ok(accumulated);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        on_chunk(ProviderChunk::Error(format!("Stream error: {e}")));
                        return Err(format!("Anthropic stream error: {e}"));
                    }
                    None => {
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
    fn anthropic_computer_tool_creation() {
        let tools = anthropic_computer_tool(1920, 1080);
        assert_eq!(tools.len(), 1);
        let json = serde_json::to_string(&tools[0]).unwrap();
        assert!(json.contains("computer_20250124"));
        assert!(json.contains("1920"));
        assert!(json.contains("1080"));
    }

    #[test]
    fn anthropic_message_serialization() {
        let msg = AnthropicMessage::User {
            content: serde_json::json!("Hello"),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
    }
}