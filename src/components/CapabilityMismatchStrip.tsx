/**
 * CapabilityMismatchStrip — inline warning when the active model doesn't
 * support a capability required by the current action (e.g., vision for
 * /screen, thinking for /think).
 */

import type { Capabilities } from '../hooks/useModelSelection';
import styles from '../styles/capability-mismatch.module.css';

interface CapabilityMismatchStripProps {
  activeModel: string | null;
  capabilities: Record<string, Capabilities>;
  /** The conflict messages to display */
  conflicts: string[];
}

export function CapabilityMismatchStrip({
  conflicts,
}: CapabilityMismatchStripProps) {
  if (conflicts.length === 0) return null;

  return (
    <div className={styles.strip}>
      {conflicts.map((msg, i) => (
        <div key={i} className={styles.warning}>
          <span className={styles.icon}>⚠</span>
          <span className={styles.text}>{msg}</span>
        </div>
      ))}
    </div>
  );
}