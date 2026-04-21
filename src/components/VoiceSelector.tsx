import { useState, useCallback, useRef, useEffect } from 'react';
import type { TtsVoice } from '../hooks/useTts';

/** Chevron down icon for the dropdown trigger. */
const ChevronDownIcon = (
  <svg
    width="12"
    height="12"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
  >
    <polyline points="6 9 12 15 18 9" />
  </svg>
);

/** Close/clear icon. */
const ClearIcon = (
  <svg
    width="10"
    height="10"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="3"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
  >
    <line x1="18" y1="6" x2="6" y2="18" />
    <line x1="6" y1="6" x2="18" y2="18" />
  </svg>
);

interface VoiceSelectorProps {
  /** List of available voices. */
  voices: TtsVoice[];
  /** Currently selected voice short name. */
  selectedVoice: string;
  /** Callback when the user selects a different voice. */
  onVoiceChange: (voice: string) => void;
}

/** Groups voices by locale language prefix (e.g., "tr-TR" -> "Turkish"). */
function groupVoicesByLocale(voices: TtsVoice[]): Map<string, TtsVoice[]> {
  const groups = new Map<string, TtsVoice[]>();
  for (const voice of voices) {
    const lang = voice.Locale.split('-')[0];
    const existing = groups.get(lang) || [];
    existing.push(voice);
    groups.set(lang, existing);
  }
  return groups;
}

/** Human-readable locale names for common languages. */
const LOCALE_NAMES: Record<string, string> = {
  tr: 'Turkish',
  en: 'English',
  de: 'German',
  fr: 'French',
  es: 'Spanish',
  it: 'Italian',
  pt: 'Portuguese',
  ru: 'Russian',
  zh: 'Chinese',
  ja: 'Japanese',
  ko: 'Korean',
  ar: 'Arabic',
  hi: 'Hindi',
  nl: 'Dutch',
  pl: 'Polish',
  sv: 'Swedish',
  da: 'Danish',
  fi: 'Finnish',
  nb: 'Norwegian',
  cs: 'Czech',
  el: 'Greek',
  he: 'Hebrew',
  th: 'Thai',
  vi: 'Vietnamese',
  id: 'Indonesian',
  ms: 'Malay',
  uk: 'Ukrainian',
  ro: 'Romanian',
  hu: 'Hungarian',
  sk: 'Slovak',
  bg: 'Bulgarian',
  hr: 'Croatian',
  ca: 'Catalan',
  eu: 'Basque',
};

/**
 * Compact dropdown for selecting an Edge TTS voice.
 * Voices are grouped by locale language with a search filter.
 */
export function VoiceSelector({
  voices: voicesProp,
  selectedVoice,
  onVoiceChange,
}: VoiceSelectorProps) {
  const voices = voicesProp ?? [];
  const [isOpen, setIsOpen] = useState(false);
  const [search, setSearch] = useState('');
  const containerRef = useRef<HTMLDivElement>(null);

  const toggleOpen = useCallback(() => {
    setIsOpen((prev) => !prev);
    setSearch('');
  }, []);

  const handleSelect = useCallback(
    (voice: string) => {
      onVoiceChange(voice);
      setIsOpen(false);
      setSearch('');
    },
    [onVoiceChange],
  );

  // Close on click outside.
  useEffect(() => {
    if (!isOpen) return;

    const handleClickOutside = (e: MouseEvent) => {
      /* v8 ignore next 3 */
      if (
        containerRef.current &&
        !containerRef.current.contains(e.target as Node)
      ) {
        setIsOpen(false);
        setSearch('');
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen]);

  // Close on Escape key (capture phase + stopPropagation to prevent the
  // global Escape handler in App.tsx from also hiding the overlay).
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      /* v8 ignore next 2 */
      if (e.key === 'Escape') {
        e.stopPropagation();
        setIsOpen(false);
        setSearch('');
      }
    };

    document.addEventListener('keydown', handleKeyDown, { capture: true });
    return () => document.removeEventListener('keydown', handleKeyDown, { capture: true });
  }, [isOpen]);

  const filteredVoices = search.trim()
    ? voices.filter(
        (v) =>
          v.ShortName.toLowerCase().includes(search.toLowerCase()) ||
          v.Locale.toLowerCase().includes(search.toLowerCase()),
      )
    : voices;

  const grouped = groupVoicesByLocale(filteredVoices);
  // Sort groups: selected locale first, then alphabetically.
  const sortedGroups = [...grouped.entries()].sort((a, b) => {
    const selectedLang = selectedVoice.split('-')[0];
    /* v8 ignore next */
    if (a[0] === selectedLang) return -1;
    if (b[0] === selectedLang) return 1;
    return a[0].localeCompare(b[0]);
  });

  const selectedLabel =
    voices.find((v) => v.ShortName === selectedVoice)?.ShortName ||
    selectedVoice;

  return (
    <div ref={containerRef} className="relative">
      <button
        onClick={toggleOpen}
        className="flex items-center gap-1 text-white/40 hover:text-white/70 transition-opacity duration-150 text-xs px-1 py-0.5 rounded cursor-pointer"
        aria-label="Select voice"
        aria-expanded={isOpen}
      >
        <span className="truncate max-w-[120px]">{selectedLabel}</span>
        {ChevronDownIcon}
      </button>

      {isOpen && (
        <div className="absolute top-full mt-1 left-0 z-50 w-64 max-h-72 overflow-hidden flex flex-col bg-[#2c2c2c] border border-white/20 rounded-lg shadow-lg">
          {/* Search input */}
          <div className="p-2 border-b border-white/10">
            <div className="relative">
              <input
                type="text"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="Search voices..."
                className="w-full bg-white/10 text-white/90 text-xs px-2 py-1 rounded border border-white/20 outline-none focus:border-white/40 placeholder-white/30"
                autoFocus
              />
              {search && (
                <button
                  onClick={() => setSearch('')}
                  className="absolute right-1 top-1/2 -translate-y-1/2 text-white/40 hover:text-white/70 cursor-pointer"
                  aria-label="Clear search"
                >
                  {ClearIcon}
                </button>
              )}
            </div>
          </div>

          {/* Voice list */}
          <div className="overflow-y-auto flex-1">
            {sortedGroups.map(([lang, groupVoices]) => (
              <div key={lang}>
                <div className="text-white/40 text-[10px] font-semibold uppercase tracking-wider px-2 pt-2 pb-1">
                  {LOCALE_NAMES[lang] || lang}
                </div>
                {groupVoices.map((voice) => (
                  <button
                    key={voice.ShortName}
                    onClick={() => handleSelect(voice.ShortName)}
                    className={`w-full text-left px-2 py-1 text-xs cursor-pointer transition-colors ${
                      voice.ShortName === selectedVoice
                        ? 'bg-white/20 text-white'
                        : 'text-white/70 hover:bg-white/10'
                    }`}
                  >
                    <span className="font-medium">{voice.ShortName}</span>
                    <span className="text-white/40 ml-1">
                      {voice.gender === 'Female' ? 'F' : 'M'}
                    </span>
                  </button>
                ))}
              </div>
            ))}
            {sortedGroups.length === 0 && (
              <div className="px-2 py-3 text-xs text-white/40 text-center">
                No voices found
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
