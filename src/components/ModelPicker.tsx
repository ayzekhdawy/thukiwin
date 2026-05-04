/**
 * ModelPicker — dropdown/popup for selecting the active Ollama model.
 *
 * Shows the current model as a chip. Clicking opens a panel listing all
 * installed models with capability badges (vision, thinking).
 * Models not yet loaded show a warmup indicator.
 */

import { useState, useRef, useEffect } from 'react';
import type { Capabilities } from '../hooks/useModelSelection';
import styles from '../styles/model-picker.module.css';

interface ModelPickerProps {
  active: string | null;
  all: string[];
  ollamaReachable: boolean;
  capabilities: Record<string, Capabilities>;
  onSelect: (model: string) => void;
}

function CapabilityBadge({ cap }: { cap: Capabilities }) {
  const badges: string[] = [];
  if (cap.vision) badges.push('vision');
  if (cap.thinking) badges.push('thinking');
  if (badges.length === 0) return null;
  return (
    <span className={styles.badgeGroup}>
      {badges.map((b) => (
        <span key={b} className={`${styles.badge} ${styles[b]}`}>
          {b}
        </span>
      ))}
    </span>
  );
}

export function ModelPicker({
  active,
  all,
  ollamaReachable,
  capabilities,
  onSelect,
}: ModelPickerProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [open]);

  if (!ollamaReachable) {
    return (
      <div className={styles.picker}>
        <span className={styles.unreachable}>Ollama unreachable</span>
      </div>
    );
  }

  if (all.length === 0) {
    return (
      <div className={styles.picker}>
        <span className={styles.noModels}>No models installed</span>
      </div>
    );
  }

  return (
    <div className={styles.picker} ref={ref}>
      <button
        className={styles.chip}
        onClick={() => setOpen((v) => !v)}
        aria-label="Select model"
        aria-expanded={open}
      >
        <span className={styles.chipLabel}>{active || 'Select model'}</span>
        <span className={styles.chevron}>{open ? '▲' : '▼'}</span>
      </button>

      {open && (
        <div className={styles.panel} role="listbox">
          {all.map((model) => (
            <button
              key={model}
              className={`${styles.row} ${model === active ? styles.active : ''}`}
              role="option"
              aria-selected={model === active}
              onClick={() => {
                onSelect(model);
                setOpen(false);
              }}
            >
              <span className={styles.modelName}>{model}</span>
              {capabilities[model] && <CapabilityBadge cap={capabilities[model]} />}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}