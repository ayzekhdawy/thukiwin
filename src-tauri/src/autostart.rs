//! Windows auto-start management via Task Scheduler.
//!
//! Uses `schtasks` commands to create/delete a logon trigger task so ThukiWin
//! starts automatically when the user logs in.

/// The name of the scheduled task for auto-start detection.
const TASK_NAME: &str = "ThukiWin";

/// Checks whether the auto-start task is registered in Task Scheduler.
pub fn is_auto_start_enabled() -> Result<bool, String> {
    let output = std::process::Command::new("schtasks")
        .args(["/Query", "/TN", TASK_NAME])
        .output()
        .map_err(|e| format!("Failed to query Task Scheduler: {e}"))?;

    Ok(output.status.success())
}

/// Creates a scheduled task that runs ThukiWin on user logon.
pub fn enable_auto_start() -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {e}"))?;
    let exe_str = exe_path
        .to_str()
        .ok_or_else(|| "Executable path is not valid UTF-8".to_string())?;

    // Delete existing task first (in case it's stale or the path changed).
    let _ = std::process::Command::new("schtasks")
        .args(["/Delete", "/TN", TASK_NAME, "/F"])
        .output();

    let output = std::process::Command::new("schtasks")
        .args([
            "/Create",
            "/TN",
            TASK_NAME,
            "/TR",
            exe_str,
            "/SC",
            "ONLOGON",
            "/RL",
            "LIMITED",
            "/F",
        ])
        .output()
        .map_err(|e| format!("Failed to create scheduled task: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("schtasks /Create failed: {stderr}"))
    }
}

/// Removes the auto-start scheduled task.
pub fn disable_auto_start() -> Result<(), String> {
    let output = std::process::Command::new("schtasks")
        .args(["/Delete", "/TN", TASK_NAME, "/F"])
        .output()
        .map_err(|e| format!("Failed to delete scheduled task: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        // Task doesn't exist is not an error.
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("could not be found") || stderr.contains("The system cannot find") {
            Ok(())
        } else {
            Err(format!("schtasks /Delete failed: {stderr}"))
        }
    }
}

// ─── Tauri commands ──────────────────────────────────────────────────────────

#[cfg_attr(coverage_nightly, coverage(off))]
#[tauri::command]
pub fn is_auto_start_enabled_command() -> Result<bool, String> {
    is_auto_start_enabled()
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[tauri::command]
pub fn enable_auto_start_command() -> Result<(), String> {
    enable_auto_start()
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[tauri::command]
pub fn disable_auto_start_command() -> Result<(), String> {
    disable_auto_start()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_name_constant() {
        assert_eq!(TASK_NAME, "ThukiWin");
    }

    #[test]
    fn is_auto_start_enabled_returns_result() {
        // On a dev machine the task may or may not exist; we just verify it doesn't panic.
        let _ = is_auto_start_enabled();
    }
}