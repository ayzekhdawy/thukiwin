/**
 * Capability conflict detection.
 *
 * Maps model capability flags to user-facing warnings when a command requires
 * a capability the active model doesn't support (e.g., /screen needs vision,
 * /think needs thinking mode).
 */

import type { Capabilities } from '../hooks/useModelSelection';

export interface CapabilityConflict {
  command: string;
  requiredCapability: 'vision' | 'thinking';
  modelSupports: boolean;
  message: string;
}

const COMMAND_CAPABILITIES: Record<string, ('vision' | 'thinking')[]> = {
  '/screen': ['vision'],
  '/think': ['thinking'],
};

export function getCapabilityConflicts(
  activeModel: string | null,
  capabilities: Record<string, Capabilities>,
  command: string,
): CapabilityConflict[] {
  if (!activeModel || !capabilities[activeModel]) return [];
  const caps = capabilities[activeModel];
  const required = COMMAND_CAPABILITIES[command];
  if (!required) return [];

  return required
    .filter((cap) => {
      const supported = cap === 'vision' ? caps.vision : caps.thinking;
      return !supported;
    })
    .map((cap) => ({
      command,
      requiredCapability: cap,
      modelSupports: false,
      message:
        cap === 'vision'
          ? `/${command.slice(1)} requires a vision-capable model, but ${activeModel} does not support images.`
          : `/${command.slice(1)} requires a thinking-capable model, but ${activeModel} does not support reasoning.`,
    }));
}

export function hasVisionConflict(
  activeModel: string | null,
  capabilities: Record<string, Capabilities>,
  imageCount: number,
): boolean {
  if (imageCount === 0) return false;
  if (!activeModel || !capabilities[activeModel]) return false;
  return !capabilities[activeModel].vision;
}

export function getMaxImages(
  activeModel: string | null,
  capabilities: Record<string, Capabilities>,
): number | null {
  if (!activeModel || !capabilities[activeModel]) return null;
  return capabilities[activeModel].maxImages;
}