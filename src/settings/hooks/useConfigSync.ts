/**
 * Loads the resolved `RawAppConfig` on mount and re-syncs whenever the
 * Settings window gains focus (file may have changed externally).
 */

import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

import type { RawAppConfig } from '../types';

export interface ConfigSyncHandle {
  config: RawAppConfig | null;
  reload: () => Promise<void>;
  setConfig: (next: RawAppConfig) => void;
}

export function useConfigSync(): ConfigSyncHandle {
  const [config, setConfig] = useState<RawAppConfig | null>(null);

  const reload = useCallback(async () => {
    try {
      const next = await invoke<RawAppConfig>('reload_config_from_disk');
      setConfig(next);
    } catch {
      // Reload failure is non-fatal; the previous in-memory snapshot is
      // still valid.
    }
  }, []);

  useEffect(() => {
    let mounted = true;
    void invoke<RawAppConfig>('get_config').then((next) => {
      if (mounted) setConfig(next);
    });

    const window_ = getCurrentWindow();
    let unlisten: (() => void) | null = null;
    void window_
      .onFocusChanged(({ payload: focused }) => {
        if (focused) void reload();
      })
      .then((stop) => {
        unlisten = stop;
        if (!mounted) stop();
      });

    return () => {
      mounted = false;
      unlisten?.();
    };
  }, [reload]);

  return { config, reload, setConfig };
}