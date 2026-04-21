//! Desktop automation via Win32 input simulation.
//!
//! Provides mouse (click, drag, scroll), keyboard (type, key combos),
//! and application launching for the agent mode loop.

use serde::{Deserialize, Serialize};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_TYPE, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, MOUSEINPUT, MOUSE_EVENT_FLAGS, VIRTUAL_KEY, VK_BACK, VK_CAPITAL,
    VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7,
    VK_F8, VK_F9, VK_F10, VK_F11, VK_F12, VK_HOME, VK_INSERT, VK_LEFT, VK_LCONTROL, VK_LMENU,
    VK_LSHIFT, VK_NEXT, VK_NUMLOCK, VK_PACKET, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_SNAPSHOT,
    VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;

// ─── Action types ───────────────────────────────────────────────────────────────

/// Actions the agent can request to control the desktop.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "params")]
pub enum AgentAction {
    Click { x: i32, y: i32 },
    DoubleClick { x: i32, y: i32 },
    RightClick { x: i32, y: i32 },
    Drag { start_x: i32, start_y: i32, end_x: i32, end_y: i32, duration_ms: u32 },
    TypeText { text: String },
    KeyPress { modifiers: Vec<String>, key: String },
    Scroll { direction: String, amount: i32 },
    Launch { target: String },
    Done { summary: String },
    Screenshot {},
}

/// Direction for scroll actions.
pub const SCROLL_UP: &str = "up";
pub const SCROLL_DOWN: &str = "down";

// ─── Mouse control ──────────────────────────────────────────────────────────────

const MOUSEEVENTF_LEFTDOWN: MOUSE_EVENT_FLAGS = MOUSE_EVENT_FLAGS(0x0002);
const MOUSEEVENTF_LEFTUP: MOUSE_EVENT_FLAGS = MOUSE_EVENT_FLAGS(0x0004);
const MOUSEEVENTF_RIGHTDOWN: MOUSE_EVENT_FLAGS = MOUSE_EVENT_FLAGS(0x0008);
const MOUSEEVENTF_RIGHTUP: MOUSE_EVENT_FLAGS = MOUSE_EVENT_FLAGS(0x0010);
const MOUSEEVENTF_WHEEL: MOUSE_EVENT_FLAGS = MOUSE_EVENT_FLAGS(0x0800);
const WHEEL_DELTA: i32 = 120;

/// Moves the cursor to absolute screen coordinates.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn set_cursor_pos(x: i32, y: i32) -> Result<(), String> {
    unsafe {
        SetCursorPos(x, y)
            .map_err(|e| format!("SetCursorPos failed: {e}"))
    }
}

/// Left-clicks at the given screen coordinates.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn execute_click(x: i32, y: i32) -> Result<(), String> {
    set_cursor_pos(x, y)?;
    let inputs = [
        mouse_input(MOUSEEVENTF_LEFTDOWN, 0, 0, 0),
        mouse_input(MOUSEEVENTF_LEFTUP, 0, 0, 0),
    ];
    send_inputs(&inputs);
    Ok(())
}

/// Double-clicks at the given screen coordinates.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn execute_double_click(x: i32, y: i32) -> Result<(), String> {
    set_cursor_pos(x, y)?;
    let inputs = [
        mouse_input(MOUSEEVENTF_LEFTDOWN, 0, 0, 0),
        mouse_input(MOUSEEVENTF_LEFTUP, 0, 0, 0),
        mouse_input(MOUSEEVENTF_LEFTDOWN, 0, 0, 0),
        mouse_input(MOUSEEVENTF_LEFTUP, 0, 0, 0),
    ];
    send_inputs(&inputs);
    // Brief pause between the two clicks for double-click recognition.
    std::thread::sleep(std::time::Duration::from_millis(100));
    Ok(())
}

/// Right-clicks at the given screen coordinates.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn execute_right_click(x: i32, y: i32) -> Result<(), String> {
    set_cursor_pos(x, y)?;
    let inputs = [
        mouse_input(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0),
        mouse_input(MOUSEEVENTF_RIGHTUP, 0, 0, 0),
    ];
    send_inputs(&inputs);
    Ok(())
}

/// Drags from (start_x, start_y) to (end_x, end_y) over the given duration.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn execute_drag(start_x: i32, start_y: i32, end_x: i32, end_y: i32, duration_ms: u32) -> Result<(), String> {
    set_cursor_pos(start_x, start_y)?;
    let down = mouse_input(MOUSEEVENTF_LEFTDOWN, 0, 0, 0);
    send_inputs(&[down]);

    let steps = 10.max(duration_ms / 16) as i32;
    let dx = (end_x - start_x) / steps;
    let dy = (end_y - start_y) / steps;
    let step_delay = duration_ms / steps.max(1) as u32;

    for i in 1..=steps {
        std::thread::sleep(std::time::Duration::from_millis(step_delay as u64));
        set_cursor_pos(start_x + dx * i, start_y + dy * i)?;
    }

    let up = mouse_input(MOUSEEVENTF_LEFTUP, 0, 0, 0);
    send_inputs(&[up]);
    Ok(())
}

/// Scrolls at the current cursor position.
/// Positive amount scrolls down, negative scrolls up.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn execute_scroll(direction: &str, amount: i32) -> Result<(), String> {
    let delta = match direction {
        SCROLL_UP => -amount * WHEEL_DELTA,
        SCROLL_DOWN => amount * WHEEL_DELTA,
        _ => return Err(format!("Unknown scroll direction: {direction}")),
    };
    let input = mouse_input(MOUSEEVENTF_WHEEL, 0, 0, delta);
    send_inputs(&[input]);
    Ok(())
}

// ─── Keyboard control ──────────────────────────────────────────────────────────

/// Named virtual key mapping for KeyPress actions.
fn named_vk(name: &str) -> Option<u16> {
    match name.to_lowercase().as_str() {
        "enter" | "return" => Some(VK_RETURN.0 as u16),
        "tab" => Some(VK_TAB.0 as u16),
        "escape" | "esc" => Some(VK_ESCAPE.0 as u16),
        "backspace" | "back" => Some(VK_BACK.0 as u16),
        "delete" | "del" => Some(VK_DELETE.0 as u16),
        "home" => Some(VK_HOME.0 as u16),
        "end" => Some(VK_END.0 as u16),
        "pageup" | "page_up" => Some(VK_PRIOR.0 as u16),
        "pagedown" | "page_down" => Some(VK_NEXT.0 as u16),
        "up" | "arrow_up" => Some(VK_UP.0 as u16),
        "down" | "arrow_down" => Some(VK_DOWN.0 as u16),
        "left" | "arrow_left" => Some(VK_LEFT.0 as u16),
        "right" | "arrow_right" => Some(VK_RIGHT.0 as u16),
        "space" => Some(VK_SPACE.0 as u16),
        "capslock" | "caps_lock" => Some(VK_CAPITAL.0 as u16),
        "numlock" | "num_lock" => Some(VK_NUMLOCK.0 as u16),
        "printscreen" | "print_screen" => Some(VK_SNAPSHOT.0 as u16),
        "insert" => Some(VK_INSERT.0 as u16),
        "f1" => Some(VK_F1.0 as u16),
        "f2" => Some(VK_F2.0 as u16),
        "f3" => Some(VK_F3.0 as u16),
        "f4" => Some(VK_F4.0 as u16),
        "f5" => Some(VK_F5.0 as u16),
        "f6" => Some(VK_F6.0 as u16),
        "f7" => Some(VK_F7.0 as u16),
        "f8" => Some(VK_F8.0 as u16),
        "f9" => Some(VK_F9.0 as u16),
        "f10" => Some(VK_F10.0 as u16),
        "f11" => Some(VK_F11.0 as u16),
        "f12" => Some(VK_F12.0 as u16),
        _ => None,
    }
}

/// Named modifier key mapping.
fn named_modifier_vk(name: &str) -> Option<u16> {
    match name.to_lowercase().as_str() {
        "ctrl" | "control" => Some(VK_LCONTROL.0 as u16),
        "alt" | "menu" => Some(VK_LMENU.0 as u16),
        "shift" => Some(VK_LSHIFT.0 as u16),
        _ => None,
    }
}

/// Types text character by character using VK_PACKET for Unicode support.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn execute_type_text(text: &str) -> Result<(), String> {
    for ch in text.chars() {
        let inputs = [
            INPUT {
                r#type: INPUT_TYPE(1),
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_PACKET,
                        wScan: ch as u16,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            INPUT {
                r#type: INPUT_TYPE(1),
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_PACKET,
                        wScan: ch as u16,
                        dwFlags: KEYEVENTF_KEYUP | KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
        ];
        send_inputs(&inputs);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Ok(())
}

/// Presses a key combination (e.g., ctrl+c, alt+tab).
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn execute_key_press(modifiers: &[String], key: &str) -> Result<(), String> {
    let vk = named_vk(key)
        .or_else(|| {
            // Single character key (e.g., "a", "1")
            let chars: Vec<char> = key.chars().collect();
            if chars.len() == 1 {
                Some(chars[0] as u16)
            } else {
                None
            }
        })
        .ok_or_else(|| format!("Unknown key: {key}"))?;

    let modifier_vks: Vec<u16> = modifiers
        .iter()
        .filter_map(|m| named_modifier_vk(m))
        .collect();

    // Check that all modifiers were recognized.
    if modifier_vks.len() != modifiers.len() {
        let unknown: Vec<&String> = modifiers
            .iter()
            .filter(|m| named_modifier_vk(m).is_none())
            .collect();
        return Err(format!("Unknown modifiers: {:?}", unknown));
    }

    let mut inputs: Vec<INPUT> = Vec::with_capacity(modifier_vks.len() * 2 + 2);

    // Press modifiers.
    for &mvk in &modifier_vks {
        inputs.push(key_input(mvk, false));
    }

    // Press main key.
    inputs.push(key_input(vk, false));

    // Release main key.
    inputs.push(key_input(vk, true));

    // Release modifiers in reverse order.
    for &mvk in modifier_vks.iter().rev() {
        inputs.push(key_input(mvk, true));
    }

    send_inputs(&inputs);
    Ok(())
}

/// Launches a program, file, or URL using ShellExecuteW.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn execute_launch(target: &str) -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(["/c", "start", "", target])
        .spawn()
        .map_err(|e| format!("Failed to launch '{target}': {e}"))?;
    Ok(())
}

// ─── Action dispatcher ──────────────────────────────────────────────────────────

/// Executes a single agent action.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn execute_action(action: &AgentAction) -> Result<(), String> {
    match action {
        AgentAction::Click { x, y } => execute_click(*x, *y),
        AgentAction::DoubleClick { x, y } => execute_double_click(*x, *y),
        AgentAction::RightClick { x, y } => execute_right_click(*x, *y),
        AgentAction::Drag { start_x, start_y, end_x, end_y, duration_ms } => {
            execute_drag(*start_x, *start_y, *end_x, *end_y, *duration_ms)
        }
        AgentAction::TypeText { text } => execute_type_text(text),
        AgentAction::KeyPress { modifiers, key } => execute_key_press(modifiers, key),
        AgentAction::Scroll { direction, amount } => execute_scroll(direction, *amount),
        AgentAction::Launch { target } => execute_launch(target),
        AgentAction::Done { .. } => Ok(()),
        AgentAction::Screenshot {} => Ok(()),
    }
}

// ─── Tauri command ────────────────────────────────────────────────────────────────

/// Tauri command wrapper for executing a single agent action from the frontend.
#[tauri::command]
pub fn execute_action_command(action: AgentAction) -> Result<String, String> {
    execute_action(&action).map(|_| "ok".to_string())
}

// ─── Action parsing ─────────────────────────────────────────────────────────────

/// Parses an agent action from a line of text.
/// Format: ACTION_NAME params (e.g., "CLICK 500 300", "TYPE Hello World", "KEY_PRESS ctrl+c")
pub fn parse_action_line(line: &str) -> Option<AgentAction> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    let action = parts[0].to_uppercase();
    let params = parts.get(1).unwrap_or(&"");

    match action.as_str() {
        "CLICK" => {
            let nums: Vec<i32> = params.split_whitespace().filter_map(|s| s.parse().ok()).collect();
            if nums.len() >= 2 {
                Some(AgentAction::Click { x: nums[0], y: nums[1] })
            } else {
                None
            }
        }
        "DOUBLE_CLICK" => {
            let nums: Vec<i32> = params.split_whitespace().filter_map(|s| s.parse().ok()).collect();
            if nums.len() >= 2 {
                Some(AgentAction::DoubleClick { x: nums[0], y: nums[1] })
            } else {
                None
            }
        }
        "RIGHT_CLICK" => {
            let nums: Vec<i32> = params.split_whitespace().filter_map(|s| s.parse().ok()).collect();
            if nums.len() >= 2 {
                Some(AgentAction::RightClick { x: nums[0], y: nums[1] })
            } else {
                None
            }
        }
        "DRAG" => {
            let nums: Vec<i32> = params.split_whitespace().filter_map(|s| s.parse().ok()).collect();
            if nums.len() >= 4 {
                Some(AgentAction::Drag {
                    start_x: nums[0],
                    start_y: nums[1],
                    end_x: nums[2],
                    end_y: nums[3],
                    duration_ms: nums.get(4).copied().unwrap_or(300) as u32,
                })
            } else {
                None
            }
        }
        "TYPE" => Some(AgentAction::TypeText { text: params.to_string() }),
        "KEY_PRESS" => {
            let key_parts: Vec<&str> = params.split('+').collect();
            if key_parts.is_empty() {
                return None;
            }
            let modifiers: Vec<String> = key_parts[..key_parts.len() - 1].iter().map(|s| s.to_string()).collect();
            let key = key_parts.last()?.to_string();
            Some(AgentAction::KeyPress { modifiers, key })
        }
        "SCROLL" => {
            let parts: Vec<&str> = params.split_whitespace().collect();
            if parts.len() >= 2 {
                let direction = parts[0].to_string();
                let amount = parts[1].parse().unwrap_or(3);
                Some(AgentAction::Scroll { direction, amount })
            } else {
                None
            }
        }
        "LAUNCH" => Some(AgentAction::Launch { target: params.to_string() }),
        "DONE" => Some(AgentAction::Done { summary: params.to_string() }),
        "SCREENSHOT" => Some(AgentAction::Screenshot {}),
        _ => None,
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────────

fn mouse_input(flags: MOUSE_EVENT_FLAGS, dx: i32, dy: i32, scroll_delta: i32) -> INPUT {
    INPUT {
        r#type: INPUT_TYPE(0),
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: scroll_delta as u32,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn key_input(vk: u16, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_TYPE(1),
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send_inputs(inputs: &[INPUT]) {
    unsafe {
        SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_click_action() {
        let action = parse_action_line("CLICK 500 300").unwrap();
        assert!(matches!(action, AgentAction::Click { x: 500, y: 300 }));
    }

    #[test]
    fn parse_double_click_action() {
        let action = parse_action_line("DOUBLE_CLICK 100 200").unwrap();
        assert!(matches!(action, AgentAction::DoubleClick { x: 100, y: 200 }));
    }

    #[test]
    fn parse_right_click_action() {
        let action = parse_action_line("RIGHT_CLICK 250 450").unwrap();
        assert!(matches!(action, AgentAction::RightClick { x: 250, y: 450 }));
    }

    #[test]
    fn parse_drag_action() {
        let action = parse_action_line("DRAG 100 200 300 400 500").unwrap();
        assert!(matches!(action, AgentAction::Drag { start_x: 100, start_y: 200, end_x: 300, end_y: 400, duration_ms: 500 }));
    }

    #[test]
    fn parse_drag_default_duration() {
        let action = parse_action_line("DRAG 10 20 30 40").unwrap();
        assert!(matches!(action, AgentAction::Drag { start_x: 10, start_y: 20, end_x: 30, end_y: 40, duration_ms: 300 }));
    }

    #[test]
    fn parse_type_action() {
        let action = parse_action_line("TYPE Hello World").unwrap();
        assert!(matches!(action, AgentAction::TypeText { ref text } if text == "Hello World"));
    }

    #[test]
    fn parse_key_press_with_modifiers() {
        let action = parse_action_line("KEY_PRESS ctrl+c").unwrap();
        assert!(matches!(action, AgentAction::KeyPress { ref modifiers, ref key } if modifiers.len() == 1 && key == "c"));
    }

    #[test]
    fn parse_key_press_multiple_modifiers() {
        let action = parse_action_line("KEY_PRESS ctrl+shift+s").unwrap();
        assert!(matches!(action, AgentAction::KeyPress { ref modifiers, ref key } if modifiers.len() == 2 && key == "s"));
    }

    #[test]
    fn parse_scroll_up() {
        let action = parse_action_line("SCROLL up 5").unwrap();
        assert!(matches!(action, AgentAction::Scroll { ref direction, amount } if direction == "up" && amount == 5));
    }

    #[test]
    fn parse_scroll_down() {
        let action = parse_action_line("SCROLL down 3").unwrap();
        assert!(matches!(action, AgentAction::Scroll { ref direction, amount } if direction == "down" && amount == 3));
    }

    #[test]
    fn parse_launch_action() {
        let action = parse_action_line("LAUNCH notepad.exe").unwrap();
        assert!(matches!(action, AgentAction::Launch { ref target } if target == "notepad.exe"));
    }

    #[test]
    fn parse_done_action() {
        let action = parse_action_line("DONE Task completed successfully").unwrap();
        assert!(matches!(action, AgentAction::Done { ref summary } if summary == "Task completed successfully"));
    }

    #[test]
    fn parse_screenshot_action() {
        let action = parse_action_line("SCREENSHOT").unwrap();
        assert!(matches!(action, AgentAction::Screenshot {}));
    }

    #[test]
    fn parse_unknown_action_returns_none() {
        assert!(parse_action_line("FLY 100 200").is_none());
    }

    #[test]
    fn parse_empty_line_returns_none() {
        assert!(parse_action_line("").is_none());
        assert!(parse_action_line("   ").is_none());
    }

    #[test]
    fn parse_click_with_whitespace() {
        let action = parse_action_line("  CLICK  500  300  ").unwrap();
        assert!(matches!(action, AgentAction::Click { x: 500, y: 300 }));
    }

    #[test]
    fn parse_case_insensitive_action() {
        let action = parse_action_line("click 100 200").unwrap();
        assert!(matches!(action, AgentAction::Click { x: 100, y: 200 }));
    }

    #[test]
    fn named_vk_maps_common_keys() {
        assert_eq!(named_vk("enter"), Some(VK_RETURN.0 as u16));
        assert_eq!(named_vk("tab"), Some(VK_TAB.0 as u16));
        assert_eq!(named_vk("escape"), Some(VK_ESCAPE.0 as u16));
        assert_eq!(named_vk("backspace"), Some(VK_BACK.0 as u16));
        assert_eq!(named_vk("delete"), Some(VK_DELETE.0 as u16));
        assert_eq!(named_vk("f1"), Some(VK_F1.0 as u16));
        assert_eq!(named_vk("space"), Some(VK_SPACE.0 as u16));
    }

    #[test]
    fn named_vk_unknown_returns_none() {
        assert!(named_vk("unknown").is_none());
        assert!(named_vk("a").is_none());
    }

    #[test]
    fn named_modifier_vk_maps_modifiers() {
        assert_eq!(named_modifier_vk("ctrl"), Some(VK_LCONTROL.0 as u16));
        assert_eq!(named_modifier_vk("alt"), Some(VK_LMENU.0 as u16));
        assert_eq!(named_modifier_vk("shift"), Some(VK_LSHIFT.0 as u16));
    }

    #[test]
    fn named_modifier_vk_unknown_returns_none() {
        assert!(named_modifier_vk("super").is_none());
    }

    #[test]
    fn action_serialization_roundtrip() {
        let actions = vec![
            AgentAction::Click { x: 100, y: 200 },
            AgentAction::TypeText { text: "Hello".to_string() },
            AgentAction::KeyPress { modifiers: vec!["ctrl".to_string()], key: "c".to_string() },
            AgentAction::Done { summary: "Done".to_string() },
        ];
        for action in actions {
            let json = serde_json::to_string(&action).unwrap();
            let deserialized: AgentAction = serde_json::from_str(&json).unwrap();
            assert_eq!(json, serde_json::to_string(&deserialized).unwrap());
        }
    }

    #[test]
    fn scroll_constants() {
        assert_eq!(SCROLL_UP, "up");
        assert_eq!(SCROLL_DOWN, "down");
        assert_eq!(WHEEL_DELTA, 120);
    }
}