import { useState, useCallback, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

/** Voice returned from the Edge TTS voices endpoint. */
export interface TtsVoice {
  name: string;
  ShortName: string;
  gender: string;
  Locale: string;
  SuggestedCodec: string;
}

/** Return type of the useTts hook. */
export interface UseTtsReturn {
  /** Whether TTS audio is currently playing. */
  isSpeaking: boolean;
  /** The message ID currently being spoken, if any. */
  speakingMessageId: string | null;
  /** Starts speaking the given text. If already speaking, stops first. */
  speak: (messageId: string, text: string, voice?: string) => Promise<void>;
  /** Stops any currently playing TTS audio. */
  stop: () => void;
  /** Fetches available voices from the Edge TTS endpoint. */
  fetchVoices: () => Promise<TtsVoice[]>;
  /** Cached list of available voices. */
  voices: TtsVoice[];
  /** Currently selected voice short name, persisted in localStorage. */
  selectedVoice: string;
  /** Changes the selected voice and persists to localStorage. */
  setSelectedVoice: (voice: string) => void;
  /** Whether a privacy disclosure has been acknowledged. */
  privacyAcknowledged: boolean;
  /** Marks the privacy disclosure as acknowledged. */
  acknowledgePrivacy: () => void;
}

const STORAGE_KEY_VOICE = 'tts_voice';
const STORAGE_KEY_PRIVACY = 'tts_privacy_acknowledged';
const DEFAULT_VOICE = 'tr-TR-EmelNeural';

/** Decodes a base64 string to an ArrayBuffer. */
function base64ToArrayBuffer(base64: string): ArrayBuffer {
  const binaryString = atob(base64);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes.buffer;
}

/**
 * Custom hook for text-to-speech via Edge TTS.
 *
 * Manages audio playback lifecycle: invokes the Rust backend to synthesize
 * text, decodes the base64 MP3 response into a Blob URL, and plays it via
 * the HTML5 Audio API. Supports stop/cancel and voice selection.
 */
export function useTts(): UseTtsReturn {
  const [isSpeaking, setIsSpeaking] = useState(false);
  const [speakingMessageId, setSpeakingMessageId] = useState<string | null>(
    null,
  );
  const [voices, setVoices] = useState<TtsVoice[]>([]);
  const [selectedVoice, setSelectedVoiceState] = useState<string>(() => {
    return localStorage.getItem(STORAGE_KEY_VOICE) || DEFAULT_VOICE;
  });
  const [privacyAcknowledged, setPrivacyAcknowledged] = useState<boolean>(
    () => {
      return localStorage.getItem(STORAGE_KEY_PRIVACY) === 'true';
    },
  );

  const audioRef = useRef<HTMLAudioElement | null>(null);
  const blobUrlRef = useRef<string | null>(null);

  /** Stops any currently playing audio and cleans up resources. */
  const cleanup = useCallback(() => {
    if (audioRef.current) {
      audioRef.current.pause();
      audioRef.current.src = '';
      audioRef.current = null;
    }
    if (blobUrlRef.current) {
      URL.revokeObjectURL(blobUrlRef.current);
      blobUrlRef.current = null;
    }
    setIsSpeaking(false);
    setSpeakingMessageId(null);
  }, []);

  /** Starts speaking the given text with the specified (or default) voice. */
  const speak = useCallback(
    async (messageId: string, text: string, voice?: string) => {
      // If already speaking, stop first.
      cleanup();

      const voiceToUse = voice || selectedVoice;

      try {
        setIsSpeaking(true);
        setSpeakingMessageId(messageId);

        const base64Audio: string = await invoke('tts_speak', {
          text,
          voice: voiceToUse,
          rate: '+0%',
          pitch: '+0%',
        });

        // If we were cancelled while waiting, bail out.
        if (!base64Audio) {
          cleanup();
          return;
        }

        const arrayBuffer = base64ToArrayBuffer(base64Audio);
        const blob = new Blob([arrayBuffer], { type: 'audio/mpeg' });
        const url = URL.createObjectURL(blob);
        blobUrlRef.current = url;

        const audio = new Audio(url);
        audioRef.current = audio;

        audio.onended = () => {
          cleanup();
        };

        audio.onerror = () => {
          cleanup();
        };

        await audio.play();
      } catch (e) {
        // "cancelled" error means the user stopped TTS; not an actual error.
        if (e instanceof Error && e.message === 'cancelled') {
          // Expected — do nothing.
        } else if (typeof e === 'string' && e === 'cancelled') {
          // Tauri returns errors as strings.
        } else {
          console.error('TTS error:', e);
        }
        cleanup();
      }
    },
    [cleanup, selectedVoice],
  );

  /** Stops any currently playing TTS audio and cancels backend synthesis. */
  const stop = useCallback(() => {
    void invoke('tts_stop');
    cleanup();
  }, [cleanup]);

  /** Fetches the list of available Edge TTS voices. */
  const fetchVoices = useCallback(async (): Promise<TtsVoice[]> => {
    try {
      const result = await invoke('tts_list_voices');
      const voices = Array.isArray(result) ? result : [];
      setVoices(voices);
      return voices;
    } catch (e) {
      console.error('Failed to fetch TTS voices:', e);
      return [];
    }
  }, []);

  /** Changes the selected voice and persists to localStorage. */
  const setSelectedVoice = useCallback((voice: string) => {
    localStorage.setItem(STORAGE_KEY_VOICE, voice);
    setSelectedVoiceState(voice);
  }, []);

  /** Marks the privacy disclosure as acknowledged. */
  const acknowledgePrivacy = useCallback(() => {
    localStorage.setItem(STORAGE_KEY_PRIVACY, 'true');
    setPrivacyAcknowledged(true);
  }, []);

  // Cleanup on unmount.
  useEffect(() => {
    return () => {
      if (audioRef.current) {
        audioRef.current.pause();
        audioRef.current.src = '';
      }
      if (blobUrlRef.current) {
        URL.revokeObjectURL(blobUrlRef.current);
      }
    };
  }, []);

  return {
    isSpeaking,
    speakingMessageId,
    speak,
    stop,
    fetchVoices,
    voices,
    selectedVoice,
    setSelectedVoice,
    privacyAcknowledged,
    acknowledgePrivacy,
  };
}
