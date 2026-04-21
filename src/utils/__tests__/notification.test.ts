import { describe, it, expect, vi, beforeEach } from 'vitest';

// ─── Mock @tauri-apps/plugin-notification ──────────────────────────────────
//
// vi.mock is hoisted before import declarations, so the factory function
// cannot reference module-level variables. We use vi.fn() directly inside
// the factory and access the mocks via the module import below.

vi.mock('@tauri-apps/plugin-notification', () => ({
  isPermissionGranted: vi.fn<() => Promise<boolean>>(async () => false),
  requestPermission: vi.fn<() => Promise<string>>(async () => 'granted'),
  sendNotification: vi.fn(),
}));

// Import after vi.mock so Vitest wires up the mock before the module loads.
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';
import {
  notifyIfUnfocused,
  resetNotificationPermission,
} from '../notification';

// ─── Mock document.hasFocus ──────────────────────────────────────────────────

const mockHasFocus = vi.spyOn(document, 'hasFocus');

beforeEach(() => {
  vi.clearAllMocks();
  resetNotificationPermission();
  // Default: window is not focused (unfocused scenario)
  mockHasFocus.mockReturnValue(false);
  // Default: permission not yet granted, request succeeds
  (isPermissionGranted as ReturnType<typeof vi.fn>).mockResolvedValue(false);
  (requestPermission as ReturnType<typeof vi.fn>).mockResolvedValue('granted');
  (sendNotification as ReturnType<typeof vi.fn>).mockReturnValue(undefined);
});

describe('notifyIfUnfocused', () => {
  it('does not send notification when window has focus', async () => {
    mockHasFocus.mockReturnValue(true);

    await notifyIfUnfocused('ThukiWin', 'Response ready');

    expect(sendNotification).not.toHaveBeenCalled();
  });

  it('requests permission and sends notification when unfocused', async () => {
    await notifyIfUnfocused('ThukiWin', 'Response ready');

    expect(isPermissionGranted).toHaveBeenCalledOnce();
    expect(requestPermission).toHaveBeenCalledOnce();
    expect(sendNotification).toHaveBeenCalledOnce();
    expect(sendNotification).toHaveBeenCalledWith({
      title: 'ThukiWin',
      body: 'Response ready',
    });
  });

  it('skips permission request on subsequent calls', async () => {
    await notifyIfUnfocused('ThukiWin', 'First');
    await notifyIfUnfocused('ThukiWin', 'Second');

    // Permission requested only once; sendNotification called twice.
    expect(requestPermission).toHaveBeenCalledOnce();
    expect(sendNotification).toHaveBeenCalledTimes(2);
  });

  it('does not send notification when permission is denied', async () => {
    (requestPermission as ReturnType<typeof vi.fn>).mockResolvedValue('denied');

    await notifyIfUnfocused('ThukiWin', 'Response ready');

    expect(sendNotification).not.toHaveBeenCalled();
  });

  it('sends notification immediately when permission already granted', async () => {
    (isPermissionGranted as ReturnType<typeof vi.fn>).mockResolvedValue(true);

    await notifyIfUnfocused('ThukiWin', 'Response ready');

    expect(requestPermission).not.toHaveBeenCalled();
    expect(sendNotification).toHaveBeenCalledOnce();
  });

  it('does not send notification when permission request fails', async () => {
    (isPermissionGranted as ReturnType<typeof vi.fn>).mockRejectedValue(
      new Error('not available'),
    );

    await notifyIfUnfocused('ThukiWin', 'Response ready');

    expect(sendNotification).not.toHaveBeenCalled();
  });
});

describe('resetNotificationPermission', () => {
  it('allows permission to be re-requested after reset', async () => {
    await notifyIfUnfocused('ThukiWin', 'First');
    expect(requestPermission).toHaveBeenCalledOnce();

    resetNotificationPermission();

    await notifyIfUnfocused('ThukiWin', 'Second');
    // After reset, permission is requested again
    expect(requestPermission).toHaveBeenCalledTimes(2);
  });
});