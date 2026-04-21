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
  it('should render with default text when no lastMessage', () => {
    render(<MinibarView status={null} lastMessage={null} onClick={() => {}} />);
    expect(screen.getByText('ThukiWin')).toBeDefined();
  });

  it('should render lastMessage when provided', () => {
    render(<MinibarView status={null} lastMessage="I found the file" onClick={() => {}} />);
    expect(screen.getByText('I found the file')).toBeDefined();
  });

  it('should call onClick when clicked', () => {
    const onClick = vi.fn();
    render(<MinibarView status={null} lastMessage={null} onClick={onClick} />);
    fireEvent.click(screen.getByText('ThukiWin'));
    expect(onClick).toHaveBeenCalledOnce();
  });

  it('should render with idle status dot', () => {
    render(<MinibarView status="idle" lastMessage={null} onClick={() => {}} />);
    // The dot should have the emerald color class
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
});