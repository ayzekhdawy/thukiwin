/**
 * Agent tab - provider, model, base URL, and API key for agent mode.
 */

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

import { Section, TextField, Dropdown } from '../components';
import { SaveField } from '../components/SaveField';
import { configHelp } from '../configHelpers';
import type { RawAppConfig } from '../types';

type AgentProvider = 'ollama' | 'openai' | 'anthropic';

const PROVIDERS: AgentProvider[] = ['ollama', 'openai', 'anthropic'];
const PROVIDER_LABELS: Record<AgentProvider, string> = {
  ollama: 'Ollama (Local)',
  openai: 'OpenAI',
  anthropic: 'Anthropic',
};

interface AgentTabProps {
  config: RawAppConfig;
  resyncToken: number;
  onSaved: (next: RawAppConfig) => void;
}

export function AgentTab({ config, resyncToken, onSaved }: AgentTabProps) {
  const [apiKey, setApiKey] = useState('');
  const provider = config.agent.provider as AgentProvider;

  // Load API key from SQLite (not in TOML for security)
  useEffect(() => {
    async function loadApiKey() {
      try {
        const settings = await invoke<Record<string, string>>('get_settings');
        const prov = settings['agent_provider'] || 'ollama';
        if (prov !== 'ollama' && settings[`api_key_${prov}`]) {
          setApiKey(settings[`api_key_${prov}`]);
        }
      } catch {
        // not set yet
      }
    }
    void loadApiKey();
  }, []);

  async function saveApiKey(key: string) {
    try {
      if (provider !== 'ollama') {
        await invoke('set_setting', { key: `api_key_${provider}`, value: key });
      }
    } catch {
      // ignore
    }
  }

  return (
    <>
      <Section heading="Provider">
        <SaveField
          section="agent"
          fieldKey="provider"
          label="Provider"
          helper={configHelp('agent', 'provider')}
          initialValue={config.agent.provider}
          resyncToken={resyncToken}
          onSaved={onSaved}
          render={(value, setValue) => (
            <Dropdown
              value={value as AgentProvider}
              options={PROVIDERS}
              onChange={(next) => setValue(next)}
              ariaLabel="Agent provider"
            />
          )}
        />
      </Section>

      <Section heading="Model">
        <SaveField
          section="agent"
          fieldKey="model"
          label="Agent model"
          helper={configHelp('agent', 'model')}
          initialValue={config.agent.model}
          resyncToken={resyncToken}
          onSaved={onSaved}
          render={(value, setValue, errored) => (
            <TextField
              value={value}
              onChange={setValue}
              placeholder="e.g. llama3.2, gpt-4o, claude-sonnet-4-20250514"
              errored={errored}
              ariaLabel="Agent model"
            />
          )}
        />
      </Section>

      <Section heading="Connection">
        <SaveField
          section="agent"
          fieldKey="base_url"
          label="Base URL"
          helper={configHelp('agent', 'base_url')}
          initialValue={config.agent.base_url}
          resyncToken={resyncToken}
          onSaved={onSaved}
          render={(value, setValue, errored) => (
            <TextField
              value={value}
              onChange={setValue}
              placeholder={
                provider === 'openai'
                  ? 'https://api.openai.com/v1'
                  : provider === 'anthropic'
                    ? 'https://api.anthropic.com'
                    : 'http://127.0.0.1:11434'
              }
              errored={errored}
              ariaLabel="Agent base URL"
            />
          )}
        />
      </Section>

      {provider !== 'ollama' ? (
        <Section heading="API Key">
          <div className="flex flex-col gap-2">
            <label className="text-xs font-medium" style={{ color: 'var(--color-text-secondary)' }}>
              API Key ({PROVIDER_LABELS[provider]})
            </label>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              onBlur={() => saveApiKey(apiKey)}
              placeholder={provider === 'openai' ? 'sk-...' : 'sk-ant-...'}
              className="w-full bg-transparent border-b border-white/20 text-sm focus:outline-none focus:border-primary"
              style={{ color: 'var(--color-text-primary)', padding: '4px 0' }}
            />
            <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
              Stored securely in local database, not in config.toml.
            </span>
          </div>
        </Section>
      ) : null}
    </>
  );
}