import { render, screen, fireEvent, act } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { SwitchConfirmation } from '../SwitchConfirmation';

describe('SwitchConfirmation', () => {
  it('renders the default "switch" variant text', () => {
    render(
      <SwitchConfirmation
        onSaveAndSwitch={vi.fn()}
        onJustSwitch={vi.fn()}
        onCancel={vi.fn()}
      />,
    );
    expect(screen.getByText('Switch conversations?')).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /save & switch/i }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /just switch/i }),
    ).toBeInTheDocument();
  });

  it('renders the "new" variant text', () => {
    render(
      <SwitchConfirmation
        variant="new"
        onSaveAndSwitch={vi.fn()}
        onJustSwitch={vi.fn()}
        onCancel={vi.fn()}
      />,
    );
    expect(screen.getByText('New conversation?')).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /save & start new/i }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /^start new$/i }),
    ).toBeInTheDocument();
  });

  it('calls onSaveAndSwitch when the save button is clicked', () => {
    const onSaveAndSwitch = vi.fn();
    render(
      <SwitchConfirmation
        onSaveAndSwitch={onSaveAndSwitch}
        onJustSwitch={vi.fn()}
        onCancel={vi.fn()}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: /save & switch/i }));
    expect(onSaveAndSwitch).toHaveBeenCalledOnce();
  });

  it('calls onJustSwitch when the proceed button is clicked', () => {
    const onJustSwitch = vi.fn();
    render(
      <SwitchConfirmation
        onSaveAndSwitch={vi.fn()}
        onJustSwitch={onJustSwitch}
        onCancel={vi.fn()}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: /just switch/i }));
    expect(onJustSwitch).toHaveBeenCalledOnce();
  });

  it('calls onCancel when cancel is clicked', () => {
    const onCancel = vi.fn();
    render(
      <SwitchConfirmation
        onSaveAndSwitch={vi.fn()}
        onJustSwitch={vi.fn()}
        onCancel={onCancel}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: /cancel/i }));
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it('calls onCancel on Escape key', () => {
    const onCancel = vi.fn();
    render(
      <SwitchConfirmation
        onSaveAndSwitch={vi.fn()}
        onJustSwitch={vi.fn()}
        onCancel={onCancel}
      />,
    );
    act(() => {
      fireEvent.keyDown(document, { key: 'Escape' });
    });
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it('calls onSaveAndSwitch on Enter key', () => {
    const onSaveAndSwitch = vi.fn();
    render(
      <SwitchConfirmation
        onSaveAndSwitch={onSaveAndSwitch}
        onJustSwitch={vi.fn()}
        onCancel={vi.fn()}
      />,
    );
    act(() => {
      fireEvent.keyDown(document, { key: 'Enter' });
    });
    expect(onSaveAndSwitch).toHaveBeenCalledOnce();
  });

  it('ignores other keys', () => {
    const onSaveAndSwitch = vi.fn();
    const onCancel = vi.fn();
    render(
      <SwitchConfirmation
        onSaveAndSwitch={onSaveAndSwitch}
        onJustSwitch={vi.fn()}
        onCancel={onCancel}
      />,
    );
    act(() => {
      fireEvent.keyDown(document, { key: 'Tab' });
    });
    expect(onSaveAndSwitch).not.toHaveBeenCalled();
    expect(onCancel).not.toHaveBeenCalled();
  });
});
