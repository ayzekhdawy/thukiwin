/**
 * useModelSelection — hook for model picker state, warmup, and capability checks.
 *
 * Manages the active model, installed model list, Ollama reachability, model
 * capabilities, and warmup status. Exposes a unified interface for the
 * ModelPicker panel and CapabilityMismatchStrip.
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface Capabilities {
  vision: boolean;
  thinking: boolean;
  maxImages: number | null;
}

export interface ModelPickerState {
  active: string | null;
  all: string[];
  ollamaReachable: boolean;
}

export interface ModelSetupState {
  state: 'ollama_unreachable' | 'no_models_installed' | 'ready';
  active_slug?: string;
  installed?: string[];
}

export function useModelSelection() {
  const [pickerState, setPickerState] = useState<ModelPickerState>({
    active: null,
    all: [],
    ollamaReachable: false,
  });
  const [capabilities, setCapabilities] = useState<Record<string, Capabilities>>({});
  const [warmupStatus, setWarmupStatus] = useState<'idle' | 'warming' | 'loaded' | 'evicted'>('idle');

  const refreshPicker = useCallback(async () => {
    try {
      const state = await invoke<ModelPickerState>('get_model_picker_state');
      setPickerState(state);
      return state;
    } catch {
      setPickerState((prev) => ({ ...prev, ollamaReachable: false }));
      return null;
    }
  }, []);

  const refreshCapabilities = useCallback(async () => {
    try {
      const caps = await invoke<Record<string, Capabilities>>('get_model_capabilities');
      setCapabilities(caps);
      return caps;
    } catch {
      return null;
    }
  }, []);

  const selectModel = useCallback(async (model: string) => {
    await invoke('set_active_model', { model });
    await refreshPicker();
    // Warm up the newly selected model so it's loaded in VRAM.
    setWarmupStatus('warming');
    try {
      await invoke('warm_up_model');
      setWarmupStatus('loaded');
    } catch {
      setWarmupStatus('idle');
    }
  }, [refreshPicker]);

  const warmup = useCallback(async () => {
    setWarmupStatus('warming');
    try {
      await invoke('warm_up_model');
      setWarmupStatus('loaded');
    } catch {
      setWarmupStatus('idle');
    }
  }, []);

  const evict = useCallback(async () => {
    try {
      await invoke('evict_model');
      setWarmupStatus('evicted');
    } catch {
      // ignore
    }
  }, []);

  const checkLoaded = useCallback(async () => {
    try {
      const loaded = await invoke<string | null>('get_loaded_model');
      setWarmupStatus(loaded ? 'loaded' : 'idle');
      return loaded;
    } catch {
      return null;
    }
  }, []);

  const checkSetup = useCallback(async () => {
    try {
      return await invoke<ModelSetupState>('check_model_setup');
    } catch {
      return { state: 'ollama_unreachable' as const };
    }
  }, []);

  // Load initial state on mount
  useEffect(() => {
    void refreshPicker();
    void refreshCapabilities();
    void checkLoaded();
  }, [refreshPicker, refreshCapabilities, checkLoaded]);

  return {
    ...pickerState,
    capabilities,
    warmupStatus,
    selectModel,
    warmup,
    evict,
    checkLoaded,
    checkSetup,
    refreshPicker,
    refreshCapabilities,
  };
}