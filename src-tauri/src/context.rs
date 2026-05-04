//! Captures contextual information at the moment of overlay activation.
//!
//! Queries the platform Accessibility API to detect any currently selected text
//! and its screen bounds. Falls back gracefully when the focused app does not
//! fully implement the accessibility protocol.
//!
//! `ActivationContext` and `calculate_window_position` are cross-platform.
//! Windows uses its own context capture in `windows_activator`.

// ─── Cross-platform public types ─────────────────────────────────────────────

/// Platform-independent screen rectangle in logical points (top-left origin).
#[derive(Debug, Clone, Copy)]
pub struct ScreenRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Context captured at the moment of overlay activation.
#[derive(Debug, Clone)]
pub struct ActivationContext {
    /// The currently selected text in the focused app, if any.
    pub selected_text: Option<String>,
    /// Screen bounds of the selection in logical points.
    /// `None` when AX cannot provide bounds for the selection (e.g. Chromium apps).
    pub bounds: Option<ScreenRect>,
    /// Mouse cursor position in logical screen coordinates at activation time.
    /// Used as a positioning anchor when `bounds` is unavailable but text was captured.
    pub mouse_position: Option<(f64, f64)>,
}

impl ActivationContext {
    /// Returns an empty context with no selection, bounds, or mouse position.
    /// Used for menu-item and tray-icon activations where no host-app context
    /// is available.
    pub fn empty() -> Self {
        Self {
            selected_text: None,
            bounds: None,
            mouse_position: None,
        }
    }
}

// ─── Activation context capture ──────────────────────────────────────────────

/// Captures the current activation context at the moment of the hotkey press.
///
/// When `overlay_is_visible` is `true` the hotkey will hide the overlay, so
/// no context is needed — skip queries entirely.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn capture_activation_context(overlay_is_visible: bool) -> ActivationContext {
    if overlay_is_visible {
        return ActivationContext::empty();
    }

    #[cfg(target_os = "windows")]
    {
        crate::windows_activator::capture()
    }

    #[cfg(not(target_os = "windows"))]
    {
        ActivationContext::empty()
    }
}

// ─── Positioning ──────────────────────────────────────────────────────────────

/// Distance (logical pts) to the right of the anchor point before the bar.
const ANCHOR_OFFSET_X: f64 = 8.0;
/// Distance (logical pts) above the anchor bottom edge for the bar top.
const ANCHOR_OFFSET_Y: f64 = 2.0;
/// Bottom padding of the overlay window in logical pts (pb-6 = 24 pt + motion py-2
/// bottom = 8 pt). Added when positioning the bar **above** a selection so the
/// bar's visible content bottom — not the transparent window edge — aligns with
/// the selection boundary.
const WINDOW_BOTTOM_PADDING: f64 = 32.0;
/// Minimum distance from any screen edge (logical pts).
pub(crate) const SCREEN_MARGIN: f64 = 16.0;
/// Menu bar height offset (logical pts). Always 0 on Windows (no system menu bar).
#[cfg(target_os = "windows")]
pub(crate) const MENU_BAR_HEIGHT: f64 = 0.0;
#[cfg(not(target_os = "windows"))]
pub(crate) const MENU_BAR_HEIGHT: f64 = 0.0;
/// Result of the window placement calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowPlacement {
    /// Logical X of the window's top-left corner.
    pub x: f64,
    /// Logical Y of the window's top-left corner.
    pub y: f64,
}

/// Returns the top-center position for the no-selection spawn point.
fn top_center(
    screen_width: f64,
    _screen_height: f64,
    window_width: f64,
    _window_height: f64,
) -> WindowPlacement {
    let x_min = SCREEN_MARGIN;
    let x_max = (screen_width - window_width - SCREEN_MARGIN).max(x_min);
    let x = ((screen_width - window_width) / 2.0).clamp(x_min, x_max);
    let y = MENU_BAR_HEIGHT + SCREEN_MARGIN + 120.0;
    WindowPlacement { x, y }
}

/// Positions the window to the right of `anchor_x / anchor_bottom_y`, flipping
/// horizontally when it would overflow the right screen edge and vertically when
/// it would overflow the bottom screen edge.
///
/// - `anchor_bottom_y`: bottom of the selection or mouse cursor Y.
/// - `anchor_top_y`: top of the selection (equals `anchor_bottom_y` for the
///   mouse-cursor case where there is no extent).
/// - `start_x`: left edge of the selection, used for the horizontal flip.
#[allow(clippy::too_many_arguments)]
fn anchor_near(
    anchor_x: f64,
    anchor_bottom_y: f64,
    anchor_top_y: f64,
    start_x: f64,
    screen_width: f64,
    screen_height: f64,
    window_width: f64,
    window_height: f64,
) -> WindowPlacement {
    // ── Horizontal ──────────────────────────────────────────────────────────
    let preferred_x = anchor_x + ANCHOR_OFFSET_X;
    let x = if preferred_x + window_width <= screen_width - SCREEN_MARGIN {
        preferred_x
    } else {
        // Bar grows leftward: right edge at (start_x - ANCHOR_OFFSET_X).
        (start_x - window_width - ANCHOR_OFFSET_X).max(SCREEN_MARGIN)
    };

    // ── Vertical ────────────────────────────────────────────────────────────
    let y_min = MENU_BAR_HEIGHT + SCREEN_MARGIN;
    let below_y = anchor_bottom_y - ANCHOR_OFFSET_Y;

    if below_y + window_height <= screen_height - SCREEN_MARGIN {
        // Enough room below: place just below the selection.
        WindowPlacement {
            x,
            y: below_y.max(y_min),
        }
    } else {
        // Not enough room below: flip above the selection. Shift by
        // WINDOW_BOTTOM_PADDING so the bar's visible content bottom (not the
        // transparent window edge) sits ANCHOR_OFFSET_Y pts above anchor_top_y.
        let fixed_bottom =
            (anchor_top_y - ANCHOR_OFFSET_Y + WINDOW_BOTTOM_PADDING).min(screen_height);
        let y = (fixed_bottom - window_height).max(y_min);
        WindowPlacement { x, y }
    }
}

/// Computes the window top-left position in logical screen coordinates.
///
/// - `screen_width` / `screen_height`: monitor size in logical points
/// - `window_width` / `window_height`: expected window size in logical points
pub fn calculate_window_position(
    ctx: &ActivationContext,
    screen_width: f64,
    screen_height: f64,
    window_width: f64,
    window_height: f64,
) -> WindowPlacement {
    if let Some(rect) = ctx.bounds {
        // AX provided full bounds: position near the end of the selection.
        anchor_near(
            rect.x + rect.width,
            rect.y + rect.height,
            rect.y,
            rect.x,
            screen_width,
            screen_height,
            window_width,
            window_height,
        )
    } else if ctx.selected_text.is_some() {
        // AX returned text but no bounds (Chromium apps) → anchor to mouse cursor.
        if let Some((mx, my)) = ctx.mouse_position {
            anchor_near(
                mx,
                my,
                my,
                mx,
                screen_width,
                screen_height,
                window_width,
                window_height,
            )
        } else {
            top_center(screen_width, screen_height, window_width, window_height)
        }
    } else {
        // No selection → top center of screen.
        top_center(screen_width, screen_height, window_width, window_height)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with_bounds(x: f64, y: f64, w: f64, h: f64) -> ActivationContext {
        ActivationContext {
            selected_text: Some("hello".to_string()),
            bounds: Some(ScreenRect {
                x,
                y,
                width: w,
                height: h,
            }),
            mouse_position: None,
        }
    }

    fn ctx_no_selection() -> ActivationContext {
        ActivationContext {
            selected_text: None,
            bounds: None,
            mouse_position: None,
        }
    }

    fn ctx_text_no_bounds_with_mouse(mx: f64, my: f64) -> ActivationContext {
        ActivationContext {
            selected_text: Some("hello".to_string()),
            bounds: None,
            mouse_position: Some((mx, my)),
        }
    }

    const SW: f64 = 1440.0;
    const SH: f64 = 900.0;
    const WW: f64 = 600.0;
    const WH: f64 = 80.0;

    #[test]
    fn no_selection_returns_top_center() {
        let p = calculate_window_position(&ctx_no_selection(), SW, SH, WW, WH);
        assert_eq!(p.x, (SW - WW) / 2.0);
        assert_eq!(p.y, MENU_BAR_HEIGHT + SCREEN_MARGIN + 120.0);
    }

    #[test]
    fn text_with_no_bounds_and_no_mouse_falls_back_to_top_center() {
        let ctx = ActivationContext {
            selected_text: Some("hello world".to_string()),
            bounds: None,
            mouse_position: None,
        };
        let p = calculate_window_position(&ctx, SW, SH, WW, WH);
        let x_min = SCREEN_MARGIN;
        let x_max = (SW - WW - SCREEN_MARGIN).max(x_min);
        assert_eq!(p.x, ((SW - WW) / 2.0).clamp(x_min, x_max));
        assert_eq!(p.y, MENU_BAR_HEIGHT + SCREEN_MARGIN + 120.0);
    }

    #[test]
    fn text_with_no_bounds_uses_mouse_position() {
        // Mouse at (400, 300). below_y = 298. Room below → normal placement.
        let ctx = ctx_text_no_bounds_with_mouse(400.0, 300.0);
        let p = calculate_window_position(&ctx, SW, SH, WW, WH);
        assert_eq!(p.x, 400.0 + ANCHOR_OFFSET_X);
        let expected_y = 300.0 - ANCHOR_OFFSET_Y;
        assert!((p.y - expected_y).abs() < 0.01);
    }

    #[test]
    fn selection_positions_near_end() {
        // Selection at x=100, y=300, w=80, h=20. End at (180, 320).
        let ctx = ctx_with_bounds(100.0, 300.0, 80.0, 20.0);
        let p = calculate_window_position(&ctx, SW, SH, WW, WH);
        assert_eq!(p.x, 180.0 + ANCHOR_OFFSET_X);
        let expected_y = 320.0 - ANCHOR_OFFSET_Y;
        assert!((p.y - expected_y).abs() < 0.01);
    }

    #[test]
    fn selection_near_top_clamps_to_min_y() {
        // Selection near top of screen: below_y = 10 - 2 = 8, clamped to y_min.
        let ctx = ctx_with_bounds(100.0, 0.0, 80.0, 10.0);
        let p = calculate_window_position(&ctx, SW, SH, WW, WH);
        let y_min = MENU_BAR_HEIGHT + SCREEN_MARGIN;
        assert_eq!(p.y, y_min);
    }

    #[test]
    fn selection_near_right_edge_flips_to_start() {
        // Selection end at 980. Window (600) would reach 1588 → overflows 1440-16=1424.
        let ctx = ctx_with_bounds(900.0, 300.0, 80.0, 20.0);
        let p = calculate_window_position(&ctx, SW, SH, WW, WH);
        assert_eq!(p.x, 900.0 - WW - ANCHOR_OFFSET_X);
    }

    #[test]
    fn flipped_x_is_clamped_by_screen_margin() {
        // Selection starts at x=10, end at x=1430 (near right edge).
        // preferred_x = 1430 + 8 = 1438. 1438 + 600 = 2038 > 1440 - 16 = 1424 → flip.
        // flipped_x = (10.0 - 600.0 - 8.0).max(16.0) = (-598.0).max(16.0) = 16.0
        let ctx = ctx_with_bounds(10.0, 300.0, 1420.0, 20.0);
        let p = calculate_window_position(&ctx, SW, SH, WW, WH);
        assert_eq!(p.x, SCREEN_MARGIN);
    }

    #[test]
    fn y_flips_above_when_selection_near_screen_bottom() {
        // Selection: y=870, h=20. below_y=888. 888+80=968 > (SH - SCREEN_MARGIN) → flip above.
        // fixed_bottom = min(870-2+32, 900) = 900. y = (900-80).max(y_min) = 820.
        let ctx = ctx_with_bounds(100.0, 870.0, 80.0, 20.0);
        let p = calculate_window_position(&ctx, SW, SH, WW, WH);
        assert_eq!(p.y, 820.0);
    }

    #[test]
    fn y_is_clamped_when_near_top_edge() {
        // Selection bottom at 12 → below_y = 10. 10+80=90 < 884 → no flip.
        // below_y.max(y_min) = 10.max(MENU_BAR_HEIGHT + SCREEN_MARGIN).
        let ctx = ctx_with_bounds(100.0, 0.0, 80.0, 12.0);
        let p = calculate_window_position(&ctx, SW, SH, WW, WH);
        let y_min = MENU_BAR_HEIGHT + SCREEN_MARGIN;
        assert_eq!(p.y, y_min);
    }

    #[test]
    fn zero_sized_selection_rect() {
        let ctx = ctx_with_bounds(200.0, 400.0, 0.0, 0.0);
        let p = calculate_window_position(&ctx, SW, SH, WW, WH);
        assert_eq!(p.x, 200.0 + ANCHOR_OFFSET_X);
    }

    #[test]
    fn selection_spanning_full_screen_width() {
        let ctx = ctx_with_bounds(0.0, 300.0, SW, 20.0);
        let p = calculate_window_position(&ctx, SW, SH, WW, WH);
        assert_eq!(p.x, SCREEN_MARGIN);
    }

    #[test]
    fn very_tall_screen_positions_below() {
        let ctx = ctx_with_bounds(100.0, 100.0, 80.0, 20.0);
        let tall_screen = 2000.0;
        let p = calculate_window_position(&ctx, SW, tall_screen, WW, WH);
        // below_y = 118. 118+80=198 < 2000-16=1984 → placed below.
        assert_eq!(p.y, 120.0 - ANCHOR_OFFSET_Y);
    }

    #[test]
    fn mouse_near_screen_edge_flips() {
        let ctx = ctx_text_no_bounds_with_mouse(1430.0, 300.0);
        let p = calculate_window_position(&ctx, SW, SH, WW, WH);
        assert_eq!(p.x, (1430.0 - WW - ANCHOR_OFFSET_X).max(SCREEN_MARGIN));
    }

    #[test]
    fn capture_activation_context_returns_empty_when_visible() {
        let ctx = capture_activation_context(true);
        assert!(ctx.selected_text.is_none());
        assert!(ctx.bounds.is_none());
        assert!(ctx.mouse_position.is_none());
    }

    #[test]
    fn activation_context_empty_has_no_fields() {
        let ctx = ActivationContext::empty();
        assert!(ctx.selected_text.is_none());
        assert!(ctx.bounds.is_none());
        assert!(ctx.mouse_position.is_none());
    }

    #[test]
    fn top_center_on_small_screen() {
        let small_w = WW + 2.0 * SCREEN_MARGIN;
        let p = calculate_window_position(&ctx_no_selection(), small_w, SH, WW, WH);
        assert_eq!(p.x, SCREEN_MARGIN);
    }
}
