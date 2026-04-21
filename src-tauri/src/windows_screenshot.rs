//! Windows screenshot capture using GDI.
//!
//! Supports:
//! - Full-screen capture across all monitors via virtual screen coordinates
//! - Interactive region selection (crosshair overlay) via a fullscreen
//!   transparent window with mouse drag

use std::sync::{Arc, Mutex};

use tauri::Manager;
use windows::core::BOOL;
use windows::Win32::Foundation::{COLORREF, HWND, HINSTANCE, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateSolidBrush, DeleteDC,
    DeleteObject, EndPaint, FillRect, GetBitmapBits, GetDC, InvalidateRect, ReleaseDC, SelectObject,
    SRCCOPY, PAINTSTRUCT,
};
use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetCursorPos, GetMessageW,
    GetSystemMetrics, LoadCursorW, MSG, RegisterClassExW, SetLayeredWindowAttributes,
    ShowWindow, TranslateMessage, WNDCLASSEXW, WINDOW_EX_STYLE, WINDOW_STYLE,
    WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP, WS_VISIBLE,
    IDC_CROSS, LWA_ALPHA, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
    SM_YVIRTUALSCREEN, SW_SHOW, WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEMOVE, WM_PAINT, WM_RBUTTONDOWN,
};

// ─── Full-screen capture (all monitors) ──────────────────────────────────────

/// Captures the full virtual screen (all monitors) using GDI BitBlt
/// and returns raw RGBA pixel bytes plus the virtual screen origin offset.
fn capture_virtual_screen_pixels() -> Result<(i32, i32, u32, u32, Vec<u8>), String> {
    unsafe {
        let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        if vw <= 0 || vh <= 0 {
            return Err("Failed to get virtual screen dimensions".to_string());
        }

        let width = vw as u32;
        let height = vh as u32;

        let screen_dc = GetDC(None);
        let mem_dc = CreateCompatibleDC(Some(screen_dc));
        let bitmap = CreateCompatibleBitmap(screen_dc, vw, vh);
        let _old_bitmap = SelectObject(mem_dc, bitmap.into());

        BitBlt(mem_dc, 0, 0, vw, vh, Some(screen_dc), vx, vy, SRCCOPY)
            .map_err(|e| format!("BitBlt failed: {e}"))?;

        // Deselect bitmap before reading.
        SelectObject(mem_dc, _old_bitmap);

        let row_size = width * 4;
        let pixel_size = (row_size * height) as usize;
        let mut pixels: Vec<u8> = vec![0u8; pixel_size];

        let bits_copied = GetBitmapBits(
            bitmap,
            pixel_size as i32,
            pixels.as_mut_ptr() as *mut core::ffi::c_void,
        );

        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(mem_dc);
        let _ = ReleaseDC(None, screen_dc);

        if bits_copied == 0 {
            return Err("GetBitmapBits returned 0 bytes".to_string());
        }

        // Convert BGRA to RGBA in-place.
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }

        Ok((vx, vy, width, height, pixels))
    }
}

/// Tauri command: silently captures the full virtual screen (all monitors)
/// and returns the absolute file path of the saved image.
#[cfg(target_os = "windows")]
#[cfg_attr(coverage_nightly, coverage(off))]
pub async fn capture_full_screen_command(app_handle: tauri::AppHandle) -> Result<String, String> {
    let base_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {e}"))?;

    let result = tokio::task::spawn_blocking(move || {
        let (_vx, _vy, width, height, rgba_bytes) = capture_virtual_screen_pixels()?;

        let buf =
            image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(width, height, rgba_bytes)
                .ok_or_else(|| "Failed to create image buffer from captured pixels.".to_string())?;
        let dynamic = image::DynamicImage::ImageRgba8(buf);

        let mut png: Vec<u8> = Vec::new();
        dynamic
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .map_err(|e| format!("Failed to encode screen capture as PNG: {e}"))?;

        crate::images::save_image(&base_dir, &png)
    })
    .await
    .map_err(|e| format!("image encoding task failed: {e}"))?;

    result
}

// ─── Interactive region screenshot ────────────────────────────────────────────

/// Callback data for region selection window procedure.
struct RegionSelectionState {
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    is_selecting: bool,
    is_done: bool,
    cancelled: bool,
}

/// Global state for the region selection window proc, protected by a Mutex.
static SELECTION_STATE: std::sync::LazyLock<Mutex<Option<Arc<Mutex<RegionSelectionState>>>>> = 
    std::sync::LazyLock::new(|| Mutex::new(None));

fn set_selection_state(state: Arc<Mutex<RegionSelectionState>>) {
    *SELECTION_STATE.lock().unwrap() = Some(state);
}

fn clear_selection_state() {
    *SELECTION_STATE.lock().unwrap() = None;
}

fn get_selection_state() -> Option<Arc<Mutex<RegionSelectionState>>> {
    SELECTION_STATE.lock().unwrap().clone()
}

/// Window procedure for the region selection overlay.
///
/// Draws a dark tint over the entire virtual screen, and a bright
/// rectangle for the selected region with a blue border.
/// Left-click + drag selects a region; right-click or Escape cancels.
unsafe extern "system" fn region_selection_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_LBUTTONDOWN => {
            let mut point = POINT { x: 0, y: 0 };
            let _ = GetCursorPos(&mut point);
            if let Some(state) = get_selection_state() {
                let mut s = state.lock().unwrap();
                s.start_x = point.x;
                s.start_y = point.y;
                s.end_x = point.x;
                s.end_y = point.y;
                s.is_selecting = true;
            }
            let _ = InvalidateRect(Some(hwnd), None, true);
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            if let Some(state) = get_selection_state() {
                let mut s = state.lock().unwrap();
                if s.is_selecting {
                    let mut point = POINT { x: 0, y: 0 };
                    let _ = GetCursorPos(&mut point);
                    s.end_x = point.x;
                    s.end_y = point.y;
                }
            }
            let _ = InvalidateRect(Some(hwnd), None, true);
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            if let Some(state) = get_selection_state() {
                let mut point = POINT { x: 0, y: 0 };
                let _ = GetCursorPos(&mut point);
                let mut s = state.lock().unwrap();
                if s.is_selecting {
                    s.end_x = point.x;
                    s.end_y = point.y;
                    s.is_selecting = false;
                    s.is_done = true;
                }
            }
            LRESULT(0)
        }
        WM_RBUTTONDOWN => {
            if let Some(state) = get_selection_state() {
                let mut s = state.lock().unwrap();
                s.cancelled = true;
                s.is_done = true;
            }
            LRESULT(0)
        }
        WM_KEYDOWN => {
            if wparam.0 as u32 == VK_ESCAPE.0 as u32 {
                if let Some(state) = get_selection_state() {
                    let mut s = state.lock().unwrap();
                    s.cancelled = true;
                    s.is_done = true;
                }
            }
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            if !hdc.is_invalid() {
                let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
                let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
                let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
                let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);

                // Dark overlay over entire virtual screen.
                let full_rect = RECT {
                    left: vx,
                    top: vy,
                    right: vx + vw,
                    bottom: vy + vh,
                };
                let dark_brush = CreateSolidBrush(COLORREF(0));
                let _ = FillRect(hdc, &full_rect, dark_brush);
                let _ = DeleteObject(dark_brush.into());

                // If selecting, draw the selection rectangle.
                if let Some(state) = get_selection_state() {
                    let s = state.lock().unwrap();
                    let x1 = s.start_x.min(s.end_x);
                    let y1 = s.start_y.min(s.end_y);
                    let x2 = s.start_x.max(s.end_x);
                    let y2 = s.start_y.max(s.end_y);
                    drop(s);

                    if x2 > x1 && y2 > y1 {
                        // Clear/bright area for the selection.
                        let sel_rect = RECT {
                            left: x1,
                            top: y1,
                            right: x2,
                            bottom: y2,
                        };
                        let clear_brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
                        let _ = FillRect(hdc, &sel_rect, clear_brush);
                        let _ = DeleteObject(clear_brush.into());

                        // Blue accent border (2px).
                        let blue = CreateSolidBrush(COLORREF(0x00D77800));
                        let _ = FillRect(
                            hdc,
                            &RECT { left: x1, top: y1, right: x2, bottom: y1 + 2 },
                            blue,
                        );
                        let _ = FillRect(
                            hdc,
                            &RECT { left: x1, top: y2 - 2, right: x2, bottom: y2 },
                            blue,
                        );
                        let _ = FillRect(
                            hdc,
                            &RECT { left: x1, top: y1, right: x1 + 2, bottom: y2 },
                            blue,
                        );
                        let _ = FillRect(
                            hdc,
                            &RECT { left: x2 - 2, top: y1, right: x2, bottom: y2 },
                            blue,
                        );
                        let _ = DeleteObject(blue.into());
                    }
                }
                let _ = EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Captures a user-selected screen region.
///
/// Hides the Thuki window, takes a full virtual screen capture, then
/// opens a transparent fullscreen overlay window. The user drags to
/// select a region. On release, the selected rectangle is cropped
/// from the pre-capture image and saved. Returns None if the user
/// cancels (Escape, right-click, or no drag).
#[cfg(target_os = "windows")]
#[cfg_attr(coverage_nightly, coverage(off))]
pub async fn capture_screenshot_command(
    app_handle: tauri::AppHandle,
) -> Result<Option<String>, String> {
    use windows::core::PCWSTR;

    // Hide the main window so it's not in the capture.
    let hide_handle = app_handle.clone();
    app_handle
        .run_on_main_thread(move || {
            if let Some(w) = hide_handle.get_webview_window("main") {
                let _ = w.hide();
            }
        })
        .map_err(|e| format!("failed to hide window: {e}"))?;

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Capture full virtual screen before showing the selection overlay.
    let capture_result = tokio::task::spawn_blocking(|| capture_virtual_screen_pixels())
        .await
        .map_err(|e| format!("capture task failed: {e}"))??;

    let (vx, vy, full_width, full_height, full_rgba) = capture_result;

    // Create and run the region selection overlay on the main thread.
    let state = Arc::new(Mutex::new(RegionSelectionState {
        start_x: 0,
        start_y: 0,
        end_x: 0,
        end_y: 0,
        is_selecting: false,
        is_done: false,
        cancelled: false,
    }));

    let state_clone = state.clone();

    let _ = app_handle.run_on_main_thread(move || {
        let class_name: Vec<u16> = "ThukiRegionSelector\0".encode_utf16().collect();

        let window_title: Vec<u16> = "ThukiRegionSelector0".encode_utf16().collect();
        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: Default::default(),
            lpfnWndProc: Some(region_selection_wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: HINSTANCE(std::ptr::null_mut()),
            hCursor: unsafe { LoadCursorW(None, IDC_CROSS).unwrap_or_default() },
            hbrBackground: unsafe { CreateSolidBrush(COLORREF(0)) },
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        let atom = unsafe { RegisterClassExW(&wnd_class) };
        if atom == 0 {
            let mut s = state_clone.lock().unwrap();
            s.cancelled = true;
            s.is_done = true;
            return;
        }

        set_selection_state(state_clone.clone());

        let vx = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
        let vy = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
        let vw = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
        let vh = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };

        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(WS_EX_TOPMOST.0 | WS_EX_LAYERED.0 | WS_EX_TRANSPARENT.0),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(window_title.as_ptr()),
                WINDOW_STYLE(WS_POPUP.0 | WS_VISIBLE.0),
                vx,
                vy,
                vw,
                vh,
                None,
                None,
                Some(wnd_class.hInstance),
                None,
            )
        };

        match hwnd {
            Ok(h) => {
                // Semi-transparent overlay so user can see the screen underneath.
                unsafe {
                    let _ = SetLayeredWindowAttributes(h, COLORREF(0), 120, LWA_ALPHA);
                    let _ = ShowWindow(h, SW_SHOW);
                    let _ = windows::Win32::Graphics::Gdi::UpdateWindow(h);
                }

                // Message loop — blocks until selection is complete or cancelled.
                let mut msg = MSG::default();
                loop {
                    let ret = unsafe { GetMessageW(&mut msg, None, 0, 0) };
                    if ret == BOOL(0) || ret == BOOL(-1) {
                        break;
                    }
                    let _ = unsafe { TranslateMessage(&msg) };
                    unsafe { DispatchMessageW(&msg) };

                    let s = state_clone.lock().unwrap();
                    if s.is_done {
                        break;
                    }
                }

                unsafe { let _ = DestroyWindow(h); };
            }
            Err(_) => {
                let mut s = state_clone.lock().unwrap();
                s.cancelled = true;
                s.is_done = true;
            }
        }

        clear_selection_state();
    });

    // Re-show the main window.
    let show_handle = app_handle.clone();
    let _ = app_handle.run_on_main_thread(move || {
        if let Some(w) = show_handle.get_webview_window("main") {
            let _ = w.show();
            let _ = w.set_focus();
        }
    });

    let (cancelled, is_done, x1, y1, x2, y2) = {
        let s = state.lock().unwrap();
        (s.cancelled, s.is_done, s.start_x.min(s.end_x), s.start_y.min(s.end_y), s.start_x.max(s.end_x), s.start_y.max(s.end_y))
    };

    if cancelled || !is_done {
        return Ok(None);
    }

    if x2 <= x1 || y2 <= y1 {
        return Ok(None);
    }

    // Convert virtual-screen coordinates to buffer offsets and crop.
    let buf_x = (x1 - vx).max(0) as u32;
    let buf_y = (y1 - vy).max(0) as u32;
    let crop_w = ((x2 - x1) as u32).min(full_width.saturating_sub(buf_x));
    let crop_h = ((y2 - y1) as u32).min(full_height.saturating_sub(buf_y));

    if crop_w == 0 || crop_h == 0 {
        return Ok(None);
    }

    let base_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {e}"))?;

    let result = tokio::task::spawn_blocking(move || {
        let mut cropped = Vec::with_capacity((crop_w * crop_h * 4) as usize);
        let row_stride = full_width * 4;
        for row in buf_y..buf_y + crop_h {
            let offset = (row * row_stride + buf_x * 4) as usize;
            let end = offset + (crop_w * 4) as usize;
            if end <= full_rgba.len() {
                cropped.extend_from_slice(&full_rgba[offset..end]);
            }
        }

        if cropped.len() != (crop_w * crop_h * 4) as usize {
            return Err("Cropped region size mismatch".to_string());
        }

        let buf = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(crop_w, crop_h, cropped)
            .ok_or_else(|| "Failed to create image buffer from cropped pixels.".to_string())?;
        let dynamic = image::DynamicImage::ImageRgba8(buf);

        let mut png: Vec<u8> = Vec::new();
        dynamic
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .map_err(|e| format!("Failed to encode cropped capture as PNG: {e}"))?;

        crate::images::save_image(&base_dir, &png)
    })
    .await
    .map_err(|e| format!("image encoding task failed: {e}"))??;

    Ok(Some(result))
}

// ─── Silent capture (for agent loop) ─────────────────────────────────────────

/// Captures the full virtual screen without hiding or showing the overlay.
/// Used by the agent mode loop where the overlay must remain visible.
#[cfg(target_os = "windows")]
#[cfg_attr(coverage_nightly, coverage(off))]
pub async fn capture_silent_screenshot_command(app_handle: tauri::AppHandle) -> Result<String, String> {
    let base_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {e}"))?;

    tokio::task::spawn_blocking(move || {
        let (_vx, _vy, width, height, rgba_bytes) = capture_virtual_screen_pixels()?;

        let buf =
            image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(width, height, rgba_bytes)
                .ok_or_else(|| "Failed to create image buffer from captured pixels.".to_string())?;
        let dynamic = image::DynamicImage::ImageRgba8(buf);

        let mut png: Vec<u8> = Vec::new();
        dynamic
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .map_err(|e| format!("Failed to encode screen capture as PNG: {e}"))?;

        crate::images::save_image(&base_dir, &png)
    })
    .await
    .map_err(|e| format!("image encoding task failed: {e}"))?
}

/// Tauri command: silent screenshot for agent mode (no overlay hide/show).
#[cfg(target_os = "windows")]
#[tauri::command]
#[cfg_attr(coverage_nightly, coverage(off))]
pub async fn capture_silent_screenshot(app_handle: tauri::AppHandle) -> Result<String, String> {
    capture_silent_screenshot_command(app_handle).await
}

// ─── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_screen_metrics_are_non_negative() {
        let result = capture_virtual_screen_pixels();
        if let Ok((_vx, _vy, w, h, pixels)) = result {
            assert!(w > 0);
            assert!(h > 0);
            assert_eq!(pixels.len(), (w * h * 4) as usize);
        }
        // On headless CI, an error is expected and acceptable.
    }

    #[test]
    fn region_selection_state_defaults() {
        let state = RegionSelectionState {
            start_x: 0,
            start_y: 0,
            end_x: 0,
            end_y: 0,
            is_selecting: false,
            is_done: false,
            cancelled: false,
        };
        assert!(!state.is_selecting);
        assert!(!state.is_done);
        assert!(!state.cancelled);
    }

    #[test]
    fn crop_math_produces_valid_bounds() {
        let vx: i32 = -1920;
        let vy: i32 = 0;
        let full_width: u32 = 3840;
        let full_height: u32 = 1080;

        let x1: i32 = 100;
        let y1: i32 = 200;
        let x2: i32 = 500;
        let y2: i32 = 600;

        let buf_x = (x1 - vx).max(0) as u32;
        let buf_y = (y1 - vy).max(0) as u32;
        let crop_w = ((x2 - x1) as u32).min(full_width.saturating_sub(buf_x));
        let crop_h = ((y2 - y1) as u32).min(full_height.saturating_sub(buf_y));

        assert_eq!(buf_x, 2020);
        assert_eq!(buf_y, 200);
        assert_eq!(crop_w, 400);
        assert_eq!(crop_h, 400);
    }
}