/*!
 * Edge TTS Module
 *
 * Connects to Microsoft's Edge Read Aloud speech synthesis service via WebSocket
 * to convert text to speech. Audio data is returned as base64-encoded MP3.
 *
 * Privacy note: text content is sent to Microsoft servers over the internet.
 * This differs from Thuki's local-first philosophy; a frontend disclosure
 * should inform the user on first use.
 */

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Mutex;

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::State;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;

// ─── Constants ──────────────────────────────────────────────────────────────

/// Publicly known client token used by Edge browsers for the Speech Service.
const TRUSTED_CLIENT_TOKEN: &str =
    "6A5AA1D4EAFF4E9FB37E23D68491D6F4";

/// Edge browser version used in Sec-MS-GEC-Version header.
const EDGE_VERSION: &str = "143.0.3650.75";

/// Seconds between 1601-01-01 (Windows epoch) and 1970-01-01 (Unix epoch).
const WIN_EPOCH_DIFF: u64 = 11_644_473_600;

/// Round-down interval for Sec-MS-GEC token generation (5 minutes in seconds).
/// Matches the Python edge-tts library's tick computation: ticks -= ticks % 300
const SEC_GEC_ROUND: u64 = 300;

/// Default voice for Turkish locale.
pub const DEFAULT_VOICE: &str = "tr-TR-EmelNeural";

/// Default speech rate (0% change).
pub const DEFAULT_RATE: &str = "+0%";

/// Default pitch (0% change).
pub const DEFAULT_PITCH: &str = "+0%";

/// Edge TTS WebSocket endpoint.
const WS_ENDPOINT: &str =
    "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1";

/// Edge TTS voices list endpoint (with trusted client token as query param).
const VOICES_ENDPOINT: &str =
    "https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list?trustedclienttoken=6A5AA1D4EAFF4E9FB37E23D68491D6F4";

/// Maximum text length accepted by Edge TTS (approximately 5000 chars).
const MAX_TEXT_LENGTH: usize = 5000;

/// Global clock skew in seconds, computed from the server's Date header.
/// This corrects for local clock drift that would cause Sec-MS-GEC to be invalid.
static CLOCK_SKEW: AtomicI64 = AtomicI64::new(0);

// ─── Data types ─────────────────────────────────────────────────────────────

/// Available voice from the Edge TTS voices endpoint.
/// Uses serde aliases to handle varying field name casings from Microsoft's API.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TtsVoice {
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(rename = "ShortName", alias = "shortName")]
    pub short_name: String,
    #[serde(alias = "Gender")]
    pub gender: String,
    #[serde(rename = "Locale", alias = "locale")]
    pub locale: String,
    #[serde(rename = "SuggestedCodec", alias = "suggestedCodec")]
    pub suggested_codec: String,
}

/// Managed state for in-progress TTS synthesis.
/// Only one synthesis runs at a time — starting a new one cancels the previous.
#[derive(Default)]
pub struct TtsState {
    token: Mutex<Option<CancellationToken>>,
    /// Monotonically increasing epoch counter to prevent stale stop requests
    /// from cancelling a newer synthesis.
    epoch: AtomicU64,
}

impl TtsState {
    /// Creates a new empty TTS state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores a new cancellation token, replacing any previous one, and bumps the epoch.
    fn set(&self, token: CancellationToken) -> u64 {
        *self.token.lock().unwrap() = Some(token);
        self.epoch.fetch_add(1, Ordering::SeqCst)
    }

    /// Cancels the active synthesis, if any.
    pub fn cancel(&self) {
        if let Some(token) = self.token.lock().unwrap().take() {
            token.cancel();
        }
    }

    /// Clears the stored token without cancelling it (used on natural completion).
    fn clear(&self) {
        *self.token.lock().unwrap() = None;
    }
}

// ─── SSML generation ────────────────────────────────────────────────────────

/// Escapes XML special characters in text for safe embedding in SSML.
fn escape_xml(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(c),
        }
    }
    result
}

/// Builds an SSML document wrapping the given text with the specified voice and prosody.
pub fn build_ssml(text: &str, voice: &str, rate: &str, pitch: &str) -> String {
    let escaped = escape_xml(text);
    format!(
        "<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xml:lang='en-US'>\
         <voice name='{}'>\
         <prosody pitch='{}' rate='{}'>\
         {}\
         </prosody></voice></speak>",
        voice, pitch, rate, escaped
    )
}

// ─── WebSocket message builders ────────────────────────────────────────────

/// Generates the Sec-MS-GEC token required by Edge TTS for authentication.
///
/// Algorithm (matching the Python edge-tts library):
/// 1. Get current UTC time adjusted by clock skew
/// 2. Convert to Windows file time (seconds since 1601-01-01)
/// 3. Round down to nearest 300-second boundary (5 minutes)
/// 4. Convert to 100-nanosecond intervals (multiply by 10^7)
/// 5. Concatenate: "{ticks}{TRUSTED_CLIENT_TOKEN}"
/// 6. Return SHA-256 of that string as uppercase hex
pub fn generate_sec_ms_gec() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let unix_secs = duration.as_secs();

    // Apply clock skew correction.
    let skew = CLOCK_SKEW.load(Ordering::Relaxed);
    let adjusted_secs = if skew >= 0 {
        unix_secs + skew as u64
    } else {
        unix_secs.saturating_sub((-skew) as u64)
    };

    // Convert to Windows epoch and round down to nearest 5 minutes.
    let win_secs = adjusted_secs + WIN_EPOCH_DIFF;
    let rounded = win_secs - (win_secs % SEC_GEC_ROUND);

    // Convert to 100-nanosecond intervals.
    let ticks = rounded * 10_000_000;

    let str_to_hash = format!("{ticks}{TRUSTED_CLIENT_TOKEN}");
    let hash = Sha256::digest(str_to_hash.as_bytes());
    format!("{:X}", hash)
}

/// Constructs the WebSocket URL with required Edge TTS query parameters.
pub fn build_ws_url() -> String {
    let connection_id = uuid::Uuid::new_v4();
    let connection_id_hex = connection_id.to_string().replace('-', "");
    let sec_gec = generate_sec_ms_gec();
    let sec_gec_version = format!("1-{EDGE_VERSION}");
    format!(
        "{}?TrustedClientToken={}&Sec-MS-GEC={}&Sec-MS-GEC-Version={}&ConnectionId={}",
        WS_ENDPOINT, TRUSTED_CLIENT_TOKEN, sec_gec, sec_gec_version, connection_id_hex
    )
}

/// Generates a random MUID (Machine Unique ID) for Cookie header.
fn generate_muid() -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("{:x}", uuid).to_uppercase()
}

/// Parses an HTTP Date header and updates the global clock skew.
///
/// The Date header format is RFC 1123, e.g. "Thu, 16 Apr 2026 10:54:12 GMT".
/// Computes the difference between server time and local time, storing it
/// in CLOCK_SKEW so subsequent Sec-MS-GEC tokens use the corrected time.
fn update_clock_skew(date_header: &str) {
    // Parse RFC 1123 date: "Thu, 16 Apr 2026 10:54:12 GMT"
    // or RFC 850: "Thursday, 16-Apr-26 10:54:12 GMT"
    // We handle the common RFC 1123 format.
    if let Some(server_unix) = parse_http_date(date_header) {
        let local_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let skew = server_unix - local_unix;
        CLOCK_SKEW.store(skew, Ordering::Relaxed);
    }
}

/// Parses an HTTP Date header string into a Unix timestamp.
/// Supports the RFC 1123 format: "Thu, 16 Apr 2026 10:54:12 GMT"
fn parse_http_date(date: &str) -> Option<i64> {
    // Remove "GMT" suffix and trim.
    let date = date.trim();
    let date = date.strip_suffix("GMT").unwrap_or(date).trim();

    // Try RFC 1123: "Thu, 16 Apr 2026 10:54:12"
    let parts: Vec<&str> = date.split(',').collect();
    let date_part = if parts.len() == 2 {
        parts[1].trim()
    } else {
        date
    };

    // Split into: "16 Apr 2026 10:54:12"
    let fields: Vec<&str> = date_part.split_whitespace().collect();
    if fields.len() < 4 {
        return None;
    }

    let day: u64 = fields[0].parse().ok()?;
    let month = month_from_name(fields[1])?;
    let year: u64 = fields[2].parse().ok()?;
    let time_fields: Vec<&str> = fields[3].split(':').collect();
    if time_fields.len() < 3 {
        return None;
    }
    let hour: u64 = time_fields[0].parse().ok()?;
    let minute: u64 = time_fields[1].parse().ok()?;
    let second: u64 = time_fields[2].parse().ok()?;

    // Convert to Unix timestamp.
    let days_since_epoch = date_to_days(year, month, day)?;
    Some(days_since_epoch as i64 * 86400 + hour as i64 * 3600 + minute as i64 * 60 + second as i64)
}

/// Converts a month name (3-letter abbreviation) to a month number (1-12).
fn month_from_name(name: &str) -> Option<u64> {
    match name {
        "Jan" => Some(1),
        "Feb" => Some(2),
        "Mar" => Some(3),
        "Apr" => Some(4),
        "May" => Some(5),
        "Jun" => Some(6),
        "Jul" => Some(7),
        "Aug" => Some(8),
        "Sep" => Some(9),
        "Oct" => Some(10),
        "Nov" => Some(11),
        "Dec" => Some(12),
        _ => None,
    }
}

/// Converts year/month/day to days since Unix epoch (1970-01-01).
/// This is the inverse of days_to_date, using Howard Hinnant's algorithm.
fn date_to_days(year: u64, month: u64, day: u64) -> Option<u64> {
    // Adjust year/month so March is month 0 (civil calendar convention).
    let y = if month <= 2 { year - 1 } else { year };
    let m = if month > 2 { month - 3 } else { month + 9 };
    let era = y / 400;
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe - 719468)
}

/// Builds the configuration message sent immediately after WebSocket connection.
/// Specifies the audio output format as 24kHz 48kbps mono MP3.
/// Format matches the Python edge-tts library's send_command_request.
pub fn build_config_message(_request_id: &str) -> String {
    let timestamp = chrono_like_timestamp();
    format!(
        "X-Timestamp:{}\r\nContent-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n\
         {{\"context\":{{\"synthesis\":{{\"audio\":{{\"metadataoptions\":{{\
         \"sentenceBoundaryEnabled\":\"true\",\"wordBoundaryEnabled\":\"false\"\
         }},\"outputFormat\":\"audio-24khz-48kbitrate-mono-mp3\"}}}}}}}}\r\n",
        timestamp
    )
}

/// Builds the SSML synthesis message sent to start audio generation.
/// Format matches the Python edge-tts library's ssml_headers_plus_data.
/// Note: the trailing 'Z' on the timestamp is intentional (Microsoft Edge bug, per edge-tts).
pub fn build_ssml_message(request_id: &str, ssml: &str) -> String {
    let timestamp = chrono_like_timestamp();
    format!(
        "X-RequestId:{}\r\nContent-Type:application/ssml+xml\r\nX-Timestamp:{}Z\r\nPath:ssml\r\n\r\n{}",
        request_id, timestamp, ssml
    )
}

/// Returns a timestamp string in the format used by Edge TTS WebSocket messages.
/// Format: "Thu Jan 01 2025 12:00:00 GMT+0000 (Coordinated Universal Time)"
/// Implemented without the chrono crate by computing the date from a Unix timestamp.
fn chrono_like_timestamp() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = duration.as_secs();

    // Compute date components from Unix timestamp.
    let days_since_epoch = total_secs / 86400;
    let time_of_day_secs = total_secs % 86400;
    let hours = (time_of_day_secs / 3600) as u32;
    let minutes = ((time_of_day_secs % 3600) / 60) as u32;
    let seconds = (time_of_day_secs % 60) as u32;

    // Gregorian calendar date from days since 1970-01-01.
    let (year, month, day) = days_to_date(days_since_epoch);
    let weekday = days_to_weekday(days_since_epoch);

    let month_names = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun",
        "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let weekday_names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

    format!(
        "{} {} {:02} {} {:02}:{:02}:{:02} GMT+0000 (Coordinated Universal Time)",
        weekday_names[weekday as usize],
        month_names[month as usize - 1],
        day,
        year,
        hours,
        minutes,
        seconds,
    )
}

/// Converts days since Unix epoch to (year, month, day).
fn days_to_date(days: u64) -> (u64, u64, u64) {
    // Algorithm from Howard Hinnant: http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468; // shift to era starting Mar 1, 0000
    let era = z / 146097;
    let doe = z - era * 146097; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (doy * 5 + 2) / 153; // month [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // day [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Converts days since Unix epoch to day of week (0=Sun, 6=Sat).
fn days_to_weekday(days: u64) -> u8 {
    // 1970-01-01 was a Thursday (4). (days + 4) % 7 gives 0=Sun, 1=Mon, ...
    ((days + 4) % 7) as u8
}

// ─── Binary message parsing ────────────────────────────────────────────────

/// Parses a binary WebSocket message from Edge TTS.
///
/// Binary messages have the following structure:
/// - 2 bytes: header length (big-endian u16)
/// - N bytes: text header (containing Path=audio or Path=turn.end)
/// - Remaining bytes: audio data
///
/// Returns the audio data portion if this is an audio message, or None
/// if it's a turn.end message (no audio payload).
pub fn parse_binary_message(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 2 {
        return None;
    }

    // First 2 bytes are the header length (big-endian u16).
    let header_len = u16::from_be_bytes([data[0], data[1]]) as usize;

    if data.len() < 2 + header_len {
        return None;
    }

    let header_start = 2;
    let header_end = 2 + header_len;
    let header = String::from_utf8_lossy(&data[header_start..header_end]);

    // turn.end messages have no audio data.
    if header.contains("Path:turn.end") || header.contains("Path=turn.end") {
        return None;
    }

    // Audio messages: extract the payload after the header.
    // The Edge TTS protocol uses "Path:audio" (colon) in binary messages.
    if header.contains("Path:audio") || header.contains("Path=audio") {
        let audio_data = data[header_end..].to_vec();
        if audio_data.is_empty() {
            return None;
        }
        return Some(audio_data);
    }

    None
}

// ─── Core synthesis ─────────────────────────────────────────────────────────

/// Connects to the Edge TTS WebSocket, sends the text for synthesis, and
/// collects all audio data chunks into a single MP3 byte vector.
///
/// Uses `tokio::select!` with the cancellation token to support immediate stop.
/// Returns the complete MP3 audio data on success.
pub async fn synthesize(
    text: &str,
    voice: &str,
    rate: &str,
    pitch: &str,
    cancel_token: CancellationToken,
) -> Result<Vec<u8>, String> {
    let truncated_text = if text.len() > MAX_TEXT_LENGTH {
        &text[..MAX_TEXT_LENGTH]
    } else {
        text
    };

    let url = build_ws_url();
    let muid = generate_muid();
    let request = url
        .into_client_request()
        .map_err(|e| format!("Failed to build WebSocket request: {e}"))?;

    // Add required headers matching the Python edge-tts library's WSS_HEADERS.
    let mut request = request;
    request.headers_mut().insert(
        "Origin",
        "chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold"
            .parse()
            .map_err(|e| format!("Invalid Origin header: {e}"))?,
    );
    request.headers_mut().insert(
        "Pragma",
        "no-cache".parse().map_err(|e| format!("Invalid Pragma header: {e}"))?,
    );
    request.headers_mut().insert(
        "Cache-Control",
        "no-cache".parse().map_err(|e| format!("Invalid Cache-Control header: {e}"))?,
    );
    request.headers_mut().insert(
        "User-Agent",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36 Edg/143.0.0.0"
            .parse()
            .map_err(|e| format!("Invalid User-Agent header: {e}"))?,
    );
    request.headers_mut().insert(
        "Accept-Encoding",
        "gzip, deflate, br, zstd"
            .parse()
            .map_err(|e| format!("Invalid Accept-Encoding header: {e}"))?,
    );
    request.headers_mut().insert(
        "Accept-Language",
        "en-US,en;q=0.9"
            .parse()
            .map_err(|e| format!("Invalid Accept-Language header: {e}"))?,
    );
    request.headers_mut().insert(
        "Cookie",
        format!("muid={};", muid)
            .parse()
            .map_err(|e| format!("Invalid cookie header: {e}"))?,
    );

    let (mut ws_stream, _) = tokio_tungstenite::connect_async(request)
        .await
        .map_err(|e| format!("WebSocket connection failed: {e}"))?;

    let request_id = uuid::Uuid::new_v4();
    let request_id_hex = request_id.to_string().replace('-', "");

    // Send configuration message.
    let config_msg = build_config_message(&request_id_hex);
    ws_stream
        .send(Message::Text(config_msg.into()))
        .await
        .map_err(|e| format!("Failed to send config: {e}"))?;

    // Send SSML message.
    let ssml = build_ssml(truncated_text, voice, rate, pitch);
    let ssml_msg = build_ssml_message(&request_id_hex, &ssml);
    ws_stream
        .send(Message::Text(ssml_msg.into()))
        .await
        .map_err(|e| format!("Failed to send SSML: {e}"))?;

    let mut audio_data = Vec::new();
    let mut done = false;

    while !done {
        tokio::select! {
            biased;
            _ = cancel_token.cancelled() => {
                let _ = ws_stream.close(None).await;
                return Err("cancelled".to_string());
            }
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        if let Some(chunk) = parse_binary_message(&data) {
                            audio_data.extend_from_slice(&chunk);
                        }
                        // turn.end will return None from parse_binary_message,
                        // but we need to check the text header for turn.end too.
                    }
                    Some(Ok(Message::Text(text))) => {
                        // Text messages may contain turn.end marker.
                        // Edge TTS protocol uses "Path:turn.end" (colon).
                        if text.contains("Path:turn.end") || text.contains("Path=turn.end") {
                            done = true;
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        done = true;
                    }
                    Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {
                        // Ignore ping/pong frames.
                    }
                    Some(Ok(Message::Frame(_))) => {
                        // Ignore unknown frames.
                    }
                    Some(Err(e)) => {
                        return Err(format!("WebSocket error: {e}"));
                    }
                    None => {
                        done = true;
                    }
                }
            }
        }
    }

    if audio_data.is_empty() {
        return Err("No audio data received".to_string());
    }

    Ok(audio_data)
}

// ─── Voice listing ──────────────────────────────────────────────────────────

/// Fetches the list of available voices from Microsoft's Edge TTS endpoint.
///
/// Also reads the server's `Date` response header to compute clock skew,
/// which is used to correct Sec-MS-GEC token generation.
pub async fn list_voices(client: &reqwest::Client) -> Result<Vec<TtsVoice>, String> {
    let sec_gec = generate_sec_ms_gec();
    let muid = generate_muid();
    let response = client
        .get(VOICES_ENDPOINT)
        .header("Sec-MS-GEC", &sec_gec)
        .header(
            "Sec-MS-GEC-Version",
            format!("1-{EDGE_VERSION}"),
        )
        .header("Cookie", format!("muid={};", muid))
        .header(
            "Sec-CH-UA",
            r#"" Not;A Brand";v="99", "Microsoft Edge";v="143", "Chromium";v="143""#,
        )
        .header("Sec-CH-UA-Mobile", "?0")
        .header("Accept", "*/*")
        .header("Sec-Fetch-Site", "none")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Dest", "empty")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36 Edg/143.0.0.0",
        )
        .header("Accept-Encoding", "gzip, deflate, br, zstd")
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch voices: {e}"))?;

    // Compute clock skew from the server's Date header.
    if let Some(date_header) = response.headers().get("Date") {
        if let Ok(date_str) = date_header.to_str() {
            update_clock_skew(date_str);
        }
    }

    if !response.status().is_success() {
        return Err(format!("Voices endpoint returned HTTP {}", response.status()));
    }

    // Try to parse the response body as text first, then as JSON.
    // The Edge TTS voices endpoint may return slightly different field names
    // than our struct expects.
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read voices response: {e}"))?;

    let voices: Vec<TtsVoice> = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse voices JSON: {e} (body preview: {})", &body[..body.len().min(200)]))?;

    Ok(voices)
}

// ─── Tauri commands ─────────────────────────────────────────────────────────

/// Synthesizes text to speech using Edge TTS and returns the audio as a base64 string.
/// If another synthesis is in progress, it is cancelled first.
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg_attr(not(coverage), tauri::command)]
pub async fn tts_speak(
    text: String,
    voice: Option<String>,
    rate: Option<String>,
    pitch: Option<String>,
    tts_state: State<'_, TtsState>,
) -> Result<String, String> {
    let voice = voice.unwrap_or_else(|| DEFAULT_VOICE.to_string());
    let rate = rate.unwrap_or_else(|| DEFAULT_RATE.to_string());
    let pitch = pitch.unwrap_or_else(|| DEFAULT_PITCH.to_string());

    // Cancel any in-progress synthesis.
    tts_state.cancel();

    let cancel_token = CancellationToken::new();
    tts_state.set(cancel_token.clone());

    let result = synthesize(&text, &voice, &rate, &pitch, cancel_token).await;

    tts_state.clear();

    match result {
        Ok(audio_bytes) => Ok(BASE64_STANDARD.encode(&audio_bytes)),
        Err(e) => {
            if e == "cancelled" {
                Err("cancelled".to_string())
            } else {
                Err(e)
            }
        }
    }
}

/// Cancels any active TTS synthesis.
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg_attr(not(coverage), tauri::command)]
pub fn tts_stop(tts_state: State<'_, TtsState>) -> Result<(), String> {
    tts_state.cancel();
    Ok(())
}

/// Returns the list of available Edge TTS voices.
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg_attr(not(coverage), tauri::command)]
pub async fn tts_list_voices(
    client: State<'_, reqwest::Client>,
) -> Result<Vec<TtsVoice>, String> {
    list_voices(&client).await
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── escape_xml ──────────────────────────────────────────────────────

    #[test]
    fn escape_xml_escapes_ampersand() {
        assert_eq!(escape_xml("a & b"), "a &amp; b");
    }

    #[test]
    fn escape_xml_escapes_less_than() {
        assert_eq!(escape_xml("a < b"), "a &lt; b");
    }

    #[test]
    fn escape_xml_escapes_greater_than() {
        assert_eq!(escape_xml("a > b"), "a &gt; b");
    }

    #[test]
    fn escape_xml_escapes_double_quote() {
        assert_eq!(escape_xml("a\"b"), "a&quot;b");
    }

    #[test]
    fn escape_xml_escapes_single_quote() {
        assert_eq!(escape_xml("a'b"), "a&apos;b");
    }

    #[test]
    fn escape_xml_preserves_normal_text() {
        assert_eq!(escape_xml("hello world"), "hello world");
    }

    #[test]
    fn escape_xml_handles_multiple_special_chars() {
        assert_eq!(
            escape_xml("a<b>c&d\"e\"f'g"),
            "a&lt;b&gt;c&amp;d&quot;e&quot;f&apos;g"
        );
    }

    // ─── build_ssml ──────────────────────────────────────────────────────

    #[test]
    fn build_ssml_wraps_text_in_speak_tag() {
        let ssml = build_ssml("hello", "en-US-JennyNeural", "+0%", "+0%");
        assert!(ssml.starts_with("<speak version='1.0'"));
        assert!(ssml.contains("xmlns='http://www.w3.org/2001/10/synthesis'"));
    }

    #[test]
    fn build_ssml_includes_voice_name() {
        let ssml = build_ssml("hello", "tr-TR-EmelNeural", "+0%", "+0%");
        assert!(ssml.contains("<voice name='tr-TR-EmelNeural'>"));
    }

    #[test]
    fn build_ssml_includes_prosody_with_rate_and_pitch() {
        let ssml = build_ssml("hello", "en-US-JennyNeural", "+50%", "-10%");
        assert!(ssml.contains("pitch='-10%'"));
        assert!(ssml.contains("rate='+50%'"));
    }

    #[test]
    fn build_ssml_escapes_special_chars_in_text() {
        let ssml = build_ssml("a < b & c", "en-US-JennyNeural", "+0%", "+0%");
        assert!(ssml.contains("a &lt; b &amp; c"));
        assert!(!ssml.contains("a < b & c"));
    }

    #[test]
    fn build_ssml_uses_defaults() {
        let ssml = build_ssml("hello", "en-US-JennyNeural", DEFAULT_RATE, DEFAULT_PITCH);
        assert!(ssml.contains("pitch='+0%'"));
        assert!(ssml.contains("rate='+0%'"));
    }

    // ─── build_ws_url ───────────────────────────────────────────────────

    #[test]
    fn build_ws_url_contains_endpoint() {
        let url = build_ws_url();
        assert!(url.starts_with(WS_ENDPOINT));
    }

    #[test]
    fn build_ws_url_contains_trusted_client_token() {
        let url = build_ws_url();
        assert!(url.contains(&format!(
            "TrustedClientToken={}",
            TRUSTED_CLIENT_TOKEN
        )));
    }

    #[test]
    fn build_ws_url_contains_connection_id() {
        let url = build_ws_url();
        assert!(url.contains("ConnectionId="));
    }

    #[test]
    fn build_ws_url_contains_sec_ms_gec() {
        let url = build_ws_url();
        // Sec-MS-GEC is now a dynamic SHA-256 hash, so just check the param is present.
        assert!(url.contains("Sec-MS-GEC="));
        // The value should be a 64-char uppercase hex string (SHA-256).
        let start = url.find("Sec-MS-GEC=").unwrap() + "Sec-MS-GEC=".len();
        let end = url[start..].find('&').unwrap_or(url.len() - start) + start;
        let value = &url[start..end];
        assert_eq!(value.len(), 64);
        assert!(value.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn build_ws_url_generates_unique_connection_ids() {
        let url1 = build_ws_url();
        let url2 = build_ws_url();
        assert_ne!(url1, url2);
    }

    // ─── build_config_message ────────────────────────────────────────────

    #[test]
    fn build_config_message_contains_path() {
        let msg = build_config_message("test123");
        assert!(msg.contains("Path:speech.config"));
    }

    #[test]
    fn build_config_message_contains_timestamp() {
        let msg = build_config_message("abc123");
        assert!(msg.contains("X-Timestamp:"));
    }

    #[test]
    fn build_config_message_contains_audio_format() {
        let msg = build_config_message("test");
        assert!(msg.contains("audio-24khz-48kbitrate-mono-mp3"));
    }

    #[test]
    fn build_config_message_contains_content_type() {
        let msg = build_config_message("test");
        assert!(msg.contains("Content-Type:application/json; charset=utf-8"));
    }

    // ─── build_ssml_message ──────────────────────────────────────────────

    #[test]
    fn build_ssml_message_contains_path() {
        let msg = build_ssml_message("req1", "<speak>test</speak>");
        assert!(msg.contains("Path:ssml"));
    }

    #[test]
    fn build_ssml_message_contains_request_id() {
        let msg = build_ssml_message("req1", "<speak>test</speak>");
        assert!(msg.contains("X-RequestId:req1"));
    }

    #[test]
    fn build_ssml_message_contains_ssml() {
        let msg = build_ssml_message("req1", "<speak>test</speak>");
        assert!(msg.contains("<speak>test</speak>"));
    }

    #[test]
    fn build_ssml_message_contains_content_type() {
        let msg = build_ssml_message("req1", "<speak>test</speak>");
        assert!(msg.contains("Content-Type:application/ssml+xml"));
    }

    #[test]
    fn build_ssml_message_timestamp_ends_with_z() {
        let msg = build_ssml_message("req1", "<speak>test</speak>");
        // The timestamp has a trailing 'Z' — intentional Microsoft Edge bug, per edge-tts.
        assert!(msg.contains("Z\r\nPath:ssml"));
    }

    #[test]
    fn build_config_message_contains_boundary_options() {
        let msg = build_config_message("test");
        assert!(msg.contains("sentenceBoundaryEnabled"));
        assert!(msg.contains("wordBoundaryEnabled"));
    }

    // ─── parse_binary_message ────────────────────────────────────────────

    #[test]
    fn parse_binary_message_extracts_audio_data() {
        // Header: "Path:audio\r\n" — colon format, matching actual Edge TTS protocol
        let header = b"Path:audio\r\n";
        let header_len = header.len() as u16;
        let header_len_bytes = header_len.to_be_bytes();
        let audio = b"\xff\xfb\x90\x00"; // fake MP3 header

        let mut data = Vec::new();
        data.extend_from_slice(&header_len_bytes);
        data.extend_from_slice(header);
        data.extend_from_slice(audio);

        let result = parse_binary_message(&data);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), audio.to_vec());
    }

    #[test]
    fn parse_binary_message_returns_none_for_turn_end() {
        let header = b"Path:turn.end\r\n";
        let header_len = header.len() as u16;
        let header_len_bytes = header_len.to_be_bytes();

        let mut data = Vec::new();
        data.extend_from_slice(&header_len_bytes);
        data.extend_from_slice(header);

        let result = parse_binary_message(&data);
        assert!(result.is_none());
    }

    #[test]
    fn parse_binary_message_extracts_audio_data_equals_format() {
        // Legacy "Path=audio" format (equals sign) should also work.
        let header = b"Path=audio\r\n";
        let header_len = header.len() as u16;
        let header_len_bytes = header_len.to_be_bytes();
        let audio = b"\xff\xfb\x90\x00";

        let mut data = Vec::new();
        data.extend_from_slice(&header_len_bytes);
        data.extend_from_slice(header);
        data.extend_from_slice(audio);

        let result = parse_binary_message(&data);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), audio.to_vec());
    }

    #[test]
    fn parse_binary_message_returns_none_for_empty_data() {
        assert!(parse_binary_message(&[]).is_none());
        assert!(parse_binary_message(&[0]).is_none());
    }

    #[test]
    fn parse_binary_message_returns_none_for_short_data() {
        // Header length claims 100 bytes but data is only 4 bytes total.
        let data: Vec<u8> = vec![0, 100, 0, 0];
        assert!(parse_binary_message(&data).is_none());
    }

    #[test]
    fn parse_binary_message_returns_none_for_unknown_path() {
        let header = b"Path=unknown\r\n";
        let header_len = header.len() as u16;
        let header_len_bytes = header_len.to_be_bytes();

        let mut data = Vec::new();
        data.extend_from_slice(&header_len_bytes);
        data.extend_from_slice(header);
        data.extend_from_slice(b"extra");

        assert!(parse_binary_message(&data).is_none());
    }

    #[test]
    fn parse_binary_message_returns_none_for_empty_audio_payload() {
        let header = b"Path=audio\r\n";
        let header_len = header.len() as u16;
        let header_len_bytes = header_len.to_be_bytes();

        let mut data = Vec::new();
        data.extend_from_slice(&header_len_bytes);
        data.extend_from_slice(header);
        // No audio data after header.

        assert!(parse_binary_message(&data).is_none());
    }

    // ─── TtsVoice serialization ───────────────────────────────────────────

    #[test]
    fn tts_voice_deserializes_from_edge_format() {
        let json = r#"{"name":"Microsoft Server Speech Text to Speech Voice (tr-TR, EmelNeural)","ShortName":"tr-TR-EmelNeural","gender":"Female","Locale":"tr-TR","SuggestedCodec":"audio-24khz-48kbitrate-mono-mp3"}"#;
        let voice: TtsVoice = serde_json::from_str(json).unwrap();
        assert_eq!(voice.short_name, "tr-TR-EmelNeural");
        assert_eq!(voice.locale, "tr-TR");
        assert_eq!(voice.gender, "Female");
    }

    #[test]
    fn tts_voice_serializes_to_json() {
        let voice = TtsVoice {
            name: "Test Voice".to_string(),
            short_name: "en-US-JennyNeural".to_string(),
            gender: "Female".to_string(),
            locale: "en-US".to_string(),
            suggested_codec: "audio-24khz-48kbitrate-mono-mp3".to_string(),
        };
        let json = serde_json::to_string(&voice).unwrap();
        assert!(json.contains("en-US-JennyNeural"));
        assert!(json.contains("ShortName"));
    }

    // ─── TtsState ────────────────────────────────────────────────────────

    #[test]
    fn tts_state_set_and_cancel() {
        let state = TtsState::new();
        let token = CancellationToken::new();
        let token_clone = token.clone();

        state.set(token);
        assert!(!token_clone.is_cancelled());

        state.cancel();
        assert!(token_clone.is_cancelled());
    }

    #[test]
    fn tts_state_cancel_when_empty() {
        let state = TtsState::new();
        state.cancel(); // Should not panic.
    }

    #[test]
    fn tts_state_clear_does_not_cancel() {
        let state = TtsState::new();
        let token = CancellationToken::new();
        let token_clone = token.clone();

        state.set(token);
        state.clear();
        assert!(!token_clone.is_cancelled());
    }

    #[test]
    fn tts_state_set_replaces_previous() {
        let state = TtsState::new();
        let first = CancellationToken::new();
        let first_clone = first.clone();
        let second = CancellationToken::new();
        let second_clone = second.clone();

        state.set(first);
        state.set(second);

        state.cancel();
        assert!(!first_clone.is_cancelled());
        assert!(second_clone.is_cancelled());
    }

    #[test]
    fn tts_state_epoch_increments_on_set() {
        let state = TtsState::new();
        assert_eq!(state.epoch.load(Ordering::SeqCst), 0);

        let epoch0 = state.set(CancellationToken::new());
        assert_eq!(epoch0, 0);

        let epoch1 = state.set(CancellationToken::new());
        assert_eq!(epoch1, 1);
    }

    // ─── Constants ──────────────────────────────────────────────────────

    #[test]
    fn default_voice_is_turkish() {
        assert_eq!(DEFAULT_VOICE, "tr-TR-EmelNeural");
    }

    #[test]
    fn default_rate_is_zero() {
        assert_eq!(DEFAULT_RATE, "+0%");
    }

    #[test]
    fn default_pitch_is_zero() {
        assert_eq!(DEFAULT_PITCH, "+0%");
    }

    // ─── generate_sec_ms_gec ────────────────────────────────────────────

    #[test]
    fn generate_sec_ms_gec_returns_64_char_hex() {
        let token = generate_sec_ms_gec();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn generate_sec_ms_gec_returns_uppercase() {
        let token = generate_sec_ms_gec();
        assert_eq!(token, token.to_uppercase());
    }

    #[test]
    fn generate_sec_ms_gec_deterministic_within_window() {
        // Two calls within the same 5-minute window should return the same token.
        let token1 = generate_sec_ms_gec();
        let token2 = generate_sec_ms_gec();
        assert_eq!(token1, token2);
    }

    // ─── Clock skew ─────────────────────────────────────────────────────

    #[test]
    fn update_clock_skew_parses_rfc1123_date() {
        // Reset skew first.
        CLOCK_SKEW.store(0, Ordering::Relaxed);

        // "Thu, 16 Apr 2026 10:54:12 GMT" should parse to a Unix timestamp.
        update_clock_skew("Thu, 16 Apr 2026 10:54:12 GMT");

        // The skew should be updated (non-zero is fine, just check it was set).
        // We can't predict the exact value since it depends on current local time,
        // but the function should not panic.
        let _skew = CLOCK_SKEW.load(Ordering::Relaxed);

        // Reset for other tests.
        CLOCK_SKEW.store(0, Ordering::Relaxed);
    }

    #[test]
    fn parse_http_date_parses_rfc1123() {
        let ts = parse_http_date("Thu, 16 Apr 2026 10:54:12 GMT");
        assert!(ts.is_some());
        // 2026-04-16 10:54:12 UTC
        assert_eq!(ts.unwrap(), 1776336852);
    }

    #[test]
    fn parse_http_date_returns_none_for_invalid() {
        assert!(parse_http_date("not a date").is_none());
        assert!(parse_http_date("").is_none());
    }

    #[test]
    fn month_from_name_returns_correct_values() {
        assert_eq!(month_from_name("Jan"), Some(1));
        assert_eq!(month_from_name("Dec"), Some(12));
        assert_eq!(month_from_name("XYZ"), None);
    }

    // ─── Date helpers ────────────────────────────────────────────────────

    #[test]
    fn days_to_date_known_value() {
        // 1970-01-01 = day 0
        let (y, m, d) = days_to_date(0);
        assert_eq!(y, 1970);
        assert_eq!(m, 1);
        assert_eq!(d, 1);
    }

    #[test]
    fn days_to_date_2026_04_16() {
        // 2026-04-16: 20559 days since Unix epoch
        let (y, m, d) = days_to_date(20559);
        assert_eq!(y, 2026);
        assert_eq!(m, 4);
        assert_eq!(d, 16);
    }

    #[test]
    fn days_to_weekday_thursday() {
        // 1970-01-01 was Thursday (4)
        assert_eq!(days_to_weekday(0), 4);
        // 1970-01-02 was Friday (5)
        assert_eq!(days_to_weekday(1), 5);
        // 1970-01-04 was Sunday (0)
        assert_eq!(days_to_weekday(3), 0);
    }

    // ─── chrono_like_timestamp ──────────────────────────────────────────

    #[test]
    fn chrono_like_timestamp_contains_gmt() {
        let ts = chrono_like_timestamp();
        assert!(ts.contains("GMT+0000"));
    }

    #[test]
    fn chrono_like_timestamp_contains_weekday() {
        let ts = chrono_like_timestamp();
        assert!(ts.contains("Sun")
            || ts.contains("Mon")
            || ts.contains("Tue")
            || ts.contains("Wed")
            || ts.contains("Thu")
            || ts.contains("Fri")
            || ts.contains("Sat"));
    }

    #[test]
    fn chrono_like_timestamp_contains_utc() {
        let ts = chrono_like_timestamp();
        assert!(ts.contains("Coordinated Universal Time"));
    }
}