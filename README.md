<h1 align="center">
  ThukiWin
</h1>

<p align="center">
  <img src="public/thuki-logo.png" alt="ThukiWin logo" width="300" />
</p>

<p align="center">
  A floating AI secretary for Windows. Fully local, completely free, zero data ever leaves your machine.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-beta-yellow.svg" alt="Beta" />
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License" /></a>
  <img src="https://img.shields.io/badge/platform-Windows-0078D4?logo=windows11&logoColor=white" alt="Platform: Windows" />
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-v2-24C8DB?logo=tauri&logoColor=white" alt="Tauri v2" />
  <img src="https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=black" alt="React 19" />
  <img src="https://img.shields.io/badge/TypeScript-5.8-3178C6?logo=typescript&logoColor=white" alt="TypeScript" />
  <img src="https://img.shields.io/badge/Rust-stable-CE422B?logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/Tailwind_CSS-4-06B6D4?logo=tailwindcss&logoColor=white" alt="Tailwind CSS 4" />
  <img src="https://img.shields.io/badge/SQLite-bundled-003B57?logo=sqlite&logoColor=white" alt="SQLite" />
  <img src="https://img.shields.io/badge/Ollama-local-black" alt="Ollama" />
</p>

---

<p align="center">
  <img src="public/demo.gif" alt="ThukiWin demo — double-tap Ctrl to summon, ask, dismiss" width="560" />
</p>

<p align="center">
  <a href="https://github.com/ayzekhdawy/thukiwin/releases/download/untagged-fb9e77c48260eb00efb2/ThukiWin-demo.mp4">Watch the full demo video</a>
</p>

**No subscriptions. No cloud required. No telemetry. Free forever.**

ThukiWin is a lightweight Windows overlay powered by AI — fully local via Ollama, or connect to OpenAI/Anthropic for advanced computer control. Built for quick, uninterrupted asks without ever leaving what you're doing.

Fork of [Thuki](https://github.com/quiet-node/thuki) (macOS) — ported to Windows with native Win32 APIs.

## How It Works

Double-tap <kbd>Ctrl</kbd> to summon ThukiWin from anywhere. Ask a question, get an answer, and dismiss. Use `/screen` or the screenshot button to capture your screen and attach it as context. Type `/do` followed by a task to let the AI take over your desktop and complete it autonomously.

ThukiWin floats above every app. Highlight text anywhere, double-tap <kbd>Ctrl</kbd>, and ThukiWin opens with your selection pre-filled as a quote, ready to ask about. When you switch windows, ThukiWin shrinks to a slim minibar so you never lose track of the AI.

## Why ThukiWin?

Most AI tools require accounts, API keys, or subscriptions that bill you per token. ThukiWin is different:

- **100% free local AI:** run models locally via Ollama, no per-query cost, ever
- **Cloud API support:** optionally connect to OpenAI (GPT-4o) or Anthropic (Claude) for advanced computer control with native tool calling
- **Zero trust by design:** no analytics, no telemetry — cloud API keys are stored locally only
- **Your data is yours:** conversations are stored in a local SQLite database on your machine and nowhere else
- **Works everywhere on Windows:** double-tap <kbd>Ctrl</kbd> and ThukiWin appears on your desktop, inside a browser, inside a terminal, and in fullscreen apps

## Features

- **Always available:** double-tap <kbd>Ctrl</kbd> to summon the overlay from any app, including fullscreen apps
- **Context-aware quotes:** highlight any text, then double-tap <kbd>Ctrl</kbd> to open ThukiWin with the selected text pre-filled as a quote
- **Computer control (agent mode):** type `/do` or ask naturally (e.g. "open notepad") to let the AI autonomously control your desktop — click, type, launch apps, and more
- **Cloud API computer control:** connect OpenAI or Anthropic for reliable tool-calling computer use — the model returns structured actions instead of text parsing
- **Smart intent detection:** Turkish and English phrases like "ekrandaki ne görüyorsun" or "open notepad" automatically trigger vision analysis or agent mode
- **Settings panel:** switch between Ollama/OpenAI/Anthropic providers, configure API keys, model selection, and gateway — all from the UI
- **Learning confirmation:** first 3 agent actions require your approval, then auto-execute; dangerous actions always need confirmation
- **Minibar mode:** overlay shrinks to a thin always-on-top strip when you switch windows, so you never lose track of the AI
- **Throwaway conversations:** fast, lightweight interactions without the overhead of a full chat app
- **Conversation history:** persist and revisit past conversations across sessions
- **Fully local LLM:** powered by Ollama; no API keys, no accounts, no cost per query
- **Image input:** paste or drag images and screenshots directly into the chat
- **Screen capture:** type `/screen` to instantly capture your entire screen and attach it to your question as context
- **Slash commands:** built-in prompt shortcuts for common tasks: `/do`, `/screen`, `/think`, `/translate`, `/rewrite`, `/tldr`, `/refine`, `/bullets`, `/todos`. Highlight text anywhere, summon ThukiWin, type a command, and hit Enter
- **Extended reasoning:** type `/think` to have the model reason through a problem step by step before answering
- **Read Aloud (TTS):** click the speaker button on any message to have it read aloud using Microsoft Edge TTS voices (Windows-only; not available in the macOS version of Thuki)
- **Desktop notifications:** get notified when the AI finishes a response while you're in another window
- **Multi-monitor support:** screenshots and overlay positioning work across all monitors
- **Interactive region capture:** drag to select a specific area of your screen instead of capturing everything
- **Privacy-first:** zero-trust architecture, all data stays on your device

## Requirements

| Requirement | Minimum | Notes |
|---|---|---|
| **OS** | Windows 10 64-bit (1809+) | Windows 11 recommended |
| **RAM** | 8 GB | 16 GB+ recommended for larger models |
| **GPU** | Not required | CPU inference works; GPU speeds up larger models |
| **Disk** | 2 GB (app) + model size | Models are typically 2-8 GB each |
| **Ollama** | Latest | Must be running at `http://127.0.0.1:11434` |
| **Vision model** | For local agent mode | Pull a vision-capable model (e.g. `llama3.2-vision`) to use `/do` |
| **API key** | For cloud agent mode | OpenAI or Anthropic API key needed for tool-calling computer control |

### Build Requirements (for building from source)

- [Bun](https://bun.sh) v1.3+
- [Rust](https://rustup.rs) stable toolchain
- [Microsoft Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (C++ build tools with MSVC v143 + Windows 10/11 SDK)
- [Ollama](https://ollama.com) (for runtime AI inference)

## Getting Started

### Step 1: Set Up Your AI Engine

> **Default model:** ThukiWin ships with `gemini-3-flash-preview` by default. You can change the model at runtime via the `THUKI_SUPPORTED_AI_MODELS` environment variable; see [Configurations](docs/configurations.md).

1. **Install Ollama**

   Download and install from [ollama.com](https://ollama.com), or via winget:

   ```powershell
   winget install Ollama.Ollama
   ```

2. **Pull a model**

   ```powershell
   # Chat model (required)
   ollama pull gemini-3-flash-preview

   # Vision model for agent mode (optional, needed for /do)
   ollama pull llama3.2-vision
   ```

   > **Note:** Model files are large (typically 2-8 GB). This step can take several minutes depending on your internet connection. You only need to do it once.

3. **Verify the model is ready**

   ```powershell
   ollama list
   ```

   You should see your model listed. Once it appears, Ollama is ready and ThukiWin will connect to it automatically at `http://127.0.0.1:11434`.

### Step 2: Install ThukiWin

#### Download (Recommended)

1. Download `ThukiWin_x64-setup.exe` from the [latest release](https://github.com/ayzekhdawy/thukiwin/releases/latest)
2. Run the installer
3. Launch ThukiWin from the Start Menu or desktop shortcut
4. ThukiWin will appear in your system tray

> **No special permissions needed on Windows.** Unlike the macOS version, ThukiWin does not require Accessibility or Screen Recording permissions. The global hotkey and screen capture work out of the box on Windows.

#### Build from Source

```powershell
# Clone and install dependencies
git clone https://github.com/ayzekhdawy/thukiwin.git
cd thukiwin
bun install

# Launch in development mode
bun run dev
```

To produce an installer:

```powershell
bun run build:all
```

The output installers will be at:
- `src-tauri/target/release/bundle/msi/ThukiWin_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/ThukiWin_x64-setup.exe`

## Windows-Specific Details

### Hotkey

ThukiWin uses a low-level Windows keyboard hook (`SetWindowsHookExW` with `WH_KEYBOARD_LL`) to detect double-tap <kbd>Ctrl</kbd> globally. This works in all applications, including fullscreen apps and elevated (admin) windows on Windows 10 1809+.

- **Activation window:** 400 ms between the two Ctrl presses
- **Cooldown:** 600 ms after each activation to prevent accidental re-triggers

### Overlay Window

ThukiWin uses Tauri's `always_on_top` and `skip_taskbar` APIs combined with Win32 window styles to float above all other windows without appearing in the taskbar or stealing focus from the app you're working in.

### Screen Capture

Full-screen capture uses the Windows GDI `BitBlt` API, which works reliably on all Windows versions. Type `/screen` in the chat to capture your entire screen and attach it as context.

### Context Capture

When you double-tap <kbd>Ctrl</kbd> with text selected, ThukiWin uses clipboard simulation (Ctrl+C via `SendInput`) to capture the selected text, then restores the original clipboard contents. The mouse position is captured via `GetCursorPos` to position the overlay near your cursor.

### Permissions

Windows does not have macOS-style Accessibility or Screen Recording permission gates. ThukiWin works without any special permission setup. The onboarding flow on Windows skips the permission step and goes directly to the intro.

### Read Aloud (TTS)

ThukiWin includes a built-in text-to-speech feature using Microsoft Edge TTS voices. Click the speaker icon on any chat message to hear it read aloud. You can choose from hundreds of voices in dozens of languages via the voice selector dropdown.

> **Privacy note:** The TTS feature sends the text of the selected message to Microsoft's Edge TTS service over the internet. A one-time disclosure is shown on first use. This feature is Windows-only and is not available in the macOS version of Thuki.

### Computer Control (Agent Mode)

ThukiWin's `/do` command (or natural language like "open notepad") lets the AI autonomously control your desktop. The agent loop works as follows:

1. **Capture** — a screenshot of your screen is taken (excluding ThukiWin's own window)
2. **Analyze** — the screenshot is sent to the AI model along with your task description
3. **Execute** — the model returns actions (click, type, key press, scroll, launch) which are carried out via Win32 `SendInput`
4. **Repeat** — the loop captures a new screenshot, re-evaluates, and continues until the task is done or you stop it

**Two execution modes:**

| Mode | Provider | How it works |
|------|----------|-------------|
| **Local (text-parsing)** | Ollama | The model returns free-text actions like `CLICK 500 300` which are parsed and executed |
| **Cloud (tool-calling)** | OpenAI / Anthropic | The model returns structured JSON tool calls (e.g. `computer_click(x, y)`) which map directly to actions — more reliable for complex tasks |

**Local mode** works with any Ollama vision model but can be unreliable for structured output. **Cloud mode** uses native tool-calling APIs for deterministic, reliable computer control.

**Supported actions:**

| Action | Syntax | Example |
|--------|--------|---------|
| Left click | `CLICK x y` | `CLICK 500 300` |
| Double click | `DOUBLE_CLICK x y` | `DOUBLE_CLICK 500 300` |
| Right click | `RIGHT_CLICK x y` | `RIGHT_CLICK 500 300` |
| Drag | `DRAG x1 y1 x2 y2` | `DRAG 100 200 400 200` |
| Type text | `TYPE text` | `TYPE hello world` |
| Key combo | `KEY_PRESS modifier+key` | `KEY_PRESS ctrl+c` |
| Scroll | `SCROLL direction amount` | `SCROLL down 3` |
| Open app | `LAUNCH app` | `LAUNCH notepad` |
| Done | `DONE` | `DONE` |

**Usage examples:**

```
/do Open Notepad and type Hello World
/do Find the Settings app and open it
/do Close the current window
/do Copy the selected text and paste it into a new document
```

**Safety:**
- A maximum of 30 iterations prevents infinite loops
- 500 ms delay between actions lets you observe what's happening
- Click the **Stop** button to cancel at any time
- **Learning mode:** first 3 actions per session require your confirmation, then auto-execute; dangerous actions (opening apps) always require confirmation
- The agent uses a vision model to see your screen before each action — it adapts to what it finds, not a fixed script

**Vision model requirement:** Agent mode requires a vision-capable model (e.g. `llama3.2-vision`). The default chat model (`gemini-3-flash-preview`) handles text conversations. Set the vision model in your environment or let ThukiWin auto-detect it.

### Minibar Mode

When you switch away from ThukiWin while it's generating a response or running an agent task, the overlay automatically shrinks to a **minibar** — a thin 40 px always-on-top strip that stays visible above all windows.

The minibar shows:
- A **status dot** — green (idle/done), amber (thinking/executing), red (error)
- A **preview** of the last message or current action
- **Click** anywhere on the minibar to restore the full overlay

When the AI finishes a response while you're in another window, a desktop notification is sent so you don't miss it.

## Architecture & Security

<details>
<summary>Click to expand</summary>

ThukiWin is a **Tauri v2** app (Rust backend + React/TypeScript frontend) that interfaces with a locally running Ollama instance at `http://127.0.0.1:11434`.

### Stack

- **Frontend:** React 19 + TypeScript + Tailwind CSS 4 + Vite
- **Backend:** Rust with Tauri v2 + Win32 API (windows crate v0.61)
- **Database:** SQLite (bundled, local only)
- **AI Engine:** Ollama (local, no cloud)

### Security Model

1. **Frontend (Tauri/React):** Operates within a secure system webview (WebView2 on Windows) with restricted IPC. Streaming uses Tauri's Channel API; the Rust backend sends typed `StreamChunk` enum variants, and the frontend hook accumulates tokens into React state.

2. **Minimal network egress:** By default, ThukiWin only communicates with the local Ollama instance at `127.0.0.1:11434`. When a cloud provider (OpenAI/Anthropic) is configured for agent mode, API calls are sent directly to the respective cloud endpoint. The only other internet traffic is the Read Aloud (TTS) feature, which sends text to Microsoft's Edge TTS service. API keys are stored locally in the SQLite database and never sent to third parties.

3. **Local storage only:** All conversations are stored in a local SQLite database in the app data directory.

4. **Agent mode input simulation:** When `/do` is invoked, ThukiWin uses Win32 `SendInput` to simulate mouse and keyboard events. Actions are parsed from the model's text output and executed locally. The agent loop captures screenshots via GDI `BitBlt` and sends them as base64-encoded images to the vision model. No remote control or screen-sharing service is involved.

### Window Lifecycle

The app starts hidden. The hotkey or tray menu shows it. The window close button hides (not quits); quit is only available from the tray.

</details>

## Configuration

See [docs/configurations.md](docs/configurations.md) for the full configuration reference (model selection, agent mode, quote display limits, and system prompt).

See [docs/commands.md](docs/commands.md) for the full slash command reference.

## Contributing

Contributions are welcome! Read [CONTRIBUTING.md](CONTRIBUTING.md) to get started. Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

## Acknowledgments

ThukiWin is a Windows port of [Thuki](https://github.com/quiet-node/thuki) by Logan Nguyen.