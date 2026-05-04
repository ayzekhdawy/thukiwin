import { memo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import type { AgentStatus, AgentConfirmationEvent } from '../hooks/useAgentMode';

interface AgentIndicatorProps {
  isActive: boolean;
  status: AgentStatus;
  lastAction: string | null;
  reasoning: string | null;
  pendingConfirmation: AgentConfirmationEvent | null;
  onStop: () => void;
  onConfirm: (actionId: string) => void;
  onReject: (actionId: string) => void;
}

const statusLabels: Record<AgentStatus, string> = {
  idle: 'Agent',
  capturing: 'Capturing screen...',
  analyzing: 'Analyzing...',
  executing: 'Executing action...',
  waiting_confirmation: 'Confirm action...',
  done: 'Done',
  error: 'Error',
};

const statusColors: Record<AgentStatus, string> = {
  idle: 'bg-neutral-500',
  capturing: 'bg-amber-400',
  analyzing: 'bg-amber-400',
  executing: 'bg-amber-400',
  waiting_confirmation: 'bg-yellow-400',
  done: 'bg-emerald-400',
  error: 'bg-red-400',
};

const AgentConfirmation = memo(function AgentConfirmation({
  confirmation,
  onConfirm,
  onReject,
}: {
  confirmation: AgentConfirmationEvent;
  onConfirm: (actionId: string) => void;
  onReject: (actionId: string) => void;
}) {
  return (
    <div className="flex flex-col gap-2 p-3 bg-yellow-500/10 border border-yellow-500/30 rounded-lg">
      <p className="text-xs text-yellow-200 font-medium">Action requires confirmation:</p>
      <p className="text-xs text-neutral-300 leading-relaxed">{confirmation.description}</p>
      <div className="flex gap-2 mt-1">
        <button
          onClick={() => onConfirm(confirmation.action_id)}
          className="flex-1 px-3 py-1.5 rounded-md text-xs font-medium bg-amber-500/20 text-amber-200 hover:bg-amber-500/30 transition-colors cursor-pointer"
        >
          Allow
        </button>
        <button
          onClick={() => onReject(confirmation.action_id)}
          className="flex-1 px-3 py-1.5 rounded-md text-xs font-medium bg-neutral-700/50 text-neutral-400 hover:text-red-400 hover:bg-red-500/10 transition-colors cursor-pointer"
        >
          Deny
        </button>
      </div>
    </div>
  );
});

export function AgentIndicator({
  isActive,
  status,
  lastAction,
  reasoning,
  pendingConfirmation,
  onStop,
  onConfirm,
  onReject,
}: AgentIndicatorProps) {
  return (
    <AnimatePresence>
      {isActive && (
        <motion.div
          initial={{ opacity: 0, y: -8 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -8 }}
          transition={{ duration: 0.2 }}
          className="flex flex-col gap-1"
        >
          <div className="flex items-center gap-2 px-3 py-1.5 bg-amber-500/10 border border-amber-500/30 rounded-lg text-sm">
            <span className={`w-2 h-2 rounded-full ${statusColors[status]} animate-pulse`} />
            <span className="text-amber-200 font-medium">
              {statusLabels[status]}
            </span>
            {lastAction && (
              <span className="text-neutral-400 text-xs truncate max-w-[200px]">
                {lastAction}
              </span>
            )}
            {reasoning && status === 'analyzing' && (
              <span className="text-neutral-500 text-xs truncate max-w-[300px]">
                {reasoning}
              </span>
            )}
            <button
              onClick={onStop}
              className="ml-auto text-neutral-400 hover:text-red-400 transition-colors text-xs font-medium"
            >
              Stop
            </button>
          </div>
          {pendingConfirmation && (
            <AgentConfirmation
              confirmation={pendingConfirmation}
              onConfirm={onConfirm}
              onReject={onReject}
            />
          )}
        </motion.div>
      )}
    </AnimatePresence>
  );
}