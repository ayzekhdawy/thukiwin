/**
 * About tab - app identity, version, links, file actions.
 *
 * Windows-adapted: no macOS permissions (Accessibility/Screen Recording),
 * uses Windows-style file reveal.
 */

import { useState } from 'react';

import { invoke } from '@tauri-apps/api/core';

import thukiLogo from '../../../src-tauri/icons/128x128.png';
import pkg from '../../../package.json';
import { Section, ConfirmDialog } from '../components';
import styles from '../../styles/settings.module.css';
import type { RawAppConfig } from '../types';

interface AboutTabProps {
  onSaved: (next: RawAppConfig) => void;
  onReload: () => Promise<void>;
}

export function AboutTab({ onSaved, onReload }: AboutTabProps) {
  const sha = import.meta.env.VITE_GIT_COMMIT_SHA?.slice(0, 7);
  const APP_VERSION = sha ? `${pkg.version}+nightly.${sha}` : pkg.version;
  const releaseUrl = `https://github.com/ayzekhdawy/thukiwin/releases/tag/v${pkg.version}`;
  const [confirmResetAll, setConfirmResetAll] = useState(false);

  return (
    <div className={styles.aboutBody}>
      <div className={styles.aboutHero}>
        <img
          src={thukiLogo}
          alt="ThukiWin"
          className={styles.aboutHeroLogo}
          draggable={false}
        />
        <div className={styles.aboutHeroTitle}>ThukiWin</div>
        <button
          type="button"
          className={styles.aboutHeroVersion}
          aria-label={`View v${APP_VERSION} release notes on GitHub`}
          onClick={() => void invoke('open_url', { url: releaseUrl })}
        >
          v{APP_VERSION}
        </button>
        <div className={styles.aboutHeroTagline}>
          A floating, local-first AI secretary for Windows.
          <br />
          <span className={styles.aboutHeroMantra}>
            No cloud. No clutter. Just answers.
          </span>
        </div>
        <div className={styles.aboutHeroActions}>
          <button
            type="button"
            className={styles.iconLinkBtn}
            aria-label="View ThukiWin on GitHub"
            onClick={() =>
              void invoke('open_url', {
                url: 'https://github.com/ayzekhdawy/thukiwin',
              })
            }
          >
            <GitHubIcon />
          </button>
          <button
            type="button"
            className={styles.iconLinkBtn}
            aria-label="Open an issue or share feedback on GitHub"
            onClick={() =>
              void invoke('open_url', {
                url: 'https://github.com/ayzekhdawy/thukiwin/issues',
              })
            }
          >
            <FeedbackIcon />
          </button>
        </div>
      </div>

      <Section heading="File">
        <div className={styles.aboutLinkRow}>
          <button
            type="button"
            className={`${styles.button} ${styles.buttonGhost}`}
            onClick={() => void invoke('reveal_config_in_explorer')}
          >
            Reveal ThukiWin app data
          </button>
          <button
            type="button"
            className={`${styles.button} ${styles.buttonGhost}`}
            onClick={() => void onReload()}
          >
            Refresh config.toml
          </button>
          <button
            type="button"
            className={`${styles.button} ${styles.buttonDestructive}`}
            onClick={() => setConfirmResetAll(true)}
          >
            Reset all to defaults...
          </button>
        </div>
      </Section>

      <ConfirmDialog
        open={confirmResetAll}
        title="Reset all settings to defaults?"
        message="Your entire config.toml will be replaced with the defaults. This cannot be undone."
        confirmLabel="Reset all"
        destructive
        onConfirm={() => {
          setConfirmResetAll(false);
          void invoke<RawAppConfig>('reset_config', { section: null }).then(
            onSaved,
          );
        }}
        onCancel={() => setConfirmResetAll(false)}
      />
    </div>
  );
}

function GitHubIcon() {
  return (
    <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor" aria-hidden>
      <path d="M12 .5C5.65.5.5 5.65.5 12c0 5.08 3.29 9.39 7.86 10.91.58.11.79-.25.79-.56 0-.27-.01-1-.02-1.96-3.2.7-3.87-1.54-3.87-1.54-.52-1.32-1.27-1.67-1.27-1.67-1.04-.71.08-.7.08-.7 1.15.08 1.76 1.18 1.76 1.18 1.02 1.75 2.68 1.24 3.34.95.1-.74.4-1.24.72-1.53-2.55-.29-5.24-1.28-5.24-5.69 0-1.26.45-2.29 1.18-3.1-.12-.29-.51-1.46.11-3.05 0 0 .96-.31 3.15 1.18a10.96 10.96 0 0 1 5.74 0c2.19-1.49 3.15-1.18 3.15-1.18.62 1.59.23 2.76.11 3.05.74.81 1.18 1.84 1.18 3.1 0 4.42-2.7 5.4-5.27 5.68.41.36.78 1.06.78 2.13 0 1.54-.01 2.78-.01 3.16 0 .31.21.68.8.56C20.21 21.39 23.5 17.08 23.5 12 23.5 5.65 18.35.5 12 .5z" />
    </svg>
  );
}

function FeedbackIcon() {
  return (
    <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z" />
    </svg>
  );
}