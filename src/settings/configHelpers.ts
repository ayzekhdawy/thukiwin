/**
 * Tooltip copy for every user-tunable Settings field.
 *
 * Indexed by the same (section, key) pair the backend's
 * `set_config_field` allowlist uses.
 */

const HELPERS = {
  inference: {
    ollama_url:
      'The web address where ThukiWin finds your local Ollama server. The default works if you run Ollama on this machine with its standard port. Change this only if you moved Ollama to a different port or another machine.',
    keep_warm:
      'When on, ThukiWin tells Ollama to keep the active model loaded in GPU memory between conversations, saving the cold-load wait on every open. Set "Release after" to -1 to keep it indefinitely, or pick a timeout in minutes so GPU memory is reclaimed when you stop using ThukiWin for a while.',
    num_ctx:
      "The size of the context window sent to Ollama with every request, in tokens. This value must match between warmup and chat so Ollama can reuse the same runner and its cached key-value prefix for the system prompt. Raise to fit longer conversations without the model forgetting early messages; lower to reduce GPU memory use. Valid range: 2048-1048576.",
  },
  prompt: {
    system:
      "Your custom personality or instructions for the AI. Leave this empty to use ThukiWin's built-in secretary personality. The list of slash commands is always added on top.",
  },
  window: {
    overlay_width:
      'How wide the floating ThukiWin window is, in pixels. Raise for wider input/chat at the cost of more screen space; lower to keep ThukiWin compact.',
    max_chat_height:
      'The largest the chat window can grow to as conversation gets longer. Raise to see more chat history without scrolling; lower to keep ThukiWin from taking over your screen.',
    max_images:
      'How many images you can attach to a single message by pasting or dragging. Raise for richer visual context per message; lower to keep prompts compact.',
  },
  quote: {
    max_display_lines:
      'How many lines of the quoted text are shown as a preview in the input bar. The full text is still sent to the AI; this only affects what you see.',
    max_display_chars:
      'How many characters of the quoted text are shown as a preview in the input bar. Same idea as max display lines: the full text is still sent to the AI.',
    max_context_length:
      'How many characters of the quoted text are actually sent to the AI. Anything past this is cut off. Raise if you quote long passages; lower if your model has a small context window.',
  },
  search: {
    searxng_url:
      "Where ThukiWin's local search engine (SearXNG) is running. Keep this on 127.0.0.1; pointing it at a remote host leaks every search query.",
    reader_url:
      "Where ThukiWin's local web-page reader is running. The reader opens promising URLs, strips out ads, menus, and scripts, and hands the clean text back so the AI can read it.",
    searxng_max_results:
      'How many results SearXNG returns for each query. Raise for wider coverage; lower for faster, narrower searches.',
    max_iterations:
      'How many rounds of searching the AI is allowed to do for a single question. Raise for hard, multi-step questions; lower for faster answers and fewer tokens.',
    top_k_urls:
      'How many web pages ThukiWin actually opens and reads after picking the most promising ones from the search results. Raise for more sources; lower for faster searches.',
    search_timeout_s:
      'How long (in seconds) ThukiWin waits for SearXNG to come back with search results before giving up. Raise for slow internet connections.',
    reader_per_url_timeout_s:
      'How long (in seconds) ThukiWin waits for one single web page to load before giving up on it. Raise for slow websites.',
    reader_batch_timeout_s:
      'How long (in seconds) ThukiWin waits for the whole batch of pages it is reading in parallel to finish. Must be larger than the per-URL timeout.',
    judge_timeout_s:
      'How long (in seconds) ThukiWin waits for the AI to decide whether the search results are good enough. Raise for slow local AI models.',
    router_timeout_s:
      'How long (in seconds) ThukiWin waits for the AI to decide whether your question needs a web search. Raise for slow local AI models.',
  },
  gateway: {
    enabled:
      'Enable the local gateway server. When enabled, other apps on your machine can send messages to ThukiWin through the gateway port.',
    port: 'The port number the local gateway listens on. Must be between 1024 and 65535. Default is 18789.',
  },
  tts: {
    voice: 'The voice used for text-to-speech. Choose from the available system voices. The default is a Turkish female voice.',
    rate: 'Speech rate adjustment. Negative values slow down speech; positive values speed it up. Range: -50 to 50.',
    pitch: 'Pitch adjustment. Negative values lower the pitch; positive values raise it. Range: -50 to 50.',
  },
  agent: {
    provider:
      'The AI provider for agent mode. Ollama uses your local models; OpenAI and Anthropic use cloud models (requires API key).',
    model: 'The model used by the agent. When using Ollama, this can be any locally available model. For cloud providers, choose from the supported models.',
    base_url:
      'The base URL for the agent provider API. Defaults to the standard endpoint for each provider. Change this only if you use a custom proxy or endpoint.',
  },
  debug: {
    search_trace_enabled:
      'When on, ThukiWin writes a detailed trace file for every /search turn. Useful for diagnosing search issues; leave off for normal use.',
  },
} as const;

export function configHelp<
  S extends keyof typeof HELPERS,
  K extends keyof (typeof HELPERS)[S],
>(section: S, key: K): string {
  return HELPERS[section][key] as string;
}