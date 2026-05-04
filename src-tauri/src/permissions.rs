/*!
 * Permissions Module
 *
 * Pure-logic helper that decides whether the onboarding screen must be shown.
 * Platform-specific permission commands live in their own modules:
 * `windows_permissions` for Windows.
 */

/// Returns `true` when at least one required permission has not been granted.
///
/// Both Accessibility (hotkey listener) and Screen Recording (/screen command)
/// must be granted for Thuki to function fully. If either is missing the
/// onboarding screen is shown instead of the normal overlay.
pub fn needs_onboarding(accessibility: bool, screen_recording: bool) -> bool {
    !accessibility || !screen_recording
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needs_onboarding_false_when_both_granted() {
        assert!(!needs_onboarding(true, true));
    }

    #[test]
    fn needs_onboarding_true_when_accessibility_missing() {
        assert!(needs_onboarding(false, true));
    }

    #[test]
    fn needs_onboarding_true_when_screen_recording_missing() {
        assert!(needs_onboarding(true, false));
    }

    #[test]
    fn needs_onboarding_true_when_both_missing() {
        assert!(needs_onboarding(false, false));
    }
}
