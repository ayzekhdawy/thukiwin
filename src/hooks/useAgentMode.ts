import { useState, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export type AgentStatus =
  | 'idle'
  | 'capturing'
  | 'analyzing'
  | 'executing'
  | 'done'
  | 'error';

export interface AgentActionEvent {
  type: string;
  action: string;
  result: string;
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
  start: (task: string) => Promise<void>;
  stop: () => Promise<void>;
}

export function useAgentMode(modelConfig: { active: string } | null): UseAgentModeReturn {
  const [isActive, setIsActive] = useState(false);
  const [status, setStatus] = useState<AgentStatus>('idle');
  const [lastAction, setLastAction] = useState<string | null>(null);
  const [lastResult, setLastResult] = useState<string | null>(null);
  const [reasoning, setReasoning] = useState<string | null>(null);
  const [screenshotUrl, setScreenshotUrl] = useState<string | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  const start = useCallback(
    async (task: string) => {
      setIsActive(true);
      setStatus('capturing');
      setLastAction(null);
      setLastResult(null);
      setReasoning(null);
      setScreenshotUrl(null);

      // Listen for agent events.
      const unlisten = await listen<{
        type: string;
        data?: unknown;
      }>('thuki://agent', (event) => {
        const { type, data } = event.payload as { type: string; data?: unknown };

        switch (type) {
          case 'status_changed': {
            setStatus((data as AgentStatus) ?? 'idle');
            if ((data as AgentStatus) === 'done' || (data as AgentStatus) === 'error') {
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

      const model = modelConfig?.active ?? 'llama3.2-vision';
      const ollamaUrl = 'http://localhost:11434';

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
    if (unlistenRef.current) {
      unlistenRef.current();
      unlistenRef.current = null;
    }
  }, []);

  return {
    isActive,
    status,
    lastAction,
    lastResult,
    reasoning,
    screenshotUrl,
    start,
    stop,
  };
}