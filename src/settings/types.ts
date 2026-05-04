/**
 * Settings panel type definitions.
 *
 * These mirror the Rust `AppConfig` schema (snake_case) so the
 * frontend can pass values straight through `set_config_field`
 * without an intermediate camelCase translation.
 */

export interface RawAppConfig {
  inference: {
    ollama_url: string;
    keep_warm_inactivity_minutes: number;
    num_ctx: number;
  };
  prompt: {
    system: string;
  };
  window: {
    overlay_width: number;
    max_chat_height: number;
    max_images: number;
  };
  quote: {
    max_display_lines: number;
    max_display_chars: number;
    max_context_length: number;
  };
  search: {
    searxng_url: string;
    reader_url: string;
    max_iterations: number;
    top_k_urls: number;
    searxng_max_results: number;
    search_timeout_s: number;
    reader_per_url_timeout_s: number;
    reader_batch_timeout_s: number;
    judge_timeout_s: number;
    router_timeout_s: number;
  };
  gateway: {
    enabled: boolean;
    port: number;
  };
  tts: {
    voice: string;
    rate: number;
    pitch: number;
  };
  agent: {
    provider: string;
    model: string;
    base_url: string;
  };
  debug: {
    search_trace_enabled: boolean;
  };
}

/** Tagged union returned by the Rust `set_config_field` command on failure. */
export type ConfigError =
  | { kind: 'seed_failed'; path: string; source: string }
  | { kind: 'io_error'; path: string; source: string }
  | { kind: 'unknown_section'; section: string }
  | { kind: 'unknown_field'; section: string; key: string }
  | { kind: 'type_mismatch'; section: string; key: string; message: string }
  | { kind: 'parse'; path: string; message: string };

/** Recovery marker payload returned by `get_corrupt_marker`. */
export interface CorruptMarker {
  path: string;
  ts: number;
}

/** Identifier for the active Settings tab. */
export type SettingsTabId =
  | 'general'
  | 'search'
  | 'display'
  | 'agent'
  | 'gateway'
  | 'sound'
  | 'about';

export function describeConfigError(err: unknown): string {
  if (typeof err !== 'object' || err === null) {
    return "Couldn't save. Please try again.";
  }
  const e = err as Partial<ConfigError> & { kind?: string; message?: string };
  switch (e.kind) {
    case 'io_error':
      return `Couldn't save: ${e.source ?? 'I/O error'}.`;
    case 'unknown_section':
      return `Unknown section: ${e.section}.`;
    case 'unknown_field':
      return `Unknown field: ${e.section}.${e.key}.`;
    case 'type_mismatch':
      return e.message ?? 'Wrong type for this field.';
    case 'parse':
      return 'config.toml has a syntax error. Restart ThukiWin to recover.';
    case 'seed_failed':
      return `Couldn't write defaults: ${e.source ?? ''}.`;
    default:
      return typeof e.message === 'string' ? e.message : "Couldn't save.";
  }
}