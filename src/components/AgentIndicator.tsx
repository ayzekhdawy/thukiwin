import { motion, AnimatePresence } from 'framer-motion';
import type { AgentStatus } from '../hooks/useAgentMode';

interface AgentIndicatorProps {
  isActive: boolean;
  status: AgentStatus;
  lastAction: string | null;
  reasoning: string | null;
  onStop: () => void;
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

export function AgentIndicator({
  isActive,
  status,
  lastAction,
  reasoning,
  onStop,
}: AgentIndicatorProps) {
  return (
    <AnimatePresence>
      {isActive && (
        <motion.div
          initial={{ opacity: 0, y: -8 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -8 }}
          transition={{ duration: 0.2 }}
          className="flex items-center gap-2 px-3 py-1.5 bg-amber-500/10 border border-amber-500/30 rounded-lg text-sm"
        >
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
        </motion.div>
      )}
    </AnimatePresence>
  );
}