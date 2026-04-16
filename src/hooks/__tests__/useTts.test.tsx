import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useTts } from '../useTts';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

describe('useTts', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    mockInvoke.mockResolvedValue([]);
  });

  it('initializes with default voice', () => {
    const { result } = renderHook(() => useTts());
    expect(result.current.selectedVoice).toBe('tr-TR-EmelNeural');
  });

  it('initializes with stored voice', () => {
    localStorage.setItem('tts_voice', 'en-US-JennyNeural');
    const { result } = renderHook(() => useTts());
    expect(result.current.selectedVoice).toBe('en-US-JennyNeural');
  });

  it('initializes with privacy not acknowledged', () => {
    const { result } = renderHook(() => useTts());
    expect(result.current.privacyAcknowledged).toBe(false);
  });

  it('initializes with privacy acknowledged from localStorage', () => {
    localStorage.setItem('tts_privacy_acknowledged', 'true');
    const { result } = renderHook(() => useTts());
    expect(result.current.privacyAcknowledged).toBe(true);
  });

  it('sets selected voice and persists to localStorage', () => {
    const { result } = renderHook(() => useTts());
    act(() => {
      result.current.setSelectedVoice('en-US-JennyNeural');
    });
    expect(result.current.selectedVoice).toBe('en-US-JennyNeural');
    expect(localStorage.getItem('tts_voice')).toBe('en-US-JennyNeural');
  });

  it('acknowledges privacy and persists', () => {
    const { result } = renderHook(() => useTts());
    act(() => {
      result.current.acknowledgePrivacy();
    });
    expect(result.current.privacyAcknowledged).toBe(true);
    expect(localStorage.getItem('tts_privacy_acknowledged')).toBe('true');
  });

  it('starts not speaking', () => {
    const { result } = renderHook(() => useTts());
    expect(result.current.isSpeaking).toBe(false);
    expect(result.current.speakingMessageId).toBeNull();
  });

  it('invokes tts_stop on stop', () => {
    const { result } = renderHook(() => useTts());
    act(() => {
      result.current.stop();
    });
    expect(mockInvoke).toHaveBeenCalledWith('tts_stop');
  });

  it('fetches voices and stores them', async () => {
    const fakeVoices = [
      {
        name: 'Test',
        ShortName: 'en-US-Test',
        gender: 'Female',
        Locale: 'en-US',
        SuggestedCodec: 'audio-24khz-48kbitrate-mono-mp3',
      },
    ];
    mockInvoke.mockResolvedValueOnce(fakeVoices);
    const { result } = renderHook(() => useTts());
    let voices: unknown;
    await act(async () => {
      voices = await result.current.fetchVoices();
    });
    expect(voices).toEqual(fakeVoices);
    expect(result.current.voices).toEqual(fakeVoices);
  });

  it('returns empty array on fetch error', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('Network error'));
    const { result } = renderHook(() => useTts());
    let voices: unknown;
    await act(async () => {
      voices = await result.current.fetchVoices();
    });
    expect(voices).toEqual([]);
  });

  it('handles non-array voice response', async () => {
    mockInvoke.mockResolvedValueOnce(null);
    const { result } = renderHook(() => useTts());
    let voices: unknown;
    await act(async () => {
      voices = await result.current.fetchVoices();
    });
    expect(voices).toEqual([]);
  });

  it('speaks text via tts_speak', async () => {
    // Mock window.Audio to prevent real playback
    const mockAudio = {
      play: vi.fn().mockResolvedValue(undefined),
      onended: null,
      onerror: null,
      src: '',
    };
    vi.stubGlobal(
      'Audio',
      vi.fn(() => mockAudio),
    );
    // Mock URL.createObjectURL and revokeObjectURL
    vi.stubGlobal('URL', {
      ...URL,
      createObjectURL: vi.fn(() => 'blob:test'),
      revokeObjectURL: vi.fn(),
    });

    const base64Mp3 = btoa('fake audio data');
    mockInvoke.mockResolvedValueOnce(base64Mp3);

    const { result } = renderHook(() => useTts());
    await act(async () => {
      await result.current.speak('msg1', 'Hello world');
    });

    expect(mockInvoke).toHaveBeenCalledWith('tts_speak', {
      text: 'Hello world',
      voice: 'tr-TR-EmelNeural',
      rate: '+0%',
      pitch: '+0%',
    });

    vi.unstubAllGlobals();
  });
});