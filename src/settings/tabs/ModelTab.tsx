/**
 * AI tab - Ollama endpoint, keep-warm controls, and system prompt.
 */

import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import { Section, TextField, Textarea } from '../components';
import { SaveField } from '../components/SaveField';
import { useDebouncedSave } from '../hooks/useDebouncedSave';
import { configHelp } from '../configHelpers';
import styles from '../../styles/settings.module.css';
import type { RawAppConfig } from '../types';

interface ModelTabProps {
  config: RawAppConfig;
  resyncToken: number;
  onSaved: (next: RawAppConfig) => void;
}

const PROMPT_MAX_CHARS = 8000;
const EJECT_RESET_MS = 2500;
const TOKENS_PER_TURN_ESTIMATE = 400;

const KEEP_WARM_TOOLTIP =
  'Keep Warm holds your active model loaded in VRAM after each use. ' +
  'The timer below sets how long before it auto-releases; use -1 to keep it indefinitely. ' +
  'Unload now releases it immediately. ' +
  'If set to 0, Ollama unloads models after its default 5-minute timeout.';

const CTX_MIN = 2048;
const CTX_MAX = 1_048_576;
const CTX_LOG_RATIO = Math.log(CTX_MAX / CTX_MIN);

function ctxToPos(v: number): number {
  return Math.round((1000 * Math.log(v / CTX_MIN)) / CTX_LOG_RATIO);
}

function posToCtx(pos: number): number {
  return (
    Math.round((CTX_MIN * Math.pow(CTX_MAX / CTX_MIN, pos / 1000)) / 1024) *
    1024
  );
}

const CTX_TICKS = [
  '2K', '4K', '8K', '16K', '32K', '64K', '128K', '256K', '512K', '1M',
];

export function ModelTab({ config, resyncToken, onSaved }: ModelTabProps) {
  const [inactivityMin, setInactivityMin] = useState(
    config.inference.keep_warm_inactivity_minutes,
  );
  const [ejecting, setEjecting] = useState(false);
  const [loadedModel, setLoadedModel] = useState<string | null>(null);

  const [numCtx, setNumCtx] = useState(config.inference.num_ctx);
  const [ctxPos, setCtxPos] = useState(() =>
    ctxToPos(config.inference.num_ctx),
  );
  const [ctxChip, setCtxChip] = useState(String(config.inference.num_ctx));
  const ctxDraggingRef = useRef(false);

  useEffect(() => {
    let unlistenLoaded: (() => void) | null = null;
    let unlistenEvicted: (() => void) | null = null;

    async function setup() {
      unlistenLoaded = await listen<string>('warmup:model-loaded', (e) => {
        setLoadedModel(e.payload);
      });
      unlistenEvicted = await listen<null>('warmup:model-evicted', () => {
        setLoadedModel(null);
      });
      invoke<string | null>('get_loaded_model')
        .then(setLoadedModel)
        .catch(() => {});
    }

    setup();

    function handleVisibilityChange() {
      if (!document.hidden) {
        invoke<string | null>('get_loaded_model')
          .then(setLoadedModel)
          .catch(() => {});
      }
    }
    document.addEventListener('visibilitychange', handleVisibilityChange);

    return () => {
      unlistenLoaded?.();
      unlistenEvicted?.();
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, []);

  const { resetTo: resetMin } = useDebouncedSave(
    'inference',
    'keep_warm_inactivity_minutes',
    inactivityMin,
    { onSaved },
  );

  const { resetTo: resetNumCtx } = useDebouncedSave(
    'inference',
    'num_ctx',
    numCtx,
    { onSaved },
  );

  const prevTokenRef = useRef(resyncToken);

  if (prevTokenRef.current !== resyncToken) {
    prevTokenRef.current = resyncToken;
    setInactivityMin(config.inference.keep_warm_inactivity_minutes);
    resetMin(config.inference.keep_warm_inactivity_minutes);
    const nextCtx = config.inference.num_ctx;
    setNumCtx(nextCtx);
    setCtxPos(ctxToPos(nextCtx));
    setCtxChip(String(nextCtx));
    resetNumCtx(nextCtx);
  }

  function commitCtx(v: number) {
    setNumCtx(v);
    setCtxPos(ctxToPos(v));
    setCtxChip(String(v));
  }

  function handleEject() {
    setEjecting(true);
    invoke('evict_model')
      .then(() => {
        setTimeout(() => setEjecting(false), EJECT_RESET_MS);
      })
      .catch(() => setEjecting(false));
  }

  const ctxTurns = Math.round(numCtx / TOKENS_PER_TURN_ESTIMATE);
  const fillPct = `${ctxPos / 10}%`;

  return (
    <>
      <Section heading="Ollama">
        <SaveField
          section="inference"
          fieldKey="ollama_url"
          label="Ollama URL"
          helper={configHelp('inference', 'ollama_url')}
          initialValue={config.inference.ollama_url}
          resyncToken={resyncToken}
          onSaved={onSaved}
          render={(value, setValue, errored) => (
            <TextField
              value={value}
              onChange={setValue}
              placeholder="http://127.0.0.1:11434"
              errored={errored}
              ariaLabel="Ollama URL"
            />
          )}
        />
      </Section>

      <Section heading="Keep Warm">
        <div className={styles.keepWarmRow1}>
          <div className={styles.keepWarmLabelLine}>
            <span className={styles.keepWarmLabel}>
              Keep active model in VRAM
            </span>
            <span className={styles.infoBtn} title={KEEP_WARM_TOOLTIP}>?</span>
          </div>
          <div className={styles.keepWarmTimerGroup}>
            <span className={styles.keepWarmBarFieldLabel}>Release after</span>
            <input
              type="number"
              className={styles.keepWarmNumberInput}
              value={inactivityMin}
              min={-1}
              max={1440}
              aria-label="Release after N minutes"
              onChange={(e) => {
                const n = parseInt(e.target.value, 10);
                if (!Number.isNaN(n)) {
                  setInactivityMin(Math.max(-1, Math.min(1440, n)));
                }
              }}
            />
            <span className={styles.keepWarmUnit}>min</span>
          </div>
        </div>

        <div className={styles.keepWarmStatusRow}>
          <div className={styles.keepWarmStatusLeft}>
            {loadedModel !== null ? (
              <div className={styles.keepWarmVramSubtitle}>
                <span className={styles.keepWarmVramDot} data-testid="vram-status-dot" aria-hidden="true" />
                <span className={styles.keepWarmVramModelName}>{loadedModel}</span>
                <span>&nbsp;in VRAM</span>
              </div>
            ) : (
              <span className={styles.keepWarmNoModel}>No model loaded</span>
            )}
          </div>
          <button
            type="button"
            className={styles.keepWarmEjectPill}
            aria-label="Unload now"
            disabled={ejecting || loadedModel === null}
            data-ejecting={ejecting}
            onClick={handleEject}
          >
            Unload now
          </button>
        </div>
      </Section>

      <Section heading="Context Window">
        <div className={styles.ctxBlock}>
          <div className={styles.ctxTopRow}>
            <span className={styles.ctxLabel}>Context window</span>
            <div className={styles.ctxChipGroup}>
              <input
                type="number"
                className={styles.ctxChipInput}
                value={ctxChip}
                min={CTX_MIN}
                max={CTX_MAX}
                aria-label="Context window tokens"
                onChange={(e) => setCtxChip(e.target.value)}
                onBlur={() => {
                  const n = parseInt(ctxChip, 10);
                  if (!Number.isNaN(n) && n >= CTX_MIN) {
                    commitCtx(Math.min(n, CTX_MAX));
                  } else {
                    setCtxChip(String(numCtx));
                  }
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') (e.target as HTMLInputElement).blur();
                }}
              />
              <span className={styles.ctxChipUnit}>tokens</span>
            </div>
          </div>

          <input
            type="range"
            className={styles.ctxSlider}
            style={{ '--fill': fillPct } as React.CSSProperties}
            min={0}
            max={1000}
            step={1}
            value={ctxPos}
            aria-label="Context window tokens"
            aria-valuemin={CTX_MIN}
            aria-valuemax={CTX_MAX}
            aria-valuenow={numCtx}
            aria-valuetext={`${numCtx} tokens`}
            onChange={(e) => {
              ctxDraggingRef.current = true;
              const pos = Number(e.target.value);
              setCtxPos(pos);
              setCtxChip(String(posToCtx(pos)));
            }}
            onMouseUp={() => {
              ctxDraggingRef.current = false;
              commitCtx(posToCtx(ctxPos));
            }}
            onTouchEnd={() => {
              ctxDraggingRef.current = false;
              commitCtx(posToCtx(ctxPos));
            }}
            onKeyUp={() => {
              if (!ctxDraggingRef.current) commitCtx(posToCtx(ctxPos));
            }}
          />

          <div className={styles.ctxTickRow} aria-hidden="true">
            {CTX_TICKS.map((label, i) => (
              <span
                key={label}
                className={styles.ctxTick}
                style={{ left: `${(i / (CTX_TICKS.length - 1)) * 100}%` }}
              >
                {label}
              </span>
            ))}
          </div>

          <div className={styles.ctxHelper}>
            ~{ctxTurns.toLocaleString()} turns of context
            {' - '}
            Ollama caps to your model&apos;s trained maximum.
          </div>

          <div className={styles.ctxVramNote}>
            <span className={styles.ctxVramIcon} aria-hidden="true">!</span>
            <span>
              The KV cache scales linearly with context length, so doubling the
              context roughly doubles its memory footprint.
            </span>
          </div>
        </div>
      </Section>

      <Section heading="Prompt">
        <SaveField
          section="prompt"
          fieldKey="system"
          label="System prompt"
          helper={configHelp('prompt', 'system')}
          vertical
          initialValue={config.prompt.system}
          resyncToken={resyncToken}
          onSaved={onSaved}
          render={(value, setValue) => (
            <>
              <Textarea
                value={value}
                onChange={setValue}
                placeholder="Use built-in secretary persona..."
                maxLength={PROMPT_MAX_CHARS}
                ariaLabel="System prompt"
              />
              <div className={styles.charCounter}>
                {value.length} / {PROMPT_MAX_CHARS}
              </div>
            </>
          )}
        />
      </Section>
    </>
  );
}