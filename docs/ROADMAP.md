# ThukiWin Feature Roadmap

## In Progress
- [ ] Klavye shortcut'ları (Escape, Ctrl+N, Ctrl+S, Ctrl+H, Ctrl+Shift+C)

## Planned — Easy (1-3 saat each)

### Model Seçme UI
- UI'dan aktif Ollama modelini değiştirme dropdown'u
- Mevcut `THUKI_SUPPORTED_AI_MODELS` env var'ı ile entegre
- WindowControls veya AskBar'a model indicator ekle
- Dosyalar: `src/App.tsx`, `src/hooks/useOllama.ts`, `src/components/ModelSelector.tsx` (yeni)

### Tray Bildirimleri
- AI yanıtı tamamlandığında Windows notification göster
- Kullanıcı başka app'deyken farkındalık
- `tauri-plugin-notification` veya Windows toast notification
- Dosyalar: `src-tauri/Cargo.toml`, `src/hooks/useOllama.ts`, `src-tauri/src/lib.rs`

### Markdown Export
- Konuşmaları `.md` dosyası olarak dışa aktarma
- History panel'e "Export" butonu ekle
- Dosyalar: `src/components/HistoryPanel.tsx`, `src/hooks/useConversationHistory.ts`

## Planned — Medium (4-5 saat each)

### Konuşma Arama
- SQLite FTS5 ile tam metin arama
- History panel'e search bar ekle
- Anlık sonuçlar (debounce ile)
- Dosyalar: `src-tauri/src/database.rs`, `src-tauri/src/history.rs`, `src/components/HistoryPanel.tsx`

### Prompt Templates
- Kullanıcı özel prompt şablonları kaydedebilme
- Slash commands gibi çalışan ama kullanıcı tanımlı
- localStorage veya SQLite'da saklama
- Dosyalar: `src/config/commands.ts`, `src/components/TemplateManager.tsx` (yeni)

### Pinned Conversations
- Sık kullanılan konuşmaları sabitleme
- Her açılışta hazır olma
- History panel'de pin ikonu
- Dosyalar: `src-tauri/src/database.rs`, `src-tauri/src/history.rs`, `src/components/HistoryPanel.tsx`

## Planned — Hard (1-3 gün each)

### Vision / Model İyileştirme
- Ollama multimodal modelleriyle resim analizi
- `/screen` sonrası model bazlı iyileştirme
- Model özellik algılama (vision capable mı)
- Dosyalar: `src/hooks/useOllama.ts`, `src/App.tsx`, `src-tauri/src/commands.rs`

### Plugin / Extension Sistemi
- Üçüncü taraf araç entegrasyonları (web arama, dosya okuma)
- Güvenlik: sandbox içinde çalıştırma
- Dosyalar: Yeni modül `src-tauri/src/plugins.rs`, `src/plugins/` (yeni dizin)

### Split-View Modu
- Yan yana iki konuşma penceresi
- Birinde referans metin, diğerinde AI yanıtı
- Dosyalar: `src/App.tsx`, `src/view/ConversationView.tsx`, `src/view/SplitView.tsx` (yeni)