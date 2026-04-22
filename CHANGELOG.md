# Changelog

## [0.9.0](https://github.com/ayzekhdawy/thukiwin/compare/v0.8.0...v0.9.0) (2026-04-22)

### Features

* **cloud API computer control**: add OpenAI (GPT-4o) and Anthropic (Claude) providers with native tool-calling for reliable desktop automation
* **provider abstraction**: streaming SSE support for OpenAI `/v1/chat/completions` and Anthropic `/v1/messages` with automatic tool call → action conversion
* **settings panel**: in-app UI for switching between Ollama/OpenAI/Anthropic providers, API key configuration, model selection, and base URL
* **intent detection**: Turkish and English phrase recognition to auto-route "ekrandaki ne görüyorsun" → vision and "open notepad" → agent mode
* **learning confirmation**: first 3 agent actions per session require user approval, then auto-execute; dangerous actions always require confirmation
* **local gateway**: OpenAI-compatible HTTP server on port 18789 for third-party integration
* **auto-start on boot**: Windows Task Scheduler integration via `schtasks`

### Bug Fixes

* fix settings panel triggering minibar activation on window focus loss
* fix ResizeObserver resize events while settings modal is open
* fix minibar drag instability from pointer event listener cleanup issues
* fix Escape/close overlay handler interfering with open settings panel
* fix OpenAI tool call streaming type conversion for `ProviderChunk::ToolCalls`
* fix Anthropic request JSON construction (removed unsupported `json!` spread)
* remove all blur/frosted glass effects for solid, stable backgrounds

## [0.8.0](https://github.com/ayzekhdawy/thukiwin/compare/v0.7.0...v0.8.0) (2026-04-17)


### Features

* add keyboard shortcuts and fix VoiceSelector Escape bug ([704012b](https://github.com/ayzekhdawy/thukiwin/commit/704012b7c10c1f413644f4bfb7bdb91d1a08cb3a))

## [0.7.0](https://github.com/ayzekhdawy/thukiwin/compare/v0.6.1...v0.7.0) (2026-04-17)


### Features

* adapt UI for Windows Fluent Design and fix Windows bugs ([ea77b18](https://github.com/ayzekhdawy/thukiwin/commit/ea77b181da5865bc158c4378594a328bd6af6ac4))
* add Edge TTS (text-to-speech) with voice selection and privacy disclosure ([1ab533d](https://github.com/ayzekhdawy/thukiwin/commit/1ab533d0c4c1556a3bc390d415e8ee778527f553))
* ThukiWin — Windows port of Thuki ([83b3bde](https://github.com/ayzekhdawy/thukiwin/commit/83b3bde2cf4b5a89ba5fc4c892036e4361bbfc32))


### Bug Fixes

* achieve 100% test coverage for CI ([a7d8034](https://github.com/ayzekhdawy/thukiwin/commit/a7d8034e73ea8408995d360e88eb8fe251ff2132))
* add media-src blob to CSP for TTS audio playback ([7d0ce3f](https://github.com/ayzekhdawy/thukiwin/commit/7d0ce3fe8d36d02310067739cd1fd1d246ad7b94))
* update CI workflow for Windows builds ([92a9ec2](https://github.com/ayzekhdawy/thukiwin/commit/92a9ec2d528b2783b21f625af6fa7fddcb14c893))
* update windows crate API for 0.61, add TTS docs, fix release-please workflow ([f648d38](https://github.com/ayzekhdawy/thukiwin/commit/f648d3825c3a601663095a42963cec3a546e8971))
* use RELEASE_PLEASE_TOKEN secret for workflow ([3e99ce8](https://github.com/ayzekhdawy/thukiwin/commit/3e99ce8bb682333ca990daf024d02202f2cea376))
