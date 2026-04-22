import { useState, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export type AgentStatus =
  | 'idle'
  | 'capturing'
  | 'analyzing'
  | 'executing'
  | 'waiting_confirmation'
  | 'done'
  | 'error';

export interface AgentActionEvent {
  type: string;
  action: string;
  result: string;
}

export interface AgentConfirmationEvent {
  action_id: string;
  action: string;
  description: string;
}

export interface AgentDoneEvent {
  summary: string;
}

interface UseAgentModeReturn {
  isActive: boolean;
  status: AgentStatus;
  lastAction: string | null;
  lastResult: string | null;
  reasoning: string | null;
  screenshotUrl: string | null;
  pendingConfirmation: AgentConfirmationEvent | null;
  start: (task: string) => Promise<void>;
  stop: () => Promise<void>;
  confirmAction: (actionId: string) => Promise<void>;
  rejectAction: (actionId: string) => Promise<void>;
}

export function useAgentMode(modelConfig: { active: string } | null): UseAgentModeReturn {
  const [isActive, setIsActive] = useState(false);
  const [status, setStatus] = useState<AgentStatus>('idle');
  const [lastAction, setLastAction] = useState<string | null>(null);
  const [lastResult, setLastResult] = useState<string | null>(null);
  const [reasoning, setReasoning] = useState<string | null>(null);
  const [screenshotUrl, setScreenshotUrl] = useState<string | null>(null);
  const [pendingConfirmation, setPendingConfirmation] = useState<AgentConfirmationEvent | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  const start = useCallback(
    async (task: string) => {
      setIsActive(true);
      setStatus('capturing');
      setLastAction(null);
      setLastResult(null);
      setReasoning(null);
      setScreenshotUrl(null);
      setPendingConfirmation(null);

      // Listen for agent events.
      const unlisten = await listen<{
        type: string;
        data?: unknown;
      }>('thuki://agent', (event) => {
        const { type, data } = event.payload as { type: string; data?: unknown };

        switch (type) {
          case 'status_changed': {
            const newStatus = (data as AgentStatus) ?? 'idle';
            setStatus(newStatus);
            if (newStatus === 'done' || newStatus === 'error') {
              setIsActive(false);
            }
            break;
          }
          case 'action_executed': {
            const d = data as { action: string; result: string };
            setLastAction(d.action);
            setLastResult(d.result);
            break;
          }
          case 'reasoning': {
            setReasoning(data as string);
            break;
          }
          case 'screenshot_taken': {
            setScreenshotUrl(data as string);
            break;
          }
          case 'confirmation_required': {
            const d = data as AgentConfirmationEvent;
            setPendingConfirmation(d);
            break;
          }
          case 'error': {
            setLastResult(data as string);
            setStatus('error');
            setIsActive(false);
            break;
          }
          case 'done': {
            const d = data as { summary: string };
            setReasoning(d.summary);
            setStatus('done');
            setIsActive(false);
            break;
          }
        }
      });

      unlistenRef.current = unlisten;

      // Check if a cloud provider is configured for agent mode.
      let providerConfig: { provider: string; model: string; base_url: string; has_api_key: boolean } | null = null;
      try {
        providerConfig = await invoke<{ provider: string; model: string; base_url: string; has_api_key: boolean }>('get_agent_provider');
      } catch {
        // No provider configured, fall back to Ollama.
      }

      const model = modelConfig?.active ?? 'llama3.2-vision';
      let ollamaUrl: string;
      try {
        ollamaUrl = await invoke<string>('get_ollama_url');
      } catch {
        ollamaUrl = 'http://127.0.0.1:11434';
      }

      // If a cloud provider is configured with an API key, set it before starting.
      if (providerConfig && providerConfig.provider !== 'ollama' && providerConfig.has_api_key) {
        try {
          await invoke('set_agent_provider', {
            provider: providerConfig.provider,
            model: providerConfig.model,
            baseUrl: providerConfig.base_url,
            apiKey: '', // Key is already stored in agent state from settings.
          });
        } catch {
          // Ignore, the provider config was already set from settings.
        }
      }

      try {
        await invoke('start_agent_mode', {
          task,
          model,
          ollamaUrl,
        });
      } catch (e) {
        console.error('Failed to start agent mode:', e);
        setStatus('error');
        setIsActive(false);
        setLastResult(String(e));
        unlisten();
      }
    },
    [modelConfig],
  );

  const stop = useCallback(async () => {
    try {
      await invoke('stop_agent_mode');
    } catch (e) {
      console.error('Failed to stop agent mode:', e);
    }
    setIsActive(false);
    setStatus('idle');
    setPendingConfirmation(null);
    if (unlistenRef.current) {
      unlistenRef.current();
      unlistenRef.current = null;
    }
  }, []);

  const confirmAction = useCallback(async (actionId: string) => {
    try {
      await invoke('confirm_agent_action', { actionId });
      setPendingConfirmation(null);
    } catch (e) {
      console.error('Failed to confirm action:', e);
    }
  }, []);

  const rejectAction = useCallback(async (actionId: string) => {
    try {
      await invoke('reject_agent_action', { actionId });
      setPendingConfirmation(null);
    } catch (e) {
      console.error('Failed to reject action:', e);
    }
  }, []);

  return {
    isActive,
    status,
    lastAction,
    lastResult,
    reasoning,
    screenshotUrl,
    pendingConfirmation,
    start,
    stop,
    confirmAction,
    rejectAction,
  };
}