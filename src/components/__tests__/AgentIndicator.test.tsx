import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';
import { AgentIndicator } from '../AgentIndicator';

// Mock framer-motion
vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: React.HTMLAttributes<HTMLDivElement> & { initial?: unknown; animate?: unknown; exit?: unknown; transition?: unknown }) =>
      React.createElement('div', props, children),
  },
  AnimatePresence: ({ children }: { children: React.ReactNode }) =>
    React.createElement(React.Fragment, null, children),
}));

describe('AgentIndicator', () => {
  it('should not render when inactive', () => {
    render(
      <AgentIndicator
        isActive={false}
        status="idle"
        lastAction={null}
        reasoning={null}
        onStop={() => {}}
      />,
    );
    expect(screen.queryByText('Agent')).toBeNull();
  });

  it('should render when active with capturing status', () => {
    render(
      <AgentIndicator
        isActive={true}
        status="capturing"
        lastAction={null}
        reasoning={null}
        onStop={() => {}}
      />,
    );
    expect(screen.getByText('Capturing screen...')).toBeDefined();
  });

  it('should render when active with analyzing status', () => {
    render(
      <AgentIndicator
        isActive={true}
        status="analyzing"
        lastAction={null}
        reasoning="Looking for the Start button"
        onStop={() => {}}
      />,
    );
    expect(screen.getByText('Analyzing...')).toBeDefined();
  });

  it('should render last action when provided', () => {
    render(
      <AgentIndicator
        isActive={true}
        status="executing"
        lastAction="Click { x: 100, y: 200 }"
        reasoning={null}
        onStop={() => {}}
      />,
    );
    expect(screen.getByText('Executing action...')).toBeDefined();
    expect(screen.getByText('Click { x: 100, y: 200 }')).toBeDefined();
  });

  it('should call onStop when stop button is clicked', () => {
    const onStop = vi.fn();
    render(
      <AgentIndicator
        isActive={true}
        status="executing"
        lastAction={null}
        reasoning={null}
        onStop={onStop}
      />,
    );

    fireEvent.click(screen.getByText('Stop'));
    expect(onStop).toHaveBeenCalledOnce();
  });

  it('should render done status correctly', () => {
    render(
      <AgentIndicator
        isActive={true}
        status="done"
        lastAction={null}
        reasoning={null}
        onStop={() => {}}
      />,
    );
    expect(screen.getByText('Done')).toBeDefined();
  });

  it('should render error status correctly', () => {
    render(
      <AgentIndicator
        isActive={true}
        status="error"
        lastAction={null}
        reasoning={null}
        onStop={() => {}}
      />,
    );
    expect(screen.getByText('Error')).toBeDefined();
  });
});