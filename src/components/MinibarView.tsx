import { motion } from 'framer-motion';
import { getCurrentWindow } from '@tauri-apps/api/window';
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

export function MinibarView({ status, onClick }: MinibarViewProps) {
  const dotColor = status ? statusDotColors[status] ?? 'bg-neutral-400' : 'bg-emerald-400';
  const isPulsing = status === 'executing' || status === 'analyzing' || status === 'capturing';

  const handlePointerDown = (e: React.PointerEvent) => {
    const startX = e.clientX;
    const startY = e.clientY;
    let resolved = false;

    const onMove = (moveEvent: PointerEvent) => {
      if (resolved) return;
      const dx = moveEvent.clientX - startX;
      const dy = moveEvent.clientY - startY;
      if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
        resolved = true;
        cleanup();
        void getCurrentWindow().startDragging();
      }
    };

    const onUp = () => {
      if (resolved) return;
      resolved = true;
      cleanup();
      onClick();
    };

    const cleanup = () => {
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
    };

    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
  };

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.8 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.8 }}
      transition={{ duration: 0.15 }}
      onPointerDown={handlePointerDown}
      className="relative w-12 h-12 rounded-full bg-surface-base cursor-pointer select-none flex items-center justify-center"
      title="ThukiWin — click to restore, drag to move"
    >
      <img
        src="thuki-logo.png"
        alt="ThukiWin"
        className="w-7 h-7 rounded-full pointer-events-none"
        draggable={false}
      />
      <span
        className={`absolute top-0 right-0 w-3 h-3 rounded-full ${dotColor} border-2 border-surface-base ${isPulsing ? 'animate-pulse' : ''}`}
      />
    </motion.div>
  );
}