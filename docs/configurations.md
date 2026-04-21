# Configurations

ThukiWin uses environment variables for runtime configuration. Vite loads these from `.env` files at build/dev time and exposes variables prefixed with `VITE_` to the frontend via `import.meta.env`.

## Setup

```powershell
copy .env.example .env
```

Edit `.env` to override any defaults. Changes take effect on the next `bun run dev` or `bun run build:all`.

> `.env` is gitignored. `.env.example` is committed as the reference template.

## Configuration Reference

### Quote Display

Controls how selected-text quotes are displayed in the AskBar preview and chat bubbles, and how much context is forwarded to the LLM.

| Variable | Description | Default | Type |
| :--- | :--- | :--- | :--- |
| `VITE_QUOTE_MAX_DISPLAY_LINES` | Maximum number of lines shown in the quote preview. Lines beyond this limit are truncated with `...`. Empty lines in the selection are skipped and do not count toward this limit. | `4` | Positive integer |
| `VITE_QUOTE_MAX_DISPLAY_CHARS` | Maximum total characters shown in the quote preview. If a line would push the total past this limit, it is truncated mid-line with `...`. | `300` | Positive integer |
| `VITE_QUOTE_MAX_CONTEXT_LENGTH` | Maximum length (in characters) of selected context text included in the prompt sent to Ollama. This is a security and performance cap; selections longer than this are silently truncated before reaching the LLM. | `4096` | Positive integer |

### System Prompt

Controls the system prompt prepended to every conversation sent to Ollama.

| Variable | Description | Default |
| :--- | :--- | :--- |
| `THUKI_SYSTEM_PROMPT` | Custom system prompt for all conversations. If unset or empty, the built-in default is used. | Built-in secretary persona prompt (see `src-tauri/src/commands.rs`) |

### Model Configuration

Controls which Ollama model(s) Thuki uses for inference.

| Variable | Description | Default |
| :--- | :--- | :--- |
| `THUKI_SUPPORTED_AI_MODELS` | Comma-separated list of Ollama model names. The first entry is the active model used for all inference. Additional entries are available for future in-app model switching. | `gemini-3-flash-preview` |

**Example:**

```bash
# Single model (default behavior)
THUKI_SUPPORTED_AI_MODELS=gemini-3-flash-preview

# Multiple models (first is active; others available for future UI picker)
THUKI_SUPPORTED_AI_MODELS=gemini-3-flash-preview,gemma4:e2b
```

Whitespace around each entry is trimmed. Empty entries are ignored. If the variable is unset or empty, ThukiWin falls back to `gemini-3-flash-preview`.

### Agent Mode Configuration

Controls the behavior of the `/do` computer-control agent.

| Variable | Description | Default |
| :--- | :--- | :--- |
| `THUKI_AGENT_MODEL` | Ollama model used for agent mode (must be vision-capable). If unset, falls back to the first model in `THUKI_SUPPORTED_AI_MODELS`. | _(uses `THUKI_SUPPORTED_AI_MODELS`)_ |
| `THUKI_AGENT_MAX_ITERATIONS` | Maximum number of screenshot→analyze→execute cycles before the agent stops. | `50` |
| `THUKI_AGENT_ACTION_DELAY_MS` | Milliseconds to wait between executing each action. Gives you time to observe what the agent is doing. | `300` |
| `THUKI_AGENT_SCREENSHOT_DELAY_MS` | Milliseconds to wait after an action before taking the next screenshot (allows UI to update). | `500` |

**Example:**

```bash
# Use a specific vision model for agent mode
THUKI_AGENT_MODEL=llama3.2-vision

# Slower actions for easier observation
THUKI_AGENT_ACTION_DELAY_MS=500
```

### Validation Rules

All configuration values are validated at startup via `src/config/index.ts`:

- **Missing or empty** values fall back to the default.
- **Non-numeric** values (e.g., `abc`) fall back to the default.
- **Zero or negative** values fall back to the default.
- **Decimal** values are floored to the nearest integer (e.g., `5.7` becomes `5`).
- **Infinity** falls back to the default.

### File Precedence

Vite loads `.env` files in the following order (later files override earlier ones):

| File | Purpose | Committed |
| :--- | :--- | :--- |
| `.env.example` | Reference template with documented defaults | Yes |
| `.env` | Local configuration | No (gitignored) |
| `.env.local` | Local overrides (highest priority) | No (gitignored via `*.local`) |
| `.env.development` | Dev-only overrides (loaded when `bun run dev`) | Optional |
| `.env.production` | Prod-only overrides (loaded when `bun run build:all`) | Optional |
