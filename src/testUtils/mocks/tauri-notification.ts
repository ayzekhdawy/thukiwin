/**
 * Mock for @tauri-apps/plugin-notification.
 *
 * All functions are no-ops in the test environment.
 * Tests that need to verify notification behaviour should use the
 * dedicated notification.test.ts which has its own vi.mock setup.
 */

export async function isPermissionGranted(): Promise<boolean> {
  return false;
}

export async function requestPermission(): Promise<string> {
  return 'denied';
}

export function sendNotification(): void {
  // no-op in tests
}