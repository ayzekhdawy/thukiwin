/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_QUOTE_MAX_DISPLAY_LINES: string | undefined;
  readonly VITE_QUOTE_MAX_DISPLAY_CHARS: string | undefined;
  readonly VITE_QUOTE_MAX_CONTEXT_LENGTH: string | undefined;
  readonly VITE_GIT_COMMIT_SHA: string | undefined;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
