import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { SpeakButton } from '../SpeakButton';

const mockOnSpeak = vi.fn();
const mockOnStop = vi.fn();
const mockOnAcknowledgePrivacy = vi.fn();

describe('SpeakButton', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders with correct aria-label in idle state', () => {
    render(
      <SpeakButton
        content="Hello world"
        messageId="msg-1"
        align="left"
        isSpeaking={false}
        onSpeak={mockOnSpeak}
        onStop={mockOnStop}
        privacyAcknowledged={true}
        onAcknowledgePrivacy={mockOnAcknowledgePrivacy}
      />,
    );
    expect(
      screen.getByRole('button', { name: 'Read aloud' }),
    ).toBeInTheDocument();
  });

  it('renders with correct aria-label in speaking state', () => {
    render(
      <SpeakButton
        content="Hello world"
        messageId="msg-1"
        align="left"
        isSpeaking={true}
        onSpeak={mockOnSpeak}
        onStop={mockOnStop}
        privacyAcknowledged={true}
        onAcknowledgePrivacy={mockOnAcknowledgePrivacy}
      />,
    );
    expect(
      screen.getByRole('button', { name: 'Stop reading' }),
    ).toBeInTheDocument();
  });

  it('calls onSpeak when clicked in idle state with privacy acknowledged', () => {
    render(
      <SpeakButton
        content="Hello world"
        messageId="msg-1"
        align="left"
        isSpeaking={false}
        onSpeak={mockOnSpeak}
        onStop={mockOnStop}
        privacyAcknowledged={true}
        onAcknowledgePrivacy={mockOnAcknowledgePrivacy}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Read aloud' }));
    expect(mockOnSpeak).toHaveBeenCalledWith('msg-1', 'Hello world');
  });

  it('calls onStop when clicked in speaking state', () => {
    render(
      <SpeakButton
        content="Hello world"
        messageId="msg-1"
        align="left"
        isSpeaking={true}
        onSpeak={mockOnSpeak}
        onStop={mockOnStop}
        privacyAcknowledged={true}
        onAcknowledgePrivacy={mockOnAcknowledgePrivacy}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Stop reading' }));
    expect(mockOnStop).toHaveBeenCalled();
  });

  it('shows privacy tooltip when privacy not acknowledged and button is clicked', () => {
    render(
      <SpeakButton
        content="Hello world"
        messageId="msg-1"
        align="left"
        isSpeaking={false}
        onSpeak={mockOnSpeak}
        onStop={mockOnStop}
        privacyAcknowledged={false}
        onAcknowledgePrivacy={mockOnAcknowledgePrivacy}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Read aloud' }));
    expect(mockOnSpeak).not.toHaveBeenCalled();
    expect(
      screen.getByText(/Text will be sent to Microsoft servers/i),
    ).toBeInTheDocument();
  });

  it('calls onAcknowledgePrivacy and onSpeak when privacy is acknowledged', () => {
    render(
      <SpeakButton
        content="Hello world"
        messageId="msg-1"
        align="left"
        isSpeaking={false}
        onSpeak={mockOnSpeak}
        onStop={mockOnStop}
        privacyAcknowledged={false}
        onAcknowledgePrivacy={mockOnAcknowledgePrivacy}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Read aloud' }));
    fireEvent.click(screen.getByRole('button', { name: /OK, read aloud/i }));
    expect(mockOnAcknowledgePrivacy).toHaveBeenCalled();
    expect(mockOnSpeak).toHaveBeenCalledWith('msg-1', 'Hello world');
  });

  it('dismisses privacy tooltip when cancel is clicked', () => {
    render(
      <SpeakButton
        content="Hello world"
        messageId="msg-1"
        align="left"
        isSpeaking={false}
        onSpeak={mockOnSpeak}
        onStop={mockOnStop}
        privacyAcknowledged={false}
        onAcknowledgePrivacy={mockOnAcknowledgePrivacy}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Read aloud' }));
    expect(
      screen.getByText(/Text will be sent to Microsoft servers/i),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(
      screen.queryByText(/Text will be sent to Microsoft servers/i),
    ).toBeNull();
    expect(mockOnSpeak).not.toHaveBeenCalled();
  });

  it('applies align="left" class', () => {
    const { container } = render(
      <SpeakButton
        content="test"
        messageId="m1"
        align="left"
        isSpeaking={false}
        onSpeak={mockOnSpeak}
        onStop={mockOnStop}
        privacyAcknowledged={true}
        onAcknowledgePrivacy={mockOnAcknowledgePrivacy}
      />,
    );
    const wrapper = container.firstElementChild;
    expect(wrapper?.classList.contains('justify-start')).toBe(true);
  });

  it('applies align="right" class', () => {
    const { container } = render(
      <SpeakButton
        content="test"
        messageId="m1"
        align="right"
        isSpeaking={false}
        onSpeak={mockOnSpeak}
        onStop={mockOnStop}
        privacyAcknowledged={true}
        onAcknowledgePrivacy={mockOnAcknowledgePrivacy}
      />,
    );
    const wrapper = container.firstElementChild;
    expect(wrapper?.classList.contains('justify-end')).toBe(true);
  });

  it('has active styling when speaking', () => {
    render(
      <SpeakButton
        content="test"
        messageId="m1"
        align="left"
        isSpeaking={true}
        onSpeak={mockOnSpeak}
        onStop={mockOnStop}
        privacyAcknowledged={true}
        onAcknowledgePrivacy={mockOnAcknowledgePrivacy}
      />,
    );
    const button = screen.getByRole('button', { name: 'Stop reading' });
    expect(button.classList.contains('text-white/70')).toBe(true);
  });

  it('has idle styling when not speaking', () => {
    render(
      <SpeakButton
        content="test"
        messageId="m1"
        align="left"
        isSpeaking={false}
        onSpeak={mockOnSpeak}
        onStop={mockOnStop}
        privacyAcknowledged={true}
        onAcknowledgePrivacy={mockOnAcknowledgePrivacy}
      />,
    );
    const button = screen.getByRole('button', { name: 'Read aloud' });
    expect(button.classList.contains('text-white/40')).toBe(true);
  });
});
