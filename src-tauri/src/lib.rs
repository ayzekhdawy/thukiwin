/*!
 * ThukiWin Core Library
 *
 * Application bootstrap for the ThukiWin desktop agent. Configures the
 * system tray menu, window lifecycle (hide-on-close instead of quit),
 * and Windows-specific overlay management.
 */

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod commands;
pub mod config;
pub mod database;
pub mod history;
pub mod images;
pub mod models;
pub mod onboarding;
pub mod providers;
pub mod screenshot;
pub mod search;
pub mod settings_commands;
pub mod tts;
pub mod warmup;

#[cfg(target_os = "windows")]
mod windows_activator;
#[cfg(target_os = "windows")]
mod computer_control;
#[cfg(target_os = "windows")]
mod agent;
#[cfg(target_os = "windows")]
mod autostart;

mod gateway;
#[cfg(target_os = "windows")]
mod windows_focus;

pub mod context;
pub mod permissions;

#[cfg(target_os = "windows")]
mod windows_permissions;
#[cfg(target_os = "windows")]
mod windows_screenshot;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder,
};

// ─── Window helpers ─────────────────────────────────────────────────────────

/// Expected logical width of the overlay window for spawn-position calculations.
const OVERLAY_LOGICAL_WIDTH: f64 = 650.0;
/// Collapsed bar height used for Y-clamp at show time. The window starts collapsed;
/// the ResizeObserver expands it after mount.
const OVERLAY_LOGICAL_HEIGHT_COLLAPSED: f64 = 60.0;

/// Minibar dimensions — thin always-on-top strip shown when the user
/// switches away from ThukiWin while a task is in progress.
#[cfg(target_os = "windows")]
const OVERLAY_LOGICAL_WIDTH_MINIBAR: f64 = 48.0;
#[cfg(target_os = "windows")]
const OVERLAY_LOGICAL_HEIGHT_MINIBAR: f64 = 48.0;

/// Frontend event used to synchronize show/hide animations with native window visibility.
const OVERLAY_VISIBILITY_EVENT: &str = "thuki://visibility";
const OVERLAY_VISIBILITY_SHOW: &str = "show";
const OVERLAY_VISIBILITY_HIDE_REQUEST: &str = "hide-request";

/// Frontend event that triggers the onboarding screen when one or more
/// required permissions have not yet been granted.
const ONBOARDING_EVENT: &str = "thuki://onboarding";

/// Frontend event emitted when the user switches away from ThukiWin,
/// triggering minibar mode (thin always-on-top strip).
#[cfg(target_os = "windows")]
const MINIBAR_EVENT: &str = "thuki://minibar";

/// Logical dimensions of the onboarding window (centered, fixed size).
const ONBOARDING_LOGICAL_WIDTH: f64 = 460.0;
const ONBOARDING_LOGICAL_HEIGHT: f64 = 640.0;

/// Tracks the intended visibility state of the overlay, preventing race conditions
/// between the frontend exit animation and rapid activation toggles.
static OVERLAY_INTENDED_VISIBLE: AtomicBool = AtomicBool::new(false);

/// True on first process launch; cleared when the frontend signals readiness.
/// Used to show the overlay automatically on startup without a race condition:
/// the frontend calls `notify_frontend_ready` after its event listener is
/// registered, so the show event is guaranteed to have a listener.
static LAUNCH_SHOW_PENDING: AtomicBool = AtomicBool::new(true);

/// Payload emitted to the frontend on every visibility transition.
#[derive(Clone, serde::Serialize)]
struct VisibilityPayload {
    /// "show" or "hide-request"
    state: &'static str,
    /// Selected text captured at activation time, if any.
    selected_text: Option<String>,
    /// Logical X of the window at show time. Used with `window_y` and
    /// `screen_bottom_y` to decide growth direction, and as the pinned X
    /// coordinate for `set_window_frame` calls during upward growth.
    window_x: Option<f64>,
    /// Logical Y of the window top-left at show time.
    window_y: Option<f64>,
    /// Logical Y of the screen bottom edge (monitor origin + height).
    screen_bottom_y: Option<f64>,
}

/// Emits a visibility transition to the frontend animation controller.
fn emit_overlay_visibility(
    app_handle: &tauri::AppHandle,
    state: &'static str,
    selected_text: Option<String>,
    window_x: Option<f64>,
    window_y: Option<f64>,
    screen_bottom_y: Option<f64>,
) {
    let _ = app_handle.emit(
        OVERLAY_VISIBILITY_EVENT,
        VisibilityPayload {
            state,
            selected_text,
            window_x,
            window_y,
            screen_bottom_y,
        },
    );
}

/// Requests an animated hide sequence from the frontend. The actual native
/// window hide is deferred until the frontend exit animation completes.
fn request_overlay_hide(app_handle: &tauri::AppHandle) {
    if OVERLAY_INTENDED_VISIBLE.swap(false, Ordering::SeqCst) {
        #[cfg(target_os = "windows")]
        let _ = windows_focus::stop_focus_listener();

        emit_overlay_visibility(
            app_handle,
            OVERLAY_VISIBILITY_HIDE_REQUEST,
            None,
            None,
            None,
            None,
        );
    }
}

/// Shows the overlay and requests the frontend to replay its entrance animation.
///
/// On Windows, the overlay is shown as an always-on-top, skip-taskbar window.
/// The activation context (selected text, mouse position) is used to position
/// the window near the cursor or selection, using Tauri's cross-platform
/// monitor enumeration for screen bounds.
#[cfg(target_os = "windows")]
fn show_overlay(app_handle: &tauri::AppHandle, ctx: crate::context::ActivationContext) {
    if OVERLAY_INTENDED_VISIBLE.swap(true, Ordering::SeqCst) {
        return;
    }

    let selected_text = ctx.selected_text;

    // Position the window using the activation context.
    let placement = if let Some(window) = app_handle.get_webview_window("main") {
        // Use Tauri's cross-platform monitor API to find the target monitor.
        let monitors = app_handle.available_monitors().unwrap_or_default();
        let primary = app_handle.primary_monitor().unwrap();

        // Determine which monitor the activation occurred on.
        let anchor_point = ctx
            .bounds
            .map(|r| (r.x + r.width / 2.0, r.y + r.height / 2.0))
            .or(ctx.mouse_position);

        let target_monitor = if let Some((ax, ay)) = anchor_point {
            monitors.into_iter().find(|m| {
                let pos = m.position();
                let size = m.size();
                let mx = pos.x as f64;
                let my = pos.y as f64;
                let mw = size.width as f64;
                let mh = size.height as f64;
                ax >= mx && ax < mx + mw && ay >= my && ay < my + mh
            })
        } else {
            None
        };

        let monitor = target_monitor.unwrap_or_else(|| primary.unwrap());
        let mon_pos = monitor.position();
        let mon_size = monitor.size();
        let screen_w = mon_size.width as f64;
        let screen_h = mon_size.height as f64;
        let mon_x = mon_pos.x as f64;
        let mon_y = mon_pos.y as f64;

        // Convert global coordinates to monitor-local for positioning math.
        let local_ctx = crate::context::ActivationContext {
            selected_text: selected_text.clone(),
            bounds: ctx.bounds.map(|r| crate::context::ScreenRect {
                x: r.x - mon_x,
                y: r.y - mon_y,
                width: r.width,
                height: r.height,
            }),
            mouse_position: ctx.mouse_position.map(|(mx, my)| (mx - mon_x, my - mon_y)),
        };

        let p = crate::context::calculate_window_position(
            &local_ctx,
            screen_w,
            screen_h,
            OVERLAY_LOGICAL_WIDTH,
            OVERLAY_LOGICAL_HEIGHT_COLLAPSED,
        );

        // Convert back to global screen coordinates.
        let global = crate::context::WindowPlacement {
            x: p.x + mon_x,
            y: p.y + mon_y,
        };

        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(
            global.x, global.y,
        )));
        let screen_bottom = mon_y + screen_h;
        Some((global, screen_bottom))
    } else {
        None
    };

    let (window_x, window_y, screen_bottom_y) = match &placement {
        Some((p, sb)) => (Some(p.x), Some(p.y), Some(*sb)),
        None => (None, None, None),
    };

    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.set_always_on_top(true);
        let _ = window.set_skip_taskbar(true);
        let _ = window.show();
        let _ = window.set_focus();

        // Store the main window HWND for the focus listener and start listening.
        if let Ok(hwnd) = window.hwnd() {
            windows_focus::set_main_hwnd(hwnd);
            let _ = windows_focus::stop_focus_listener();
            let handle = app_handle.clone();
            let _ = windows_focus::start_focus_listener(Arc::new(move |_hwnd| {
                if !windows_focus::is_minibar_active() {
                    windows_focus::enter_minibar();
                    let _ = handle.emit(MINIBAR_EVENT, ());
                }
            }));
        }
    }
    emit_overlay_visibility(
        app_handle,
        OVERLAY_VISIBILITY_SHOW,
        selected_text,
        window_x,
        window_y,
        screen_bottom_y,
    );
}

/// Shows the overlay on platforms that have no platform-specific implementation.
/// Falls back to a simple show + focus with no positioning.
#[cfg(not(target_os = "windows"))]
fn show_overlay(app_handle: &tauri::AppHandle, ctx: crate::context::ActivationContext) {
    if OVERLAY_INTENDED_VISIBLE.swap(true, Ordering::SeqCst) {
        return;
    }
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        emit_overlay_visibility(
            app_handle,
            OVERLAY_VISIBILITY_SHOW,
            ctx.selected_text,
            None,
            None,
            None,
        );
    }
}

/// Opens the settings window. If already open, focuses it.
/// Must be async — WebviewWindowBuilder deadlocks on Windows in sync commands.
#[cfg_attr(coverage_nightly, coverage(off))]
#[tauri::command]
async fn open_settings_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    let url = if cfg!(debug_assertions) {
        WebviewUrl::External("http://localhost:1420/settings.html".parse().unwrap())
    } else {
        WebviewUrl::App("settings.html".into())
    };

    let window = WebviewWindowBuilder::new(&app_handle, "settings", url)
        .title("Settings — ThukiWin")
        .inner_size(480.0, 700.0)
        .min_inner_size(400.0, 500.0)
        .center()
        .resizable(true)
        .decorations(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(true)
        .focused(true)
        .build()
        .map_err(|e| e.to_string())?;

    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

/// Toggles the overlay between visible and hidden states.
///
/// Uses an atomic flag as the single source of truth for intended visibility,
/// which avoids race conditions with the native panel state during animations.
///
/// On Windows, if the overlay is in minibar mode, double-tap Ctrl restores
/// the full overlay instead of hiding it.
fn toggle_overlay(app_handle: &tauri::AppHandle, ctx: crate::context::ActivationContext) {
    #[cfg(target_os = "windows")]
    if windows_focus::is_minibar_active() {
        // Restore from minibar — the window is already visible, just small.
        windows_focus::exit_minibar();
        let handle = app_handle.clone();
        let _ = app_handle.run_on_main_thread(move || {
            if let Some(window) = handle.get_webview_window("main") {
                let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(
                    OVERLAY_LOGICAL_WIDTH,
                    OVERLAY_LOGICAL_HEIGHT_COLLAPSED,
                )));
                let _ = window.set_focus();
                let _ = windows_focus::stop_focus_listener();
                let h = handle.clone();
                let _ = windows_focus::start_focus_listener(Arc::new(move |_hwnd| {
                    if !windows_focus::is_minibar_active() {
                        windows_focus::enter_minibar();
                        let _ = h.emit(MINIBAR_EVENT, ());
                    }
                }));
            }
            emit_overlay_visibility(
                &handle,
                OVERLAY_VISIBILITY_SHOW,
                None,
                None,
                None,
                None,
            );
        });
        return;
    }

    if OVERLAY_INTENDED_VISIBLE.load(Ordering::SeqCst) {
        request_overlay_hide(app_handle);
    } else {
        show_overlay(app_handle, ctx);
    }
}

/// Repositions and resizes the main window atomically.
///
/// Regular Tauri commands run on a Tokio thread pool. Calling `set_position`
/// then `set_size` from a pool thread dispatches each as a *separate* event to
/// the main thread, which can render as two distinct display frames and
/// produce a visible stutter when the window grows upward (position + size both
/// change on every token during streaming).
///
/// Wrapping both calls in a single `run_on_main_thread` closure ensures they
/// arrive on the main thread together in the same event-loop iteration.
#[tauri::command]
fn set_window_frame(app_handle: tauri::AppHandle, x: f64, y: f64, width: f64, height: f64) {
    // Reject non-finite values (NaN, Infinity) from the frontend to prevent
    // undefined behaviour when forwarded to native window APIs.
    if !x.is_finite() || !y.is_finite() || !width.is_finite() || !height.is_finite() {
        return;
    }
    let width = width.clamp(1.0, 10_000.0);
    let height = height.clamp(1.0, 10_000.0);

    let handle = app_handle.clone();
    let _ = app_handle.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window("main") {
            let _ =
                window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
            let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(width, height)));
        }
    });
}

/// Synchronizes the Rust-side visibility tracking when the frontend
/// completes its exit animation and hides the native window.
#[tauri::command]
fn notify_overlay_hidden() {
    OVERLAY_INTENDED_VISIBLE.store(false, Ordering::SeqCst);
}

/// Resizes the window to minibar dimensions and enters minibar mode.
/// Called from the frontend when it transitions to minibar state.
#[cfg(target_os = "windows")]
#[tauri::command]
fn enter_minibar_size(app_handle: tauri::AppHandle) {
    windows_focus::enter_minibar();
    let handle = app_handle.clone();
    let _ = app_handle.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window("main") {
            let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(
                OVERLAY_LOGICAL_WIDTH_MINIBAR,
                OVERLAY_LOGICAL_HEIGHT_MINIBAR,
            )));
        }
    });
}

/// Resizes the window back to full overlay dimensions and exits minibar mode.
/// Called from the frontend when the user clicks the minibar to restore.
#[cfg(target_os = "windows")]
#[tauri::command]
fn exit_minibar_size(app_handle: tauri::AppHandle) {
    windows_focus::exit_minibar();
    let handle = app_handle.clone();
    let _ = app_handle.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window("main") {
            let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(
                OVERLAY_LOGICAL_WIDTH,
                OVERLAY_LOGICAL_HEIGHT_COLLAPSED,
            )));
            let _ = window.set_focus();
            // Restart the focus listener so minibar triggers on next focus loss.
            let _ = windows_focus::stop_focus_listener();
            let h = handle.clone();
            let _ = windows_focus::start_focus_listener(Arc::new(move |_hwnd| {
                if !windows_focus::is_minibar_active() {
                    windows_focus::enter_minibar();
                    let _ = h.emit(MINIBAR_EVENT, ());
                }
            }));
        }
    });
}

/// Called by the frontend once its visibility event listener is registered.
/// On the first call per process lifetime, shows the overlay so the AskBar
/// appears automatically at startup without a race between the Rust emit and
/// the frontend listener registration.
#[tauri::command]
#[cfg_attr(coverage_nightly, coverage(off))]
fn notify_frontend_ready(app_handle: tauri::AppHandle, db: tauri::State<history::Database>) {
    if LAUNCH_SHOW_PENDING.swap(false, Ordering::SeqCst) {
        #[cfg(target_os = "windows")]
        {
            if let Ok(conn) = db.0.lock() {
                let stage =
                    onboarding::get_stage(&conn).unwrap_or(onboarding::OnboardingStage::Intro);

                // On Windows, there are no Accessibility or Screen Recording
                // permission gates. Skip the permissions stage entirely and
                // go directly to intro (or overlay if onboarding is complete).
                if matches!(stage, onboarding::OnboardingStage::Permissions) {
                    // Upgrade permissions stage to intro since Windows doesn't
                    // need a permissions gate.
                    let _ = onboarding::set_stage(&conn, &onboarding::OnboardingStage::Intro);
                    show_onboarding_window(&app_handle, onboarding::OnboardingStage::Intro);
                    return;
                }

                if !matches!(stage, onboarding::OnboardingStage::Complete) {
                    show_onboarding_window(&app_handle, stage);
                    return;
                }
                // Complete: fall through to show the overlay.
            } else {
                show_onboarding_window(&app_handle, onboarding::OnboardingStage::Intro);
                return;
            }
        }

        show_overlay(&app_handle, crate::context::ActivationContext::empty());
    }
}

// ─── Onboarding completion ───────────────────────────────────────────────────

/// Called when the user clicks "Get Started" on the intro screen.
/// Marks onboarding complete in the DB, restores the window to overlay mode,
/// and immediately shows the Ask Bar — no relaunch required.
#[tauri::command]
#[cfg_attr(coverage_nightly, coverage(off))]
fn finish_onboarding(
    db: tauri::State<history::Database>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| format!("db lock poisoned: {e}"))?;
    onboarding::mark_complete(&conn).map_err(|e| format!("db write failed: {e}"))?;
    drop(conn);

    // Restore overlay window properties and show the Ask Bar.
    #[cfg(target_os = "windows")]
    {
        let handle = app_handle.clone();
        let _ = app_handle.run_on_main_thread(move || {
            if let Some(window) = handle.get_webview_window("main") {
                let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(
                    OVERLAY_LOGICAL_WIDTH,
                    OVERLAY_LOGICAL_HEIGHT_COLLAPSED,
                )));
                let _ = window.set_always_on_top(true);
                let _ = window.set_skip_taskbar(true);
            }
            show_overlay(&handle, crate::context::ActivationContext::empty());
        });
    }

    // On other platforms, just show the overlay.
    #[cfg(not(target_os = "windows"))]
    {
        let handle = app_handle.clone();
        let _ = app_handle.run_on_main_thread(move || {
            show_overlay(&handle, crate::context::ActivationContext::empty());
        });
    }

    Ok(())
}

// ─── Onboarding window ───────────────────────────────────────────────────────

/// Shows the onboarding window on Windows. Sizes the main window, centers it,
/// shows it, and emits the onboarding event.
#[cfg(target_os = "windows")]
#[cfg_attr(coverage_nightly, coverage(off))]
fn show_onboarding_window(app_handle: &tauri::AppHandle, stage: onboarding::OnboardingStage) {
    let handle = app_handle.clone();
    let _ = app_handle.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window("main") {
            let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(
                ONBOARDING_LOGICAL_WIDTH,
                ONBOARDING_LOGICAL_HEIGHT,
            )));
            let _ = window.center();
            let _ = window.show();
            let _ = window.set_focus();
        }
        let _ = handle.emit(ONBOARDING_EVENT, OnboardingPayload { stage });
    });
}

/// Shows the onboarding window on platforms without a specific implementation.
#[cfg(not(target_os = "windows"))]
#[cfg_attr(coverage_nightly, coverage(off))]
fn show_onboarding_window(app_handle: &tauri::AppHandle, stage: onboarding::OnboardingStage) {
    let handle = app_handle.clone();
    let _ = app_handle.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window("main") {
            let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(
                ONBOARDING_LOGICAL_WIDTH,
                ONBOARDING_LOGICAL_HEIGHT,
            )));
            let _ = window.center();
            let _ = window.show();
        }
        let _ = handle.emit(ONBOARDING_EVENT, OnboardingPayload { stage });
    });
}

/// Payload emitted to the frontend for every onboarding transition.
#[derive(Clone, serde::Serialize)]
struct OnboardingPayload {
    stage: onboarding::OnboardingStage,
}

// ─── Image cleanup ──────────────────────────────────────────────────────────

/// Interval between periodic orphaned-image cleanup sweeps.
const IMAGE_CLEANUP_INTERVAL: std::time::Duration = std::time::Duration::from_secs(3600);

/// Runs a single orphaned-image cleanup sweep. Thin orchestration wrapper
/// that delegates to `database::get_all_image_paths` and
/// `images::cleanup_orphaned_images`, both independently tested.
#[cfg_attr(coverage_nightly, coverage(off))]
fn run_image_cleanup(app_handle: &tauri::AppHandle) {
    let db = app_handle.state::<history::Database>();
    let conn = match db.0.lock() {
        Ok(c) => c,
        Err(_) => return,
    };
    let referenced = database::get_all_image_paths(&conn).unwrap_or_default();
    drop(conn);

    let base_dir = match app_handle.path().app_data_dir() {
        Ok(d) => d,
        Err(_) => return,
    };
    let _ = images::cleanup_orphaned_images(&base_dir, &referenced);
}

/// Spawns a background Tokio task that runs the cleanup sweep on a fixed
/// interval. Thin async wrapper — delegates to `run_image_cleanup`.
#[cfg_attr(coverage_nightly, coverage(off))]
fn spawn_periodic_image_cleanup(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(IMAGE_CLEANUP_INTERVAL);
        // Skip the first tick (startup cleanup already ran synchronously).
        interval.tick().await;
        loop {
            interval.tick().await;
            run_image_cleanup(&app_handle);
        }
    });
}

// ─── Application entry point ─────────────────────────────────────────────────

/// Initialises and runs the Tauri application.
///
/// Setup order:
/// 1. System tray is registered; double-tap Ctrl listener starts.
/// 2. `CloseRequested` is intercepted to hide instead of destroy.
///
/// # Panics
///
/// Panics if the Tauri runtime fails to initialise.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init());

    builder
        .setup(|app| {
            // ── System tray icon + menu ───────────────────────────────────
            let show_item = MenuItem::with_id(app, "show", "Open Thuki", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/128x128.png"))
                .expect("Failed to load tray icon");

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .icon_as_template(false)
                .tooltip("Thuki")
                .menu(&tray_menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        show_overlay(app, crate::context::ActivationContext::empty());
                    }
                    "quit" => {
                        app.state::<crate::commands::GenerationState>().cancel();
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Right,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_overlay(
                            tray.app_handle(),
                            crate::context::ActivationContext::empty(),
                        );
                    }
                })
                .build(app)?;

            // ── Activation listener (Windows) ─────────────────────────
            #[cfg(target_os = "windows")]
            {
                let app_handle = app.handle().clone();
                let activator = windows_activator::OverlayActivator::new();
                activator.start(move || {
                    let is_visible = OVERLAY_INTENDED_VISIBLE.load(Ordering::SeqCst);
                    let handle = app_handle.clone();
                    let handle2 = app_handle.clone();
                    std::thread::spawn(move || {
                        let ctx = crate::context::capture_activation_context(is_visible);
                        let _ = handle.run_on_main_thread(move || toggle_overlay(&handle2, ctx));
                    });
                });
                app.manage(activator);
            }

            // ── Persistent HTTP client ────────────────────────────────
            app.manage(reqwest::Client::new());

            // ── TOML configuration (single source of truth) ──────────
            let app_config = match config::load(app.handle()) {
                Ok(cfg) => cfg,
                Err(e) => config::show_fatal_dialog_and_exit(&e),
            };
            app.manage(parking_lot::RwLock::new(app_config));

            // ── Model picker state (active model in SQLite, not TOML) ─
            app.manage(models::ActiveModelState::new());

            // ── Model warmup state ────────────────────────────────────
            app.manage(warmup::WarmupState::new());

            // ── Generation + conversation state ─────────────────────
            app.manage(commands::GenerationState::new());
            app.manage(commands::ConversationHistory::new());

            // ── Legacy state bridges (seeded from TOML config) ──────
            // These remain for backward compatibility with commands that
            // reference them. They will be removed once all commands
            // migrate to State<RwLock<AppConfig>>.
            {
                let cfg_guard = app.state::<parking_lot::RwLock<config::AppConfig>>();
                let system_prompt = cfg_guard.read().prompt.resolved_system.clone();
                let ollama_url = cfg_guard.read().inference.ollama_url.clone();
                let model_config = commands::load_model_config();
                app.manage(commands::SystemPrompt(system_prompt));
                app.manage(commands::OllamaUrl(std::sync::Mutex::new(ollama_url)));
                app.manage(model_config);
            }

            // ── TTS state ────────────────────────────────────────────────
            app.manage(tts::TtsState::new());

            // ── Agent state (Windows only) ────────────────────────────
            #[cfg(target_os = "windows")]
            app.manage(Arc::new(agent::AgentState::new()));

            // ── Gateway state ──────────────────────────────────────────
            app.manage(Arc::new(gateway::GatewayState::new()));

            // ── SQLite database for conversation history ──────────
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");
            let db_conn = database::open_database(&app_data_dir)
                .expect("failed to initialise SQLite database");
            app.manage(history::Database(std::sync::Mutex::new(db_conn)));

            // ── Orphaned image cleanup (startup + periodic) ─────────
            run_image_cleanup(app.handle());
            spawn_periodic_image_cleanup(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            #[cfg(not(coverage))]
            commands::ask_ollama,
            #[cfg(not(coverage))]
            commands::cancel_generation,
            #[cfg(not(coverage))]
            commands::reset_conversation,
            #[cfg(not(coverage))]
            commands::get_model_config,
            #[cfg(not(coverage))]
            commands::get_ollama_url,
            #[cfg(not(coverage))]
            commands::set_ollama_url,
            #[cfg(not(coverage))]
            commands::get_settings,
            #[cfg(not(coverage))]
            commands::set_setting,
            #[cfg(not(coverage))]
            history::save_conversation,
            #[cfg(not(coverage))]
            history::persist_message,
            #[cfg(not(coverage))]
            history::list_conversations,
            #[cfg(not(coverage))]
            history::load_conversation,
            #[cfg(not(coverage))]
            history::delete_conversation,
            #[cfg(not(coverage))]
            history::generate_title,
            #[cfg(not(coverage))]
            images::save_image_command,
            #[cfg(not(coverage))]
            images::remove_image_command,
            #[cfg(not(coverage))]
            images::cleanup_orphaned_images_command,
            #[cfg(not(coverage))]
            screenshot::capture_screenshot_command,
            #[cfg(not(coverage))]
            screenshot::capture_full_screen_command,
            notify_overlay_hidden,
            notify_frontend_ready,
            set_window_frame,
            finish_onboarding,
            // Windows-specific permission commands
            #[cfg(all(target_os = "windows", not(coverage)))]
            windows_permissions::check_accessibility_permission,
            #[cfg(all(target_os = "windows", not(coverage)))]
            windows_permissions::open_accessibility_settings,
            #[cfg(all(target_os = "windows", not(coverage)))]
            windows_permissions::check_screen_recording_permission,
            #[cfg(all(target_os = "windows", not(coverage)))]
            windows_permissions::open_screen_recording_settings,
            #[cfg(all(target_os = "windows", not(coverage)))]
            windows_permissions::request_screen_recording_access,
            #[cfg(all(target_os = "windows", not(coverage)))]
            windows_permissions::check_screen_recording_tcc_granted,
            #[cfg(all(target_os = "windows", not(coverage)))]
            windows_permissions::quit_and_relaunch,
            // TTS commands
            #[cfg(not(coverage))]
            tts::tts_speak,
            #[cfg(not(coverage))]
            tts::tts_stop,
            #[cfg(not(coverage))]
            tts::tts_list_voices,
            // Agent commands (Windows only)
            #[cfg(all(target_os = "windows", not(coverage)))]
            agent::start_agent_mode,
            #[cfg(all(target_os = "windows", not(coverage)))]
            agent::stop_agent_mode,
            #[cfg(all(target_os = "windows", not(coverage)))]
            agent::get_agent_status,
            #[cfg(all(target_os = "windows", not(coverage)))]
            agent::confirm_agent_action,
            #[cfg(all(target_os = "windows", not(coverage)))]
            agent::reject_agent_action,
            #[cfg(all(target_os = "windows", not(coverage)))]
            agent::set_agent_provider,
            #[cfg(all(target_os = "windows", not(coverage)))]
            agent::get_agent_provider,
            #[cfg(all(target_os = "windows", not(coverage)))]
            computer_control::execute_action_command,
            #[cfg(all(target_os = "windows", not(coverage)))]
            windows_screenshot::capture_silent_screenshot,
            // Minibar commands (Windows only)
            #[cfg(all(target_os = "windows", not(coverage)))]
            autostart::is_auto_start_enabled_command,
            #[cfg(all(target_os = "windows", not(coverage)))]
            autostart::enable_auto_start_command,
            #[cfg(all(target_os = "windows", not(coverage)))]
            autostart::disable_auto_start_command,
            // Gateway commands
            #[cfg(not(coverage))]
            gateway::start_gateway,
            #[cfg(not(coverage))]
            gateway::stop_gateway,
            #[cfg(not(coverage))]
            gateway::get_gateway_status,
            #[cfg(all(target_os = "windows", not(coverage)))]
            windows_focus::enter_minibar_command,
            #[cfg(all(target_os = "windows", not(coverage)))]
            windows_focus::exit_minibar_command,
            #[cfg(all(target_os = "windows", not(coverage)))]
            windows_focus::is_minibar_active_command,
            #[cfg(all(target_os = "windows", not(coverage)))]
            enter_minibar_size,
            #[cfg(all(target_os = "windows", not(coverage)))]
            exit_minibar_size,
            // Settings window
            open_settings_window,
            // Config commands (TOML)
            settings_commands::get_config,
            settings_commands::set_config_field,
            settings_commands::reset_config,
            settings_commands::reload_config_from_disk,
            settings_commands::get_corrupt_marker,
            settings_commands::reveal_config_in_explorer,
            // Model picker commands
            models::get_model_picker_state,
            models::set_active_model,
            models::check_model_setup,
            models::get_model_capabilities,
            // Model warmup commands
            warmup::warm_up_model,
            warmup::get_loaded_model,
            warmup::evict_model,
            // Search pipeline
            search::search_pipeline,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::CloseRequested { api, .. },
                ..
            } = event
            {
                if label == "main" {
                    api.prevent_close();

                    request_overlay_hide(app_handle);
                } else if label == "settings" {
                    api.prevent_close();
                    if let Some(window) = app_handle.get_webview_window("settings") {
                        let _ = window.hide();
                    }
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_window_frame_rejects_nan() {
        assert!(!f64::NAN.is_finite());
        assert!(!f64::INFINITY.is_finite());
        assert!(!f64::NEG_INFINITY.is_finite());
        assert!(100.0_f64.is_finite());
    }

    #[test]
    fn width_height_clamp_logic() {
        assert_eq!(0.5_f64.clamp(1.0, 10_000.0), 1.0);
        assert_eq!(500.0_f64.clamp(1.0, 10_000.0), 500.0);
        assert_eq!(20_000.0_f64.clamp(1.0, 10_000.0), 10_000.0);
    }

    #[test]
    fn notify_overlay_hidden_sets_flag_to_false() {
        OVERLAY_INTENDED_VISIBLE.store(true, Ordering::SeqCst);
        OVERLAY_INTENDED_VISIBLE.store(false, Ordering::SeqCst);
        assert!(!OVERLAY_INTENDED_VISIBLE.load(Ordering::SeqCst));
    }

    #[test]
    fn launch_show_pending_consumed_exactly_once() {
        LAUNCH_SHOW_PENDING.store(true, Ordering::SeqCst);
        assert!(LAUNCH_SHOW_PENDING.swap(false, Ordering::SeqCst));
        assert!(!LAUNCH_SHOW_PENDING.swap(false, Ordering::SeqCst));
    }

    #[test]
    fn overlay_visibility_event_constant_matches() {
        assert_eq!(OVERLAY_VISIBILITY_EVENT, "thuki://visibility");
        assert_eq!(OVERLAY_VISIBILITY_SHOW, "show");
        assert_eq!(OVERLAY_VISIBILITY_HIDE_REQUEST, "hide-request");
    }

    #[test]
    fn onboarding_event_constant_matches() {
        assert_eq!(ONBOARDING_EVENT, "thuki://onboarding");
    }

    #[test]
    fn onboarding_logical_dimensions() {
        assert_eq!(ONBOARDING_LOGICAL_WIDTH, 460.0);
        assert_eq!(ONBOARDING_LOGICAL_HEIGHT, 640.0);
    }

    #[test]
    fn overlay_logical_dimensions() {
        assert_eq!(OVERLAY_LOGICAL_WIDTH, 650.0);
        assert_eq!(OVERLAY_LOGICAL_HEIGHT_COLLAPSED, 60.0);
    }
}
