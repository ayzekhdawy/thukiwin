import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';

/** Provider type for agent mode. */
type AgentProvider = 'ollama' | 'openai' | 'anthropic';

interface SettingsViewProps {
  modelConfig: { active: string; all: string[] } | null;
  onClose: () => void;
}

/** Recommended models for each provider. */
const PROVIDER_MODELS: Record<AgentProvider, string[]> = {
  ollama: ['gemini-3-flash-preview', 'llama3.2-vision', 'llama3.2', 'mistral'],
  openai: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo'],
  anthropic: ['claude-sonnet-4-20250514', 'claude-3-5-sonnet-20241022', 'claude-3-5-haiku-20241022'],
};

/** Default base URLs for each provider. */
const PROVIDER_URLS: Record<AgentProvider, string> = {
  ollama: 'http://127.0.0.1:11434',
  openai: 'https://api.openai.com/v1',
  anthropic: 'https://api.anthropic.com',
};

/** Gear (settings) icon SVG. */
export const SETTINGS_ICON = (
  <svg
    width="18"
    height="18"
    viewBox="0 0 20 20"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
  >
    <path
      fillRule="evenodd"
      clipRule="evenodd"
      d="M8.257 3.099A1 1 0 019.26 2h1.48a1 1 0 01.997.917l.082.827a6.073 6.073 0 011.387.8l.696-.442a1 1 0 011.244.206l1.045 1.045a1 1 0 01.206 1.244l-.442.696a6.073 6.073 0 01.8 1.387l.827.082A1 1 0 0118 9.26v1.48a1 1 0 01-.917.997l-.827.082a6.073 6.073 0 01-.8 1.387l.442.696a1 1 0 01-.206 1.244l-1.045 1.045a1 1 0 01-1.244.206l-.696-.442a6.073 6.073 0 01-1.387.8l-.082.827A1 1 0 0110.74 18H9.26a1 1 0 01-.997-.917l-.082-.827a6.073 6.073 0 01-1.387-.8l-.696.442a1 1 0 01-1.244-.206l-1.045-1.045a1 1 0 01-.206-1.244l.442-.696a6.073 6.073 0 01-.8-1.387l-.827-.082A1 1 0 012 10.74V9.26a1 1 0 01.917-.997l.827-.082a6.073 6.073 0 01.8-1.387l-.442-.696a1 1 0 01.206-1.244l1.045-1.045a1 1 0 011.244-.206l.696.442a6.073 6.073 0 011.387-.8l.082-.827zM10 13a3 3 0 100-6 3 3 0 000 6z"
      fill="currentColor"
    />
  </svg>
);

export function SettingsView({ modelConfig, onClose }: SettingsViewProps) {
  const [selectedModel, setSelectedModel] = useState(modelConfig?.active ?? '');
  const [ollamaUrl, setOllamaUrl] = useState('http://127.0.0.1:11434');
  const [autoStart, setAutoStart] = useState(false);
  const [gatewayEnabled, setGatewayEnabled] = useState(false);
  const [gatewayPort, setGatewayPort] = useState('18789');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Agent provider settings.
  const [agentProvider, setAgentProvider] = useState<AgentProvider>('ollama');
  const [agentModel, setAgentModel] = useState('gpt-4o');
  const [agentApiKey, setAgentApiKey] = useState('');
  const [agentBaseUrl, setAgentBaseUrl] = useState('https://api.openai.com/v1');

  // Load current settings on mount.
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const url = await invoke<string>('get_ollama_url');
        setOllamaUrl(url);
      } catch { /* use default */ }

      try {
        const settings = await invoke<Record<string, string>>('get_settings');
        if (settings['active_model']) setSelectedModel(settings['active_model']);
        if (settings['gateway_enabled'] === 'true') setGatewayEnabled(true);
        if (settings['gateway_port']) setGatewayPort(settings['gateway_port']);
      } catch { /* use defaults */ }

      try {
        const enabled = await invoke<boolean>('is_auto_start_enabled_command');
        setAutoStart(enabled);
      } catch { /* not available */ }

      // Load agent provider config.
      try {
        const provider = await invoke<{ provider: string; model: string; base_url: string; has_api_key: boolean }>('get_agent_provider');
        if (provider.provider) {
          setAgentProvider(provider.provider as AgentProvider);
          setAgentModel(provider.model || PROVIDER_MODELS[provider.provider as AgentProvider][0]);
          setAgentBaseUrl(provider.base_url || PROVIDER_URLS[provider.provider as AgentProvider]);
        }
      } catch { /* not set yet, use defaults */ }

      // Load persisted API key from app_config.
      try {
        const settings = await invoke<Record<string, string>>('get_settings');
        const prov = settings['agent_provider'] || 'ollama';
        if (prov !== 'ollama' && settings[`api_key_${prov}`]) {
          setAgentApiKey(settings[`api_key_${prov}`]);
        }
        if (settings['agent_model']) setAgentModel(settings['agent_model']);
        if (settings['agent_base_url']) setAgentBaseUrl(settings['agent_base_url']);
      } catch { /* use defaults */ }
    };
    void loadSettings();
  }, []);

  // Update agent model and URL when provider changes.
  useEffect(() => {
    setAgentModel(PROVIDER_MODELS[agentProvider][0]);
    setAgentBaseUrl(PROVIDER_URLS[agentProvider]);
  }, [agentProvider]);

  const handleSave = useCallback(async () => {
    setSaving(true);
    setError(null);
    setSuccess(null);

    try {
      // Save model selection.
      if (selectedModel && selectedModel !== modelConfig?.active) {
        await invoke('set_active_model', { model: selectedModel });
      }

      // Save Ollama URL.
      await invoke('set_ollama_url', { url: ollamaUrl });

      // Save gateway settings.
      await invoke('set_setting', { key: 'gateway_enabled', value: gatewayEnabled ? 'true' : 'false' });
      await invoke('set_setting', { key: 'gateway_port', value: gatewayPort });

      // Save auto-start.
      if (autoStart) {
        await invoke('enable_auto_start_command');
      } else {
        await invoke('disable_auto_start_command');
      }

      // Save agent provider config.
      await invoke('set_agent_provider', {
        provider: agentProvider,
        model: agentModel,
        baseUrl: agentBaseUrl,
        apiKey: agentApiKey,
      });

      // Persist API key to app_config for survival across restarts.
      await invoke('set_setting', { key: 'agent_provider', value: agentProvider });
      await invoke('set_setting', { key: 'agent_model', value: agentModel });
      await invoke('set_setting', { key: 'agent_base_url', value: agentBaseUrl });
      if (agentProvider !== 'ollama') {
        await invoke('set_setting', { key: `api_key_${agentProvider}`, value: agentApiKey });
      }

      setSuccess('Settings saved');
      setTimeout(() => setSuccess(null), 2000);
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }, [selectedModel, ollamaUrl, autoStart, gatewayEnabled, gatewayPort, modelConfig, agentProvider, agentModel, agentBaseUrl, agentApiKey]);

  return (
    <motion.div
      className="fixed inset-0 z-50 flex items-center justify-center"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.15 }}
    >
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/60" onClick={onClose} />

      {/* Settings panel */}
      <motion.div
        className="relative z-10 w-full max-w-sm bg-surface-base border border-surface-border rounded-xl shadow-lg overflow-hidden"
        initial={{ scale: 0.95, y: 20 }}
        animate={{ scale: 1, y: 0 }}
        exit={{ scale: 0.95, y: 20 }}
        transition={{ duration: 0.15 }}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-surface-border">
          <h2 className="text-sm font-semibold text-text-primary">Settings</h2>
          <button
            onClick={onClose}
            className="text-text-secondary hover:text-text-primary transition-colors"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M4 4L12 12M12 4L4 12" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
        </div>

        {/* Body */}
        <div className="px-4 py-3 space-y-4 max-h-[60vh] overflow-y-auto">
          {/* Chat Model (Ollama) */}
          <div>
            <label className="block text-xs font-medium text-text-secondary mb-1">
              Chat Model (Ollama)
            </label>
            {modelConfig && modelConfig.all.length > 0 ? (
              <select
                value={selectedModel}
                onChange={(e) => setSelectedModel(e.target.value)}
                className="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary"
              >
                {modelConfig.all.map((m) => (
                  <option key={m} value={m}>{m}</option>
                ))}
              </select>
            ) : (
              <input
                type="text"
                value={selectedModel}
                onChange={(e) => setSelectedModel(e.target.value)}
                placeholder="Model name"
                className="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary"
              />
            )}
          </div>

          {/* Ollama URL */}
          <div>
            <label className="block text-xs font-medium text-text-secondary mb-1">
              Ollama URL
            </label>
            <input
              type="text"
              value={ollamaUrl}
              onChange={(e) => setOllamaUrl(e.target.value)}
              placeholder="http://127.0.0.1:11434"
              className="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary"
            />
          </div>

          {/* Agent Provider section */}
          <div className="pt-2 border-t border-surface-border">
            <h3 className="text-xs font-semibold text-text-secondary uppercase tracking-wider mb-2">
              Agent Mode (Computer Use)
            </h3>

            <div className="space-y-3">
              {/* Provider selector */}
              <div>
                <label className="block text-xs font-medium text-text-secondary mb-1">
                  Provider
                </label>
                <select
                  value={agentProvider}
                  onChange={(e) => setAgentProvider(e.target.value as AgentProvider)}
                  className="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary"
                >
                  <option value="ollama">Ollama (Local)</option>
                  <option value="openai">OpenAI</option>
                  <option value="anthropic">Anthropic</option>
                </select>
              </div>

              {/* Model selector */}
              <div>
                <label className="block text-xs font-medium text-text-secondary mb-1">
                  Agent Model
                </label>
                <select
                  value={agentModel}
                  onChange={(e) => setAgentModel(e.target.value)}
                  className="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary"
                >
                  {PROVIDER_MODELS[agentProvider].map((m) => (
                    <option key={m} value={m}>{m}</option>
                  ))}
                </select>
              </div>

              {/* API Key (only for cloud providers) */}
              {agentProvider !== 'ollama' && (
                <div>
                  <label className="block text-xs font-medium text-text-secondary mb-1">
                    API Key
                  </label>
                  <input
                    type="password"
                    value={agentApiKey}
                    onChange={(e) => setAgentApiKey(e.target.value)}
                    placeholder={agentProvider === 'openai' ? 'sk-...' : 'sk-ant-...'}
                    className="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary"
                  />
                </div>
              )}

              {/* Base URL (editable for custom endpoints) */}
              <div>
                <label className="block text-xs font-medium text-text-secondary mb-1">
                  Base URL
                </label>
                <input
                  type="text"
                  value={agentBaseUrl}
                  onChange={(e) => setAgentBaseUrl(e.target.value)}
                  className="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary"
                />
              </div>
            </div>
          </div>

          {/* Auto-start */}
          <div className="flex items-center justify-between">
            <label className="text-xs font-medium text-text-secondary">
              Start on Boot
            </label>
            <button
              onClick={() => setAutoStart(!autoStart)}
              className={`relative w-10 h-5 rounded-full transition-colors ${
                autoStart ? 'bg-primary' : 'bg-surface-elevated'
              }`}
            >
              <span
                className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
                  autoStart ? 'left-5' : 'left-0.5'
                }`}
              />
            </button>
          </div>

          {/* Gateway */}
          <div>
            <div className="flex items-center justify-between mb-1">
              <label className="text-xs font-medium text-text-secondary">
                Local Gateway
              </label>
              <button
                onClick={() => setGatewayEnabled(!gatewayEnabled)}
                className={`relative w-10 h-5 rounded-full transition-colors ${
                  gatewayEnabled ? 'bg-primary' : 'bg-surface-elevated'
                }`}
              >
                <span
                  className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
                    gatewayEnabled ? 'left-5' : 'left-0.5'
                  }`}
                />
              </button>
            </div>
            {gatewayEnabled && (
              <input
                type="text"
                value={gatewayPort}
                onChange={(e) => setGatewayPort(e.target.value)}
                placeholder="18789"
                className="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary"
              />
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="px-4 py-3 border-t border-surface-border flex items-center justify-between">
          {error && <span className="text-xs text-red-400">{error}</span>}
          {success && <span className="text-xs text-emerald-400">{success}</span>}
          {!error && !success && <span />}
          <button
            onClick={handleSave}
            disabled={saving}
            className="px-4 py-1.5 bg-primary text-white text-sm font-medium rounded-lg hover:bg-primary/90 disabled:opacity-50 transition-colors"
          >
            {saving ? 'Saving...' : 'Save'}
          </button>
        </div>
      </motion.div>
    </motion.div>
  );
}