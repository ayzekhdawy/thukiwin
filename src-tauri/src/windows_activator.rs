//! Windows activation listener, context capture, and permissions.
//!
//! This module combines the Windows-specific functionality needed for ThukiWin:
//! - Double-tap Ctrl hotkey detection via SetWindowsHookExW
//! - Context capture (clipboard fallback for selected text)
//! - Permission stubs (Windows has no TCC equivalent)

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use windows::core::BOOL;
use windows::Win32::Foundation::{LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_TYPE, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_C,
    VK_CONTROL,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetCursorPos, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL,
};

use crate::context::{ActivationContext, ScreenRect};

use windows::Win32::UI::WindowsAndMessaging::{
    GetGUIThreadInfo, GetWindowRect, GUITHREADINFO,
};

// ─── Constants ──────────────────────────────────────────────────────────────────

const ACTIVATION_WINDOW: Duration = Duration::from_millis(400);
const ACTIVATION_COOLDOWN: Duration = Duration::from_millis(600);
const VK_LCONTROL: i32 = 0xA2;
const VK_RCONTROL: i32 = 0xA3;

// ─── Global Hook State ─────────────────────────────────────────────────────────

static GLOBAL_ACTIVATION_STATE: LazyLock<Mutex<ActivationState>> = LazyLock::new(|| {
    Mutex::new(ActivationState {
        last_trigger: None,
        is_pressed: false,
        last_activation: None,
    })
});

type ActivationCallback = Arc<dyn Fn() + Send + Sync>;

#[allow(clippy::type_complexity)]
static GLOBAL_ON_ACTIVATION: LazyLock<Mutex<Option<ActivationCallback>>> =
    LazyLock::new(|| Mutex::new(None));

static GLOBAL_HOOK_ACTIVE: AtomicBool = AtomicBool::new(false);

// ─── Activation Logic ──────────────────────────────────────────────────────────

struct ActivationState {
    last_trigger: Option<Instant>,
    is_pressed: bool,
    last_activation: Option<Instant>,
}

fn evaluate_activation(state: &mut ActivationState, is_press: bool) -> bool {
    if is_press && !state.is_pressed {
        state.is_pressed = true;
        let now = Instant::now();

        if let Some(last_act) = state.last_activation {
            if now.duration_since(last_act) < ACTIVATION_COOLDOWN {
                return false;
            }
        }

        if let Some(last) = state.last_trigger {
            if now.duration_since(last) < ACTIVATION_WINDOW {
                state.last_trigger = None;
                state.last_activation = Some(now);
                return true;
            }
        }
        state.last_trigger = Some(now);
    } else if !is_press {
        state.is_pressed = false;
    }

    false
}

// ─── Public Interface ───────────────────────────────────────────────────────────

pub struct OverlayActivator {
    is_active: Arc<AtomicBool>,
}

impl OverlayActivator {
    pub fn new() -> Self {
        Self {
            is_active: Arc::new(AtomicBool::new(false)),
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn start<F>(&self, on_activation: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        if self.is_active.load(Ordering::SeqCst) {
            return;
        }
        self.is_active.store(true, Ordering::SeqCst);

        {
            let mut cb = GLOBAL_ON_ACTIVATION.lock().unwrap();
            *cb = Some(Arc::new(on_activation));
        }

        let is_active = self.is_active.clone();

        std::thread::spawn(move || {
            run_hook_loop(is_active);
        });
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn run_hook_loop(is_active: Arc<AtomicBool>) {
    GLOBAL_HOOK_ACTIVE.store(true, Ordering::SeqCst);

    let hook = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_callback), None, 0) };

    match hook {
        Ok(hook_handle) => {
            eprintln!("thuki: [activator] keyboard hook installed — listening for double-tap Ctrl");

            let mut msg = MSG::default();
            while is_active.load(Ordering::SeqCst) {
                let ret = unsafe { GetMessageW(&mut msg, None, 0, 0) };
                if ret == BOOL(0) || ret == BOOL(-1) {
                    break;
                }
            }

            let _ = unsafe { UnhookWindowsHookEx(hook_handle) };
        }
        Err(_) => {
            eprintln!("thuki: [activator] failed to install keyboard hook");
        }
    }

    GLOBAL_HOOK_ACTIVE.store(false, Ordering::SeqCst);

    {
        let mut cb = GLOBAL_ON_ACTIVATION.lock().unwrap();
        *cb = None;
    }

    eprintln!("thuki: [activator] keyboard hook removed");
}

unsafe extern "system" fn keyboard_hook_callback(
    code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if code < 0 {
        return unsafe { CallNextHookEx(None, code, w_param, l_param) };
    }

    let kb_struct = unsafe { &*(l_param.0 as *const KBDLLHOOKSTRUCT) };

    // Ignore synthetic key events (e.g. from SendInput in clipboard_fallback)
    // to prevent simulated Ctrl presses from interfering with activation detection.
    // LLKHF_INJECTED = 0x0010 — set for events injected by SendInput or similar.
    let injected = kb_struct.flags.0 & 0x10 != 0;
    if injected {
        return unsafe { CallNextHookEx(None, code, w_param, l_param) };
    }

    let wparam_val = w_param.0 as u32;
    let is_press = wparam_val == 0x0100 || wparam_val == 0x0104;

    let vk_code = kb_struct.vkCode as i32;

    if vk_code == VK_LCONTROL || vk_code == VK_RCONTROL {
        let is_release = wparam_val == 0x0101 || wparam_val == 0x0105;
        let key_down = is_press && !is_release;

        let mut s = GLOBAL_ACTIVATION_STATE.lock().unwrap();
        if evaluate_activation(&mut s, key_down) {
            let cb = GLOBAL_ON_ACTIVATION.lock().unwrap();
            if let Some(ref callback) = *cb {
                callback();
            }
        }
    }

    unsafe { CallNextHookEx(None, code, w_param, l_param) }
}

// ─── Context Capture ────────────────────────────────────────────────────────────

fn current_mouse_position() -> (f64, f64) {
    let mut point = POINT { x: 0, y: 0 };
    unsafe {
        let _ = GetCursorPos(&mut point);
    }
    (point.x as f64, point.y as f64)
}

fn simulate_ctrl_c() {
    let inputs: [INPUT; 4] = [
        INPUT {
            r#type: INPUT_TYPE(1),
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_CONTROL,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        INPUT {
            r#type: INPUT_TYPE(1),
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_C,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        INPUT {
            r#type: INPUT_TYPE(1),
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_C,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        INPUT {
            r#type: INPUT_TYPE(1),
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_CONTROL,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];
    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

fn clipboard_text() -> String {
    unsafe {
        if OpenClipboard(None).is_err() {
            return String::new();
        }
        let result = if let Ok(handle) = GetClipboardData(CF_UNICODETEXT.0 as u32) {
            let ptr = handle.0 as *const u16;
            if ptr.is_null() {
                String::new()
            } else {
                let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
                let slice = std::slice::from_raw_parts(ptr, len);
                String::from_utf16_lossy(slice)
            }
        } else {
            String::new()
        };
        let _ = CloseClipboard();
        result
    }
}

fn write_clipboard(text: &str) {
    unsafe {
        if OpenClipboard(None).is_err() {
            return;
        }
        let _ = EmptyClipboard();
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let size = wide.len() * std::mem::size_of::<u16>();
        if let Ok(h_mem) = GlobalAlloc(GMEM_MOVEABLE, size) {
            let ptr = GlobalLock(h_mem);
            if !ptr.is_null() {
                std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr as *mut u16, wide.len());
                let _ = GlobalUnlock(h_mem);
                let _ = SetClipboardData(
                    CF_UNICODETEXT.0 as u32,
                    Some(windows::Win32::Foundation::HANDLE(h_mem.0)),
                );
            }
        }
        let _ = CloseClipboard();
    }
}

fn clipboard_fallback() -> Option<String> {
    let before = clipboard_text();
    simulate_ctrl_c();

    let mut after = before.clone();
    for delay_ms in [10, 20, 40, 80] {
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        after = clipboard_text();
        if after != before {
            break;
        }
    }

    if after != before {
        write_clipboard(&before);
    }

    let trimmed = after.trim().to_string();
    if after != before && !trimmed.is_empty() {
        Some(trimmed)
    } else {
        None
    }
}

/// Attempts to get the bounding rectangle of the focused UI element.
/// Uses GetGUIThreadInfo to find the focused control and GetWindowRect
/// as a fallback. This provides approximate bounds for window positioning.
fn get_focused_element_bounds() -> Option<ScreenRect> {
    unsafe {
        let mut gui_info = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };

        // Get info about the foreground thread's UI state.
        if GetGUIThreadInfo(0, &mut gui_info).is_err() {
            return None;
        }

        // If we have a focused control handle, get its screen rectangle.
        let hwnd = if !gui_info.hwndFocus.is_invalid() && !gui_info.hwndFocus.0.is_null() {
            gui_info.hwndFocus
        } else if !gui_info.hwndActive.is_invalid() && !gui_info.hwndActive.0.is_null() {
            gui_info.hwndActive
        } else {
            return None;
        };

        let mut rect = std::mem::zeroed();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            Some(ScreenRect {
                x: rect.left as f64,
                y: rect.top as f64,
                width: (rect.right - rect.left) as f64,
                height: (rect.bottom - rect.top) as f64,
            })
        } else {
            None
        }
    }
}

pub fn capture() -> ActivationContext {
    let mouse = current_mouse_position();
    let text = clipboard_fallback();
    let bounds = get_focused_element_bounds();
    ActivationContext {
        selected_text: text,
        bounds,
        mouse_position: Some(mouse),
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_activation_sequence() {
        let mut state = ActivationState {
            last_trigger: None,
            is_pressed: false,
            last_activation: None,
        };
        assert!(!evaluate_activation(&mut state, true));
        evaluate_activation(&mut state, false);
        assert!(evaluate_activation(&mut state, true));
    }

    #[test]
    fn rejects_stale_sequence() {
        let mut state = ActivationState {
            last_trigger: None,
            is_pressed: false,
            last_activation: None,
        };
        evaluate_activation(&mut state, true);
        evaluate_activation(&mut state, false);
        state.last_trigger = Some(Instant::now() - Duration::from_millis(500));
        assert!(!evaluate_activation(&mut state, true));
    }

    #[test]
    fn boundary_timing_at_exactly_400ms_is_rejected() {
        let mut state = ActivationState {
            last_trigger: Some(Instant::now() - Duration::from_millis(400)),
            is_pressed: false,
            last_activation: None,
        };
        assert!(!evaluate_activation(&mut state, true));
    }

    #[test]
    fn release_without_press_does_nothing() {
        let mut state = ActivationState {
            last_trigger: None,
            is_pressed: false,
            last_activation: None,
        };
        assert!(!evaluate_activation(&mut state, false));
        assert!(state.last_trigger.is_none());
    }
}
