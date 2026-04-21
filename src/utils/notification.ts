/**
 * Toast notification utility for notifying the user when an AI response
 * completes while the app window is not focused.
 *
 * Uses the Tauri notification plugin which delegates to Windows Toast
 * Notifications on Windows, NSUserNotificationCenter on macOS, and
 * libnotify on Linux.
 */

import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';

/** Whether we have already requested (and received) notification permission. */
let permissionGranted = false;

/**
 * Requests notification permission from the OS if we have not already done so.
 * Safe to call repeatedly — only contacts the OS once.
 */
async function ensurePermission(): Promise<boolean> {
  if (permissionGranted) return true;

  try {
    let granted = await isPermissionGranted();
    if (!granted) {
      const permission = await requestPermission();
      granted = permission === 'granted';
    }
    permissionGranted = granted;
    return granted;
  } catch {
    // Permission request can fail in test environments or on platforms that
    // don't support notifications; silently skip.
    return false;
  }
}

/**
 * Shows a desktop toast notification **only if** the app window is not focused.
 *
 * Call this when an AI response finishes streaming. The notification lets the
 * user know the response is ready even if they have switched to another app.
 *
 * @param title   Notification title (e.g. "ThukiWin")
 * @param body    Notification body text (e.g. "Response ready")
 */
export async function notifyIfUnfocused(
  title: string,
  body: string,
): Promise<void> {
  // Only show a notification when the window does NOT have focus.
  // This avoids an annoying pop-up every time a response completes while
  // the user is actively reading it in the overlay.
  if (document.hasFocus()) return;

  const granted = await ensurePermission();
  if (!granted) return;

  sendNotification({ title, body });
}

/**
 * Resets the cached permission state. Intended for use in tests only.
 */
export function resetNotificationPermission(): void {
  permissionGranted = false;
}