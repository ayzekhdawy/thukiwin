import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

/** Provider type for agent mode. */
type AgentProvider = 'ollama' | 'openai' | 'anthropic';

/** Notification sound method. */
type NotificationSound = 'system' | 'custom' | 'none';

interface SettingsViewProps {
  modelConfig: { active: string; all: string[] } | null;
  onClose?: () => void;
  isStandalone?: boolean;
}

const PROVIDER_MODELS: Record<AgentProvider, string[]> = {
  ollama: ['gemini-3-flash-preview', 'llama3.2-vision', 'llama3.2', 'mistral'],
  openai: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo'],
  anthropic: ['claude-sonnet-4-20250514', 'claude-3-5-sonnet-20241022', 'claude-3-5-haiku-20241022'],
};

const PROVIDER_URLS: Record<AgentProvider, string> = {
  ollama: 'http://127.0.0.1:11434',
  openai: 'https://api.openai.com/v1',
  anthropic: 'https://api.anthropic.com',
};

export const SETTINGS_ICON = (
  <svg width="18" height="18" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
    <path fillRule="evenodd" clipRule="evenodd" d="M8.257 3.099A1 1 0 019.26 2h1.48a1 1 0 01.997.917l.082.827a6.073 6.073 0 011.387.8l.696-.442a1 1 0 011.244.206l1.045 1.045a1 1 0 01.206 1.244l-.442.696a6.073 6.073 0 01.8 1.387l.827.082A1 1 0 0118 9.26v1.48a1 1 0 01-.917.997l-.827.082a6.073 6.073 0 01-.8 1.387l.442.696a1 1 0 01-.206 1.244l-1.045 1.045a1 1 0 01-1.244.206l-.696-.442a6.073 6.073 0 01-1.387.8l-.082.827A1 1 0 0110.74 18H9.26a1 1 0 01-.997-.917l-.082-.827a6.073 6.073 0 01-1.387-.8l-.696.442a1 1 0 01-1.244-.206l-1.045-1.045a1 1 0 01-.206-1.244l.442-.696a6.073 6.073 0 01-.8-1.387l-.827-.082A1 1 0 012 10.74V9.26a1 1 0 01.917-.997l.827-.082a6.073 6.073 0 01.8-1.387l-.442-.696a1 1 0 01.206-1.244l1.045-1.045a1 1 0 011.244-.206l.696.442a6.073 6.073 0 011.387-.8l.082-.827zM10 13a3 3 0 100-6 3 3 0 000 6z" fill="currentColor" />
  </svg>
);

/** Toggle switch component. */
function Toggle({ value, onChange }: { value: boolean; onChange: () => void }) {
  return (
    <button
      onClick={onChange}
      className={`relative w-10 h-5 rounded-full transition-colors shrink-0 ${value ? 'bg-primary' : 'bg-surface-elevated'}`}
    >
      <span className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform ${value ? 'left-5' : 'left-0.5'}`} />
    </button>
  );
}

export function SettingsView({ modelConfig, onClose, isStandalone }: SettingsViewProps) {
  const [selectedModel, setSelectedModel] = useState(modelConfig?.active ?? '');
  const [ollamaUrl, setOllamaUrl] = useState('http://127.0.0.1:11434');
  const [autoStart, setAutoStart] = useState(false);
  const [gatewayEnabled, setGatewayEnabled] = useState(false);
  const [gatewayPort, setGatewayPort] = useState('18789');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const [agentProvider, setAgentProvider] = useState<AgentProvider>('ollama');
  const [agentModel, setAgentModel] = useState('gpt-4o');
  const [agentApiKey, setAgentApiKey] = useState('');
  const [agentBaseUrl, setAgentBaseUrl] = useState('https://api.openai.com/v1');

  const [notificationSound, setNotificationSound] = useState<NotificationSound>('system');
  const [ttsVoice, setTtsVoice] = useState('tr-TR-EmelNeural');
  const [ttsRate, setTtsRate] = useState(0);
  const [ttsPitch, setTtsPitch] = useState(0);
  const [ttsVoices, setTtsVoices] = useState<{ name: string; ShortName: string; Locale: string; gender: string }[]>([]);

  // Center settings window on mount.
  useEffect(() => {
    if (!isStandalone) return;
    void getCurrentWindow().center();
  }, [isStandalone]);

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
        if (settings['notification_sound']) setNotificationSound(settings['notification_sound'] as NotificationSound);
        if (settings['tts_voice']) setTtsVoice(settings['tts_voice']);
        if (settings['tts_rate']) setTtsRate(Number(settings['tts_rate']));
        if (settings['tts_pitch']) setTtsPitch(Number(settings['tts_pitch']));
      } catch { /* use defaults */ }

      try {
        const enabled = await invoke<boolean>('is_auto_start_enabled_command');
        setAutoStart(enabled);
      } catch { /* not available */ }

      try {
        const provider = await invoke<{ provider: string; model: string; base_url: string; has_api_key: boolean }>('get_agent_provider');
        if (provider.provider) {
          setAgentProvider(provider.provider as AgentProvider);
          setAgentModel(provider.model || PROVIDER_MODELS[provider.provider as AgentProvider][0]);
          setAgentBaseUrl(provider.base_url || PROVIDER_URLS[provider.provider as AgentProvider]);
        }
      } catch { /* not set yet */ }

      try {
        const settings = await invoke<Record<string, string>>('get_settings');
        const prov = settings['agent_provider'] || 'ollama';
        if (prov !== 'ollama' && settings[`api_key_${prov}`]) setAgentApiKey(settings[`api_key_${prov}`]);
        if (settings['agent_model']) setAgentModel(settings['agent_model']);
        if (settings['agent_base_url']) setAgentBaseUrl(settings['agent_base_url']);
      } catch { /* use defaults */ }

      const storedVoice = localStorage.getItem('tts_voice');
      if (storedVoice) setTtsVoice(storedVoice);

      try {
        const voices = await invoke<Array<{ name: string; ShortName: string; Locale: string; gender: string }>>('tts_list_voices');
        setTtsVoices(voices);
      } catch { /* TTS not available */ }
    };
    void loadSettings();
  }, []);

  useEffect(() => {
    setAgentModel(PROVIDER_MODELS[agentProvider][0]);
    setAgentBaseUrl(PROVIDER_URLS[agentProvider]);
  }, [agentProvider]);

  const handleSave = useCallback(async () => {
    setSaving(true);
    setError(null);
    setSuccess(null);
    try {
      if (selectedModel && selectedModel !== modelConfig?.active) {
        await invoke('set_active_model', { model: selectedModel });
      }
      await invoke('set_ollama_url', { url: ollamaUrl });
      await invoke('set_setting', { key: 'gateway_enabled', value: gatewayEnabled ? 'true' : 'false' });
      await invoke('set_setting', { key: 'gateway_port', value: gatewayPort });
      if (autoStart) { await invoke('enable_auto_start_command'); } else { await invoke('disable_auto_start_command'); }
      await invoke('set_agent_provider', { provider: agentProvider, model: agentModel, baseUrl: agentBaseUrl, apiKey: agentApiKey });
      await invoke('set_setting', { key: 'agent_provider', value: agentProvider });
      await invoke('set_setting', { key: 'agent_model', value: agentModel });
      await invoke('set_setting', { key: 'agent_base_url', value: agentBaseUrl });
      if (agentProvider !== 'ollama') { await invoke('set_setting', { key: `api_key_${agentProvider}`, value: agentApiKey }); }
      await invoke('set_setting', { key: 'notification_sound', value: notificationSound });
      await invoke('set_setting', { key: 'tts_voice', value: ttsVoice });
      await invoke('set_setting', { key: 'tts_rate', value: String(ttsRate) });
      await invoke('set_setting', { key: 'tts_pitch', value: String(ttsPitch) });
      localStorage.setItem('tts_voice', ttsVoice);
      setSuccess('Settings saved');
      setTimeout(() => setSuccess(null), 2000);
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }, [selectedModel, ollamaUrl, autoStart, gatewayEnabled, gatewayPort, modelConfig, agentProvider, agentModel, agentBaseUrl, agentApiKey, notificationSound, ttsVoice, ttsRate, ttsPitch]);

  const handleClose = useCallback(() => {
    if (onClose) { onClose(); return; }
    getCurrentWindow().hide().catch(() => {});
  }, [onClose]);

  const voicesByLocale = ttsVoices.reduce<Record<string, typeof ttsVoices>>((acc, v) => {
    const locale = v.Locale;
    if (!acc[locale]) acc[locale] = [];
    acc[locale].push(v);
    return acc;
  }, {});
  const sortedLocales = Object.keys(voicesByLocale).sort();

  const inputCls = 'w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary';
  const labelCls = 'block text-xs font-medium text-text-secondary mb-1';
  const sectionTitleCls = 'text-xs font-semibold text-text-secondary uppercase tracking-wider mb-3';

  return (
    <div className="w-full h-full bg-surface-base text-text-primary flex flex-col min-h-0">
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-3 border-b border-surface-border shrink-0" data-tauri-drag-region>
        <h2 className="text-sm font-semibold text-text-primary">Settings</h2>
        {isStandalone && (
          <button onClick={handleClose} className="text-text-secondary hover:text-text-primary transition-colors rounded-md hover:bg-surface-elevated p-1">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M4 4L12 12M12 4L4 12" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" /></svg>
          </button>
        )}
      </div>

      {/* Body - scrollable content */}
      <div className="px-5 py-4 space-y-5 overflow-y-auto flex-1 min-h-0">
        {/* ── Model ── */}
        <section>
          <h3 className={sectionTitleCls}>Model</h3>
          <div className="space-y-3">
            <div>
              <label className={labelCls}>Chat Model</label>
              <input
                type="text"
                list="model-list"
                value={selectedModel}
                onChange={(e) => setSelectedModel(e.target.value)}
                placeholder="Select or type a model name"
                className={inputCls}
              />
              <datalist id="model-list">
                {/* Ollama models from local */}
                {modelConfig?.all.map((m) => <option key={m} value={m} />)}
                {/* Cloud provider models */}
                <option value="gpt-4o" />
                <option value="gpt-4o-mini" />
                <option value="gpt-4-turbo" />
                <option value="claude-sonnet-4-20250514" />
                <option value="claude-3-5-sonnet-20241022" />
                <option value="claude-3-5-haiku-20241022" />
                <option value="gemini-3-flash-preview" />
                <option value="llama3.2-vision" />
                <option value="llama3.2" />
                <option value="mistral" />
              </datalist>
            </div>
            <div>
              <label className={labelCls}>Ollama URL</label>
              <input type="text" value={ollamaUrl} onChange={(e) => setOllamaUrl(e.target.value)} placeholder="http://127.0.0.1:11434" className={inputCls} />
            </div>
          </div>
        </section>

        {/* ── Sound ── */}
        <section>
          <h3 className={sectionTitleCls}>Sound</h3>
          <div className="space-y-3">
            <div>
              <label className={labelCls}>Notification Sound</label>
              <select value={notificationSound} onChange={(e) => setNotificationSound(e.target.value as NotificationSound)} className={inputCls}>
                <option value="system">System Default</option>
                <option value="custom">Custom Sound</option>
                <option value="none">Silent</option>
              </select>
            </div>
            <div>
              <label className={labelCls}>TTS Voice</label>
              <select value={ttsVoice} onChange={(e) => setTtsVoice(e.target.value)} className={inputCls}>
                {sortedLocales.length > 0 ? sortedLocales.map((locale) => (
                  <optgroup key={locale} label={locale}>
                    {voicesByLocale[locale].map((v) => <option key={v.ShortName} value={v.ShortName}>{v.ShortName} ({v.gender})</option>)}
                  </optgroup>
                )) : <option value={ttsVoice}>{ttsVoice}</option>}
              </select>
            </div>
            <div>
              <label className={labelCls}>TTS Speed</label>
              <div className="flex items-center gap-3">
                <span className="text-[10px] text-text-secondary w-8">Slow</span>
                <input type="range" min="-50" max="50" value={ttsRate} onChange={(e) => setTtsRate(Number(e.target.value))} className="flex-1 accent-primary" />
                <span className="text-[10px] text-text-secondary w-8 text-right">Fast</span>
              </div>
            </div>
            <div>
              <label className={labelCls}>TTS Pitch</label>
              <div className="flex items-center gap-3">
                <span className="text-[10px] text-text-secondary w-8">Low</span>
                <input type="range" min="-50" max="50" value={ttsPitch} onChange={(e) => setTtsPitch(Number(e.target.value))} className="flex-1 accent-primary" />
                <span className="text-[10px] text-text-secondary w-8 text-right">High</span>
              </div>
            </div>
          </div>
        </section>

        {/* ── Agent Mode ── */}
        <section>
          <h3 className={sectionTitleCls}>Agent Mode</h3>
          <div className="space-y-3">
            <div>
              <label className={labelCls}>Provider</label>
              <select value={agentProvider} onChange={(e) => setAgentProvider(e.target.value as AgentProvider)} className={inputCls}>
                <option value="ollama">Ollama (Local)</option>
                <option value="openai">OpenAI</option>
                <option value="anthropic">Anthropic</option>
              </select>
            </div>
            <div>
              <label className={labelCls}>Agent Model</label>
              <select value={agentModel} onChange={(e) => setAgentModel(e.target.value)} className={inputCls}>
                {PROVIDER_MODELS[agentProvider].map((m) => <option key={m} value={m}>{m}</option>)}
              </select>
            </div>
            {agentProvider !== 'ollama' && (
              <div>
                <label className={labelCls}>API Key</label>
                <input type="password" value={agentApiKey} onChange={(e) => setAgentApiKey(e.target.value)} placeholder={agentProvider === 'openai' ? 'sk-...' : 'sk-ant-...'} className={inputCls} />
              </div>
            )}
            <div>
              <label className={labelCls}>Base URL</label>
              <input type="text" value={agentBaseUrl} onChange={(e) => setAgentBaseUrl(e.target.value)} className={inputCls} />
            </div>
          </div>
        </section>

        {/* ── System ── */}
        <section>
          <h3 className={sectionTitleCls}>System</h3>
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <label className="text-xs font-medium text-text-secondary">Start on Boot</label>
              <Toggle value={autoStart} onChange={() => setAutoStart(!autoStart)} />
            </div>
            <div>
              <div className="flex items-center justify-between mb-1">
                <label className="text-xs font-medium text-text-secondary">Local Gateway</label>
                <Toggle value={gatewayEnabled} onChange={() => setGatewayEnabled(!gatewayEnabled)} />
              </div>
              {gatewayEnabled && (
                <input type="text" value={gatewayPort} onChange={(e) => setGatewayPort(e.target.value)} placeholder="18789" className={inputCls} />
              )}
            </div>
          </div>
        </section>
      </div>

      {/* Footer */}
      <div className="px-5 py-3 border-t border-surface-border flex items-center justify-between shrink-0">
        {error && <span className="text-xs text-red-400">{error}</span>}
        {success && <span className="text-xs text-emerald-400">{success}</span>}
        {!error && !success && <span />}
        <button onClick={handleSave} disabled={saving} className="px-5 py-1.5 bg-primary text-white text-sm font-medium rounded-lg hover:bg-primary/90 disabled:opacity-50 transition-colors">
          {saving ? 'Saving...' : 'Save'}
        </button>
      </div>
    </div>
  );
}