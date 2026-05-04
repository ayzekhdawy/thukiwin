/*!
 * Screenshot capture.
 *
 * Exposes two Tauri commands on Windows, delegating to `windows_screenshot`:
 *
 * 1. `capture_screenshot_command`: interactive region select, returns base64.
 * 2. `capture_full_screen_command`: silently captures all screens, returns
 *    the absolute file path of the saved image.
 *
 * `temp_screenshot_path` and `encode_as_base64` are pure helpers extracted
 * from the command wrapper so they can be unit-tested without Tauri context.
 * The command wrappers themselves are excluded from coverage (thin I/O wrappers).
 */

use std::path::PathBuf;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

/// Returns a unique temp path for a single screenshot capture.
/// A new UUID is generated on every call, preventing collisions.
/// Uses the system temp directory for cross-platform compatibility.
pub fn temp_screenshot_path() -> PathBuf {
    std::env::temp_dir().join(format!("{}-thuki.png", uuid::Uuid::new_v4()))
}

/// Encodes raw bytes to a standard base64 string for IPC transfer.
pub fn encode_as_base64(bytes: &[u8]) -> String {
    BASE64.encode(bytes)
}

/// Converts a captured screenshot temp file into a base64-encoded PNG string.
///
/// Returns `Ok(None)` if the file was not created (user cancelled via Escape).
/// Returns `Ok(Some(base64))` on success, deleting the temp file after reading.
/// Returns `Err` if the file exists but cannot be read.
pub fn process_screenshot_result(path: &PathBuf) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None); // user cancelled: no file created
    }
    let bytes = std::fs::read(path).map_err(|e| format!("failed to read screenshot file: {e}"))?;
    let _ = std::fs::remove_file(path);
    Ok(Some(encode_as_base64(&bytes)))
}

// ─── Tauri commands (Windows) ─────────────────────────────────────────────────

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_screenshot_result_returns_none_when_file_missing() {
        let path = PathBuf::from(format!("/tmp/{}-missing.png", uuid::Uuid::new_v4()));
        assert_eq!(process_screenshot_result(&path).unwrap(), None);
    }

    #[test]
    fn process_screenshot_result_returns_base64_and_deletes_file() {
        let path = temp_screenshot_path();
        let content = b"fake png content";
        std::fs::write(&path, content).unwrap();
        let result = process_screenshot_result(&path).unwrap();
        assert_eq!(result, Some(encode_as_base64(content)));
        assert!(
            !path.exists(),
            "temp file should be deleted after processing"
        );
    }

    #[test]
    fn process_screenshot_result_returns_error_when_file_unreadable() {
        // A directory path exists but cannot be read as a file.
        let dir = std::env::temp_dir();
        let err = process_screenshot_result(&dir).unwrap_err();
        assert!(
            err.contains("failed to read screenshot file"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn temp_screenshot_path_is_in_temp_dir_and_ends_with_png() {
        let path = temp_screenshot_path();
        let s = path.to_str().unwrap();
        assert!(
            s.ends_with("-thuki.png"),
            "expected -thuki.png suffix, got: {s}"
        );
        assert!(
            path.parent().is_some(),
            "expected temp path to have a parent directory"
        );
    }

    #[test]
    fn temp_screenshot_path_generates_unique_paths() {
        let a = temp_screenshot_path();
        let b = temp_screenshot_path();
        assert_ne!(a, b, "two calls should return different paths");
    }

    #[test]
    fn encode_as_base64_roundtrip() {
        let original = b"hello screenshot world";
        let encoded = encode_as_base64(original);
        let decoded = BASE64.decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn encode_as_base64_empty_input() {
        assert_eq!(encode_as_base64(b""), "");
    }

}

// ─── Windows screenshot commands ────────────────────────────────────────────────

#[cfg(target_os = "windows")]
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg_attr(not(coverage), tauri::command)]
pub async fn capture_full_screen_command(app_handle: tauri::AppHandle) -> Result<String, String> {
    crate::windows_screenshot::capture_full_screen_command(app_handle).await
}

#[cfg(target_os = "windows")]
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg_attr(not(coverage), tauri::command)]
pub async fn capture_screenshot_command(
    app_handle: tauri::AppHandle,
) -> Result<Option<String>, String> {
    crate::windows_screenshot::capture_screenshot_command(app_handle).await
}
