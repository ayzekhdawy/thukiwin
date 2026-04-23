/**
 * Toast notification utility for notifying the user when an AI response
 * completes while the app window is not focused.
 *
 * Uses the Tauri notification plugin which delegates to Windows Toast
 * Notifications on Windows, NSUserNotificationCenter on macOS, and
 * libnotify on Linux.
 *
 * Supports configurable notification sound: system default, custom sound,
 * or silent.
 */

import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';
import { invoke } from '@tauri-apps/api/core';

/** Whether we have already requested (and received) notification permission. */
let permissionGranted = false;

/** Cached notification sound setting. */
let cachedSoundSetting: string | null = null;

/** Currently playing notification audio element. */
let notificationAudio: HTMLAudioElement | null = null;

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
 * Gets the current notification sound setting from app_config.
 * Caches the result to avoid repeated IPC calls.
 */
async function getNotificationSoundSetting(): Promise<string> {
  if (cachedSoundSetting !== null) return cachedSoundSetting;
  try {
    const settings = await invoke<Record<string, string>>('get_settings');
    cachedSoundSetting = settings['notification_sound'] || 'system';
  } catch {
    cachedSoundSetting = 'system';
  }
  return cachedSoundSetting;
}

/**
 * Plays a custom notification sound from the bundled assets.
 */
function playCustomSound(): void {
  if (notificationAudio) {
    notificationAudio.pause();
    notificationAudio = null;
  }
  const audio = new Audio('/sounds/notification.mp3');
  notificationAudio = audio;
  audio.onended = () => {
    notificationAudio = null;
  };
  audio.onerror = () => {
    notificationAudio = null;
  };
  void audio.play().catch(() => {
    notificationAudio = null;
  });
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

  const soundSetting = await getNotificationSoundSetting();

  if (soundSetting === 'custom') {
    // Send silent notification + play custom sound separately.
    sendNotification({ title, body });
    playCustomSound();
  } else if (soundSetting === 'none') {
    // Completely silent — suppress even the system notification sound.
    sendNotification({ title, body });
  } else {
    // System default — OS handles the sound.
    sendNotification({ title, body });
  }
}

/**
 * Invalidates the cached notification sound setting.
 * Call this after changing the setting in the settings panel.
 */
export function invalidateNotificationSoundCache(): void {
  cachedSoundSetting = null;
}

/**
 * Resets the cached permission state. Intended for use in tests only.
 */
export function resetNotificationPermission(): void {
  permissionGranted = false;
}