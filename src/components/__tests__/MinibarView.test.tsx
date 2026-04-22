import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';
import { MinibarView } from '../MinibarView';

// Mock framer-motion
vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: React.HTMLAttributes<HTMLDivElement> & { initial?: unknown; animate?: unknown; exit?: unknown; transition?: unknown }) =>
      React.createElement('div', props, children),
  },
  AnimatePresence: ({ children }: { children: React.ReactNode }) =>
    React.createElement(React.Fragment, null, children),
}));

describe('MinibarView', () => {
  it('should render the logo icon', () => {
    render(<MinibarView status={null} lastMessage={null} onClick={() => {}} />);
    const img = screen.getByAltText('ThukiWin');
    expect(img).toBeDefined();
  });

  it('should call onClick on pointer down + up without movement', () => {
    const onClick = vi.fn();
    const { container } = render(<MinibarView status={null} lastMessage={null} onClick={onClick} />);
    const el = container.firstChild as HTMLElement;
    fireEvent.pointerDown(el, { clientX: 10, clientY: 10 });
    fireEvent.pointerUp(el, { clientX: 10, clientY: 10 });
    expect(onClick).toHaveBeenCalledOnce();
  });

  it('should NOT call onClick when pointer moves beyond threshold', () => {
    const onClick = vi.fn();
    const { container } = render(<MinibarView status={null} lastMessage={null} onClick={onClick} />);
    const el = container.firstChild as HTMLElement;
    fireEvent.pointerDown(el, { clientX: 10, clientY: 10 });
    // Simulate a move beyond the 3px threshold
    fireEvent.pointerMove(el, { clientX: 20, clientY: 10 });
    fireEvent.pointerUp(el, { clientX: 20, clientY: 10 });
    expect(onClick).not.toHaveBeenCalled();
  });

  it('should render with idle status dot', () => {
    render(<MinibarView status="idle" lastMessage={null} onClick={() => {}} />);
    const dot = document.querySelector('.bg-emerald-400');
    expect(dot).not.toBeNull();
  });

  it('should render with executing status dot', () => {
    render(<MinibarView status="executing" lastMessage={null} onClick={() => {}} />);
    const dot = document.querySelector('.bg-amber-400');
    expect(dot).not.toBeNull();
  });

  it('should render with error status dot', () => {
    render(<MinibarView status="error" lastMessage={null} onClick={() => {}} />);
    const dot = document.querySelector('.bg-red-400');
    expect(dot).not.toBeNull();
  });

  it('should pulse during active agent status', () => {
    render(<MinibarView status="analyzing" lastMessage={null} onClick={() => {}} />);
    const dot = document.querySelector('.animate-pulse');
    expect(dot).not.toBeNull();
  });
});