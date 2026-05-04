/**
 * Sound tab - notification sound and TTS voice settings.
 */

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

import { Section, NumberSlider } from '../components';
import { SaveField } from '../components/SaveField';
import { configHelp } from '../configHelpers';
import styles from '../../styles/settings.module.css';
import type { RawAppConfig } from '../types';

type NotificationSound = 'system' | 'custom' | 'none';

const NOTIFICATION_OPTIONS: NotificationSound[] = ['system', 'custom', 'none'];
const NOTIFICATION_LABELS: Record<NotificationSound, string> = {
  system: 'System Default',
  custom: 'Custom Sound',
  none: 'Silent',
};

interface SoundTabProps {
  config: RawAppConfig;
  resyncToken: number;
  onSaved: (next: RawAppConfig) => void;
}

export function SoundTab({ config, resyncToken, onSaved }: SoundTabProps) {
  const [ttsVoices, setTtsVoices] = useState<
    { name: string; ShortName: string; Locale: string; gender: string }[]
  >([]);
  const [notificationSound, setNotificationSound] =
    useState<NotificationSound>('system');

  useEffect(() => {
    async function loadVoices() {
      try {
        const voices = await invoke<
          Array<{ name: string; ShortName: string; Locale: string; gender: string }>
        >('tts_list_voices');
        setTtsVoices(voices);
      } catch {
        // TTS not available
      }
    }
    void loadVoices();

    async function loadNotificationSetting() {
      try {
        const settings = await invoke<Record<string, string>>('get_settings');
        if (settings['notification_sound']) {
          setNotificationSound(settings['notification_sound'] as NotificationSound);
        }
      } catch {
        // use default
      }
    }
    void loadNotificationSetting();
  }, []);

  async function saveNotificationSound(value: NotificationSound) {
    setNotificationSound(value);
    try {
      await invoke('set_setting', {
        key: 'notification_sound',
        value,
      });
    } catch {
      // ignore
    }
  }

  const voicesByLocale = ttsVoices.reduce<
    Record<string, typeof ttsVoices>
  >((acc, v) => {
    const locale = v.Locale;
    if (!acc[locale]) acc[locale] = [];
    acc[locale].push(v);
    return acc;
  }, {});
  const sortedLocales = Object.keys(voicesByLocale).sort();

  return (
    <>
      <Section heading="Notifications">
        <div className={styles.row}>
          <div className={styles.rowLabelGroup}>
            <span className={styles.rowLabel}>Notification sound</span>
          </div>
          <div className={styles.rowControl}>
            <select
              className={styles.dropdown}
              value={notificationSound}
              aria-label="Notification sound"
              onChange={(e) =>
                saveNotificationSound(e.target.value as NotificationSound)
              }
            >
              {NOTIFICATION_OPTIONS.map((opt) => (
                <option key={opt} value={opt}>
                  {NOTIFICATION_LABELS[opt]}
                </option>
              ))}
            </select>
          </div>
        </div>
      </Section>

      <Section heading="Text-to-Speech">
        <SaveField
          section="tts"
          fieldKey="voice"
          label="Voice"
          helper={configHelp('tts', 'voice')}
          initialValue={config.tts.voice}
          resyncToken={resyncToken}
          onSaved={onSaved}
          render={(value, setValue, errored) =>
            sortedLocales.length > 0 ? (
              <select
                className={styles.dropdown}
                value={value}
                aria-label="TTS voice"
                onChange={(e) => setValue(e.target.value)}
              >
                {sortedLocales.map((locale) => (
                  <optgroup key={locale} label={locale}>
                    {voicesByLocale[locale].map((v) => (
                      <option key={v.ShortName} value={v.ShortName}>
                        {v.ShortName} ({v.gender})
                      </option>
                    ))}
                  </optgroup>
                ))}
              </select>
            ) : (
              <input
                type="text"
                className={`${styles.input} ${errored ? styles.inputError : ''}`}
                value={value}
                onChange={(e) => setValue(e.target.value)}
                aria-label="TTS voice"
                spellCheck={false}
              />
            )
          }
        />
        <SaveField
          section="tts"
          fieldKey="rate"
          label="Speed"
          helper={configHelp('tts', 'rate')}
          initialValue={config.tts.rate}
          resyncToken={resyncToken}
          onSaved={onSaved}
          render={(value, setValue) => (
            <NumberSlider
              value={value}
              min={-50}
              max={50}
              unit=""
              onChange={setValue}
              ariaLabel="TTS speed"
            />
          )}
        />
        <SaveField
          section="tts"
          fieldKey="pitch"
          label="Pitch"
          helper={configHelp('tts', 'pitch')}
          initialValue={config.tts.pitch}
          resyncToken={resyncToken}
          onSaved={onSaved}
          render={(value, setValue) => (
            <NumberSlider
              value={value}
              min={-50}
              max={50}
              unit=""
              onChange={setValue}
              ariaLabel="TTS pitch"
            />
          )}
        />
      </Section>
    </>
  );
}