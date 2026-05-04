/**
 * Per-field debounced auto-save hook for the Settings panel.
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

import type { ConfigError, RawAppConfig } from '../types';

export interface DebouncedSaveHandle<TValue> {
  error: ConfigError | null;
  flushNow: () => Promise<RawAppConfig | null>;
  resetTo: (next: TValue) => void;
}

export function useDebouncedSave<TValue>(
  section: string,
  key: string,
  value: TValue,
  options: {
    delayMs?: number;
    onSaved?: (next: RawAppConfig) => void;
  } = {},
): DebouncedSaveHandle<TValue> {
  const { delayMs = 250, onSaved } = options;
  const [error, setError] = useState<ConfigError | null>(null);

  const valueRef = useRef(value);
  valueRef.current = value;

  const lastSavedRef = useRef<TValue>(value);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const epochRef = useRef(0);
  const isMountedRef = useRef(true);

  const sectionRef = useRef(section);
  const keyRef = useRef(key);
  const onSavedRef = useRef(onSaved);
  sectionRef.current = section;
  keyRef.current = key;
  onSavedRef.current = onSaved;

  const performSave = useCallback(async (): Promise<RawAppConfig | null> => {
    const myEpoch = epochRef.current;
    const sentValue = valueRef.current;
    try {
      const next = await invoke<RawAppConfig>('set_config_field', {
        section: sectionRef.current,
        key: keyRef.current,
        value: sentValue,
      });
      if (epochRef.current !== myEpoch) return null;
      lastSavedRef.current = sentValue;
      if (isMountedRef.current) setError(null);
      onSavedRef.current?.(next);
      return next;
    } catch (e) {
      if (epochRef.current !== myEpoch) return null;
      if (isMountedRef.current) setError(e as ConfigError);
      return null;
    }
  }, []);

  const performSaveRef = useRef(performSave);
  performSaveRef.current = performSave;

  useEffect(() => {
    if (areEqual(value, lastSavedRef.current)) return;
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      timerRef.current = null;
      void performSave();
    }, delayMs);
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [value, delayMs, performSave]);

  useEffect(() => {
    return () => {
      isMountedRef.current = false;
      if (timerRef.current) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
        void performSaveRef.current();
      }
    };
  }, []);

  const flushNow = useCallback(async (): Promise<RawAppConfig | null> => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    return performSave();
  }, [performSave]);

  const resetTo = useCallback((next: TValue) => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    epochRef.current += 1;
    valueRef.current = next;
    lastSavedRef.current = next;
    if (isMountedRef.current) setError(null);
  }, []);

  return { error, flushNow, resetTo };
}

function areEqual<T>(a: T, b: T): boolean {
  if (Object.is(a, b)) return true;
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i += 1) {
      if (!Object.is(a[i], b[i])) return false;
    }
    return true;
  }
  return false;
}