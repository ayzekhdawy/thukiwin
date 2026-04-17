# ThukiWin Feature Roadmap

## In Progress
- [ ] Keyboard shortcuts (Escape, Ctrl+N, Ctrl+S, Ctrl+H, Ctrl+Shift+C)

## Planned — Easy (1-3 hours each)

### Model Selection UI
- Dropdown to change the active Ollama model from the UI
- Integrate with the existing `THUKI_SUPPORTED_AI_MODELS` env var
- Add model indicator to WindowControls or AskBar
- Files: `src/App.tsx`, `src/hooks/useOllama.ts`, `src/components/ModelSelector.tsx` (new)

### Tray Notifications
- Show a Windows notification when AI response completes
- Awareness when the user is in another app
- `tauri-plugin-notification` or Windows toast notification
- Files: `src-tauri/Cargo.toml`, `src/hooks/useOllama.ts`, `src-tauri/src/lib.rs`

### Markdown Export
- Export conversations as `.md` files
- Add "Export" button to history panel
- Files: `src/components/HistoryPanel.tsx`, `src/hooks/useConversationHistory.ts`

## Planned — Medium (4-5 hours each)

### Conversation Search
- Full-text search with SQLite FTS5
- Add search bar to history panel
- Instant results (with debounce)
- Files: `src-tauri/src/database.rs`, `src-tauri/src/history.rs`, `src/components/HistoryPanel.tsx`

### Prompt Templates
- Allow users to save custom prompt templates
- Works like slash commands but user-defined
- Store in localStorage or SQLite
- Files: `src/config/commands.ts`, `src/components/TemplateManager.tsx` (new)

### Pinned Conversations
- Pin frequently used conversations
- Ready on every launch
- Pin icon in history panel
- Files: `src-tauri/src/database.rs`, `src-tauri/src/history.rs`, `src/components/HistoryPanel.tsx`

## Planned — Hard (1-3 days each)

### Vision / Model Improvement
- Image analysis with Ollama multimodal models
- Model-based improvement after `/screen`
- Model feature detection (is it vision capable?)
- Files: `src/hooks/useOllama.ts`, `src/App.tsx`, `src-tauri/src/commands.rs`

### Plugin / Extension System
- Third-party tool integrations (web search, file reading)
- Security: run inside sandbox
- Files: New module `src-tauri/src/plugins.rs`, `src/plugins/` (new directory)

### Split-View Mode
- Side-by-side dual conversation windows
- Reference text in one, AI response in the other
- Files: `src/App.tsx`, `src/view/ConversationView.tsx`, `src/view/SplitView.tsx` (new)