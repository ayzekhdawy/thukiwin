import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useAgentMode } from '../useAgentMode';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// These are the project's vitest-alias mocks from testUtils/mocks/tauri.ts
const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;
const mockListen = listen as unknown as ReturnType<typeof vi.fn>;

describe('useAgentMode', () => {
  beforeEach(() => {
    // Default: get_ollama_url succeeds, start_agent_mode succeeds, listen succeeds.
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_ollama_url') return Promise.resolve('http://127.0.0.1:11434');
      return Promise.resolve(undefined);
    });
    mockListen.mockResolvedValue(vi.fn());
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('should start in idle state', () => {
    const { result } = renderHook(() => useAgentMode(null));
    expect(result.current.isActive).toBe(false);
    expect(result.current.status).toBe('idle');
    expect(result.current.lastAction).toBeNull();
    expect(result.current.lastResult).toBeNull();
    expect(result.current.reasoning).toBeNull();
    expect(result.current.screenshotUrl).toBeNull();
  });

  it('should expose start and stop functions', () => {
    const { result } = renderHook(() => useAgentMode(null));
    expect(typeof result.current.start).toBe('function');
    expect(typeof result.current.stop).toBe('function');
  });

  it('should set error state when start_agent_mode rejects', async () => {
    // get_ollama_url succeeds, but start_agent_mode fails.
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_ollama_url') return Promise.resolve('http://127.0.0.1:11434');
      if (cmd === 'start_agent_mode') return Promise.reject(new Error('Failed'));
      return Promise.resolve(undefined);
    });

    const { result } = renderHook(() => useAgentMode(null));

    await act(async () => {
      await result.current.start('Test');
    });

    expect(result.current.status).toBe('error');
    expect(result.current.isActive).toBe(false);
  });

  it('should transition to idle when stopped', async () => {
    const { result } = renderHook(() => useAgentMode(null));

    await act(async () => {
      await result.current.start('Test');
    });

    await act(async () => {
      await result.current.stop();
    });

    expect(result.current.isActive).toBe(false);
    expect(result.current.status).toBe('idle');
  });

  it('should clear previous state on start', async () => {
    const { result } = renderHook(() => useAgentMode(null));

    await act(async () => {
      await result.current.start('Test task');
    });

    await act(async () => {
      await result.current.start('New task');
    });

    expect(result.current.lastAction).toBeNull();
    expect(result.current.reasoning).toBeNull();
    expect(result.current.screenshotUrl).toBeNull();
  });
});