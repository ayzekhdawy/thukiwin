/**
 * Top-level component for the Settings window.
 *
 * 7 tabs: AI, Web, Display, Agent, Gateway, Sound, About.
 * Uses Ctrl+, and Ctrl+W keyboard shortcuts (Windows convention).
 */

import {
  type MouseEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from 'react';

import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

import { useConfigSync } from './hooks/useConfigSync';
import { useSettingsAutoResize } from './hooks/useSettingsAutoResize';
import { ModelTab } from './tabs/ModelTab';
import { SearchTab } from './tabs/SearchTab';
import { DisplayTab } from './tabs/DisplayTab';
import { AgentTab } from './tabs/AgentTab';
import { GatewayTab } from './tabs/GatewayTab';
import { SoundTab } from './tabs/SoundTab';
import { AboutTab } from './tabs/AboutTab';
import { SavedPill } from './components';
import styles from '../styles/settings.module.css';
import type { CorruptMarker, RawAppConfig, SettingsTabId } from './types';

const TABS: ReadonlyArray<{
  id: SettingsTabId;
  label: string;
  icon: ReactNode;
}> = [
  {
    id: 'general',
    label: 'AI',
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
        <path d="M9.5 2a3 3 0 0 0-3 3v.5a2.5 2.5 0 0 0-2 4 3 3 0 0 0 .5 5 2.5 2.5 0 0 0 1.5 4.5 3 3 0 0 0 5.5-1.5V5a3 3 0 0 0-2.5-3z" />
        <path d="M14.5 2a3 3 0 0 1 3 3v.5a2.5 2.5 0 0 1 2 4 3 3 0 0 1-.5 5 2.5 2.5 0 0 1-1.5 4.5 3 3 0 0 1-5.5-1.5V5a3 3 0 0 1 2.5-3z" />
      </svg>
    ),
  },
  {
    id: 'search',
    label: 'Web',
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
        <circle cx="12" cy="12" r="10" />
        <line x1="2" y1="12" x2="22" y2="12" />
        <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
      </svg>
    ),
  },
  {
    id: 'display',
    label: 'Display',
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
        <rect x="2" y="3" width="20" height="14" rx="2" />
        <line x1="8" y1="21" x2="16" y2="21" />
        <line x1="12" y1="17" x2="12" y2="21" />
      </svg>
    ),
  },
  {
    id: 'agent',
    label: 'Agent',
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
        <path d="M12 2a5 5 0 0 1 5 5v1a5 5 0 0 1-10 0V7a5 5 0 0 1 5-5z" />
        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
      </svg>
    ),
  },
  {
    id: 'gateway',
    label: 'Gateway',
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
        <rect x="2" y="2" width="20" height="8" rx="2" />
        <rect x="2" y="14" width="20" height="8" rx="2" />
        <line x1="6" y1="6" x2="6.01" y2="6" />
        <line x1="6" y1="18" x2="6.01" y2="18" />
      </svg>
    ),
  },
  {
    id: 'sound',
    label: 'Sound',
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
        <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
        <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
        <path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
      </svg>
    ),
  },
  {
    id: 'about',
    label: 'About',
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="16" x2="12" y2="12" />
        <line x1="12" y1="8" x2="12.01" y2="8" />
      </svg>
    ),
  },
];

const SAVED_PILL_DURATION_MS = 1500;
const CHROME_HEIGHT = 148;
const BANNER_HEIGHT = 56;

export function SettingsWindow() {
  const { config, reload, setConfig } = useConfigSync();
  const [activeTab, setActiveTab] = useState<SettingsTabId>('general');
  const [savedVisible, setSavedVisible] = useState(false);
  const [marker, setMarker] = useState<CorruptMarker | null>(null);
  const [markerDismissed, setMarkerDismissed] = useState(false);

  const [resyncToken, setResyncToken] = useState(0);

  const savedTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [contentEl, setContentEl] = useState<HTMLDivElement | null>(null);

  const bannerVisible = Boolean(marker && !markerDismissed);
  const bodyShouldScroll = useSettingsAutoResize(
    contentEl,
    CHROME_HEIGHT + (bannerVisible ? BANNER_HEIGHT : 0),
    activeTab,
  );

  const handleSaved = useCallback(
    (next: RawAppConfig) => {
      setConfig(next);
      setResyncToken((prev) => prev + 1);
      setSavedVisible(true);
      if (savedTimerRef.current) clearTimeout(savedTimerRef.current);
      savedTimerRef.current = setTimeout(() => {
        setSavedVisible(false);
        savedTimerRef.current = null;
      }, SAVED_PILL_DURATION_MS);
    },
    [setConfig],
  );

  useEffect(
    () => () => {
      if (savedTimerRef.current) clearTimeout(savedTimerRef.current);
    },
    [],
  );

  useEffect(() => {
    void invoke<CorruptMarker | null>('get_corrupt_marker').then((m) => {
      if (m) setMarker(m);
    });
  }, []);

  // Ctrl+, and Ctrl+W keyboard shortcuts (Windows convention)
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === ',') {
        e.preventDefault();
        void getCurrentWindow().setFocus();
      }
      if (e.ctrlKey && e.key === 'w') {
        e.preventDefault();
        void getCurrentWindow().hide();
      }
    };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, []);

  const handleHide = useCallback(() => {
    void getCurrentWindow().hide();
  }, []);

  const handleDragStart = useCallback((e: MouseEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;
    const el = e.target as HTMLElement;

    const INTERACTIVE_TAGS = new Set([
      'TEXTAREA',
      'INPUT',
      'BUTTON',
      'A',
      'SELECT',
      'PATH',
      'SVG',
      'LABEL',
    ]);
    let current: HTMLElement | null = el;
    while (current) {
      if (INTERACTIVE_TAGS.has(current.tagName.toUpperCase())) return;
      current = current.parentElement;
    }

    for (const node of Array.from(el.childNodes)) {
      if (
        node.nodeType === Node.TEXT_NODE &&
        node.textContent &&
        node.textContent.trim().length > 0
      ) {
        return;
      }
    }

    e.preventDefault();
    void getCurrentWindow().startDragging();
  }, []);

  if (!config) return null;

  return (
    <div className={styles.window} onMouseDown={handleDragStart}>
      {/* Close button for Windows */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'flex-end',
          padding: '4px 8px 0',
          flexShrink: 0,
        }}
      >
        <button
          type="button"
          onClick={handleHide}
          style={{
            width: 28,
            height: 28,
            border: 'none',
            background: 'transparent',
            color: 'var(--color-text-secondary)',
            cursor: 'pointer',
            borderRadius: 4,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 14,
          }}
          aria-label="Close settings"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
            <path
              d="M1 1L9 9M9 1L1 9"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
            />
          </svg>
        </button>
      </div>

      {marker && !markerDismissed ? (
        <div className={styles.banner} role="alert">
          <span className={styles.bannerIcon} aria-hidden>
            !
          </span>
          <span className={styles.bannerText}>
            Your previous <code>config.toml</code> had a syntax error and was
            saved as <code>{baseName(marker.path)}</code>. Defaults are now
            active.
          </span>
          <span className={styles.bannerActions}>
            <button
              type="button"
              className={`${styles.button} ${styles.buttonGhost}`}
              onClick={() => void invoke('reveal_config_in_explorer')}
            >
              Reveal
            </button>
            <button
              type="button"
              className={`${styles.button} ${styles.buttonGhost}`}
              onClick={() => setMarkerDismissed(true)}
            >
              Dismiss
            </button>
          </span>
        </div>
      ) : null}

      <div
        role="tablist"
        aria-label="Settings sections"
        className={styles.tabBar}
      >
        {TABS.map((tab) => {
          const active = tab.id === activeTab;
          return (
            <button
              key={tab.id}
              type="button"
              role="tab"
              aria-selected={active}
              aria-controls={`panel-${tab.id}`}
              tabIndex={active ? 0 : -1}
              className={`${styles.tab} ${active ? styles.tabActive : ''}`}
              onClick={() => setActiveTab(tab.id)}
              onKeyDown={(e) => {
                if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') {
                  e.preventDefault();
                  const idx = TABS.findIndex((t) => t.id === activeTab);
                  const next =
                    e.key === 'ArrowRight'
                      ? TABS[(idx + 1) % TABS.length]
                      : TABS[(idx - 1 + TABS.length) % TABS.length];
                  setActiveTab(next.id);
                }
              }}
            >
              <span className={styles.tabIcon} aria-hidden>
                {tab.icon}
              </span>
              <span className={styles.tabLabel}>{tab.label}</span>
            </button>
          );
        })}
      </div>

      <div
        className={`${styles.body} ${bodyShouldScroll ? styles.bodyScrollable : ''}`}
        id={`panel-${activeTab}`}
        role="tabpanel"
      >
        <div ref={setContentEl}>
          {activeTab === 'general' ? (
            <ModelTab config={config} resyncToken={resyncToken} onSaved={handleSaved} />
          ) : null}
          {activeTab === 'search' ? (
            <SearchTab config={config} resyncToken={resyncToken} onSaved={handleSaved} />
          ) : null}
          {activeTab === 'display' ? (
            <DisplayTab config={config} resyncToken={resyncToken} onSaved={handleSaved} />
          ) : null}
          {activeTab === 'agent' ? (
            <AgentTab config={config} resyncToken={resyncToken} onSaved={handleSaved} />
          ) : null}
          {activeTab === 'gateway' ? (
            <GatewayTab config={config} resyncToken={resyncToken} onSaved={handleSaved} />
          ) : null}
          {activeTab === 'sound' ? (
            <SoundTab config={config} resyncToken={resyncToken} onSaved={handleSaved} />
          ) : null}
          {activeTab === 'about' ? (
            <AboutTab onSaved={handleSaved} onReload={reload} />
          ) : null}
        </div>
      </div>

      <SavedPill visible={savedVisible} />
    </div>
  );
}

function baseName(path: string): string {
  const idx = path.lastIndexOf(/[/\\]/.test(path) ? (path.includes('\\') ? '\\' : '/') : '/');
  return idx >= 0 ? path.slice(idx + 1) : path;
}