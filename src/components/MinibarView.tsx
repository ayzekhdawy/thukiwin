import { motion } from 'framer-motion';
import type { AgentStatus } from '../hooks/useAgentMode';

interface MinibarViewProps {
  status: AgentStatus | null;
  lastMessage: string | null;
  onClick: () => void;
}

const statusDotColors: Record<string, string> = {
  idle: 'bg-emerald-400',
  capturing: 'bg-amber-400',
  analyzing: 'bg-amber-400',
  executing: 'bg-amber-400',
  done: 'bg-emerald-400',
  error: 'bg-red-400',
};

export function MinibarView({ status, lastMessage, onClick }: MinibarViewProps) {
  const dotColor = status ? statusDotColors[status] ?? 'bg-neutral-400' : 'bg-emerald-400';

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: 10 }}
      transition={{ duration: 0.2 }}
      onClick={onClick}
      className="flex items-center gap-2 px-3 h-[40px] bg-surface-base/95 backdrop-blur-xl rounded-lg cursor-pointer select-none shadow-bar border border-surface-border"
    >
      <span className={`w-2 h-2 rounded-full ${dotColor} ${status === 'executing' || status === 'analyzing' || status === 'capturing' ? 'animate-pulse' : ''}`} />
      {lastMessage ? (
        <span className="text-text-secondary text-xs truncate max-w-[500px]">
          {lastMessage}
        </span>
      ) : (
        <span className="text-text-secondary text-xs">ThukiWin</span>
      )}
    </motion.div>
  );
}