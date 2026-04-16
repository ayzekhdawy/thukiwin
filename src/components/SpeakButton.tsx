import { useState, useCallback, useEffect, useRef } from 'react';
import { AnimatePresence, motion } from 'framer-motion';

/** Speaker icon (idle state). */
const SpeakerIcon = (
  <svg
    width="14"
    height="14"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
  >
    <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
    <path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
    <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
  </svg>
);

/** Speaker-off icon (speaking/stop state). */
const SpeakerOffIcon = (
  <svg
    width="14"
    height="14"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
  >
    <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
    <line x1="23" y1="9" x2="17" y2="15" />
    <line x1="17" y1="9" x2="23" y2="15" />
  </svg>
);

interface SpeakButtonProps {
  /** Raw text content to speak. */
  content: string;
  /** Unique ID of the message being spoken (for tracking). */
  messageId: string;
  /** Which side of the action bar the button sits on. */
  align: 'left' | 'right';
  /** Whether this specific message is currently being spoken. */
  isSpeaking: boolean;
  /** Called when the user clicks the speak button to start speaking. */
  onSpeak: (messageId: string, content: string) => void;
  /** Called when the user clicks stop on a playing message. */
  onStop: () => void;
  /** Whether the privacy disclosure has been acknowledged. */
  privacyAcknowledged: boolean;
  /** Called when the user acknowledges the privacy disclosure. */
  onAcknowledgePrivacy: () => void;
}

/**
 * One-click speak button rendered below a chat message.
 * Shows a speaker icon in idle state, and a speaker-off icon while speaking.
 * On first click without privacy acknowledgment, shows a disclosure tooltip.
 */
export function SpeakButton({
  content,
  messageId,
  align,
  isSpeaking,
  onSpeak,
  onStop,
  privacyAcknowledged,
  onAcknowledgePrivacy,
}: SpeakButtonProps) {
  const [showPrivacyTip, setShowPrivacyTip] = useState(false);
  const tipRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleClick = useCallback(() => {
    if (isSpeaking) {
      onStop();
      return;
    }

    if (!privacyAcknowledged) {
      setShowPrivacyTip(true);
      return;
    }

    onSpeak(messageId, content);
  }, [isSpeaking, onStop, privacyAcknowledged, onSpeak, messageId, content]);

  const handleAcknowledge = useCallback(() => {
    onAcknowledgePrivacy();
    setShowPrivacyTip(false);
    onSpeak(messageId, content);
  }, [onAcknowledgePrivacy, onSpeak, messageId, content]);

  const handleDismissPrivacy = useCallback(() => {
    setShowPrivacyTip(false);
  }, []);

  // Auto-hide privacy tooltip after 10 seconds.
  useEffect(() => {
    if (showPrivacyTip) {
      if (tipRef.current) clearTimeout(tipRef.current);
      tipRef.current = setTimeout(() => setShowPrivacyTip(false), 10000);
    }
    return () => {
      if (tipRef.current) clearTimeout(tipRef.current);
    };
  }, [showPrivacyTip]);

  return (
    <div
      className={`relative flex ${align === 'right' ? 'justify-end' : 'justify-start'}`}
    >
      <button
        onClick={handleClick}
        className={`transition-opacity duration-150 p-0.5 rounded cursor-pointer ${
          isSpeaking ? 'text-white/70' : 'text-white/40 hover:text-white/70'
        }`}
        aria-label={isSpeaking ? 'Stop reading' : 'Read aloud'}
      >
        <AnimatePresence mode="wait" initial={false}>
          {isSpeaking ? (
            <motion.span
              key="speaking"
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.8 }}
              transition={{ duration: 0.1 }}
              className="flex"
            >
              {SpeakerOffIcon}
            </motion.span>
          ) : (
            <motion.span
              key="speaker"
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.8 }}
              transition={{ duration: 0.1 }}
              className="flex"
            >
              {SpeakerIcon}
            </motion.span>
          )}
        </AnimatePresence>
      </button>
      {showPrivacyTip && (
        <div className="absolute bottom-full mb-2 left-0 right-0 z-50">
          <div className="bg-white/10 backdrop-blur-sm border border-white/20 rounded-lg px-3 py-2 text-xs text-white/90 max-w-[260px]">
            <p className="mb-2">
              Text will be sent to Microsoft servers for speech synthesis. This
              differs from Thuki&apos;s local-first approach.
            </p>
            <div className="flex gap-2">
              <button
                onClick={handleAcknowledge}
                className="px-2 py-0.5 bg-white/20 hover:bg-white/30 rounded text-xs cursor-pointer"
              >
                OK, read aloud
              </button>
              <button
                onClick={handleDismissPrivacy}
                className="px-2 py-0.5 hover:bg-white/10 rounded text-xs cursor-pointer"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
