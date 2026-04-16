import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { VoiceSelector } from '../VoiceSelector';
import type { TtsVoice } from '../../hooks/useTts';

const mockVoices: TtsVoice[] = [
  {
    name: 'Microsoft Server Speech Text to Speech Voice (tr-TR, EmelNeural)',
    ShortName: 'tr-TR-EmelNeural',
    gender: 'Female',
    Locale: 'tr-TR',
    SuggestedCodec: 'audio-24khz-48kbitrate-mono-mp3',
  },
  {
    name: 'Microsoft Server Speech Text to Speech Voice (tr-TR, AhmetNeural)',
    ShortName: 'tr-TR-AhmetNeural',
    gender: 'Male',
    Locale: 'tr-TR',
    SuggestedCodec: 'audio-24khz-48kbitrate-mono-mp3',
  },
  {
    name: 'Microsoft Server Speech Text to Speech Voice (en-US, JennyNeural)',
    ShortName: 'en-US-JennyNeural',
    gender: 'Female',
    Locale: 'en-US',
    SuggestedCodec: 'audio-24khz-48kbitrate-mono-mp3',
  },
];

const defaultProps = {
  voices: mockVoices,
  selectedVoice: 'tr-TR-EmelNeural',
  onVoiceChange: vi.fn(),
};

describe('VoiceSelector', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the selected voice label', () => {
    render(<VoiceSelector {...defaultProps} />);
    expect(screen.getByText('tr-TR-EmelNeural')).toBeInTheDocument();
  });

  it('opens dropdown on click', () => {
    render(<VoiceSelector {...defaultProps} />);
    const trigger = screen.getByLabelText('Select voice');
    fireEvent.click(trigger);
    expect(screen.getByPlaceholderText('Search voices...')).toBeInTheDocument();
  });

  it('calls onVoiceChange when a voice is selected', () => {
    render(<VoiceSelector {...defaultProps} />);
    const trigger = screen.getByLabelText('Select voice');
    fireEvent.click(trigger);
    const voiceButton = screen.getByText('en-US-JennyNeural');
    fireEvent.click(voiceButton);
    expect(defaultProps.onVoiceChange).toHaveBeenCalledWith(
      'en-US-JennyNeural',
    );
  });

  it('filters voices by search', () => {
    render(<VoiceSelector {...defaultProps} />);
    const trigger = screen.getByLabelText('Select voice');
    fireEvent.click(trigger);
    const input = screen.getByPlaceholderText('Search voices...');
    fireEvent.change(input, { target: { value: 'Jenny' } });
    expect(screen.getByText('en-US-JennyNeural')).toBeInTheDocument();
  });

  it('shows "No voices found" when search matches nothing', () => {
    render(<VoiceSelector {...defaultProps} />);
    const trigger = screen.getByLabelText('Select voice');
    fireEvent.click(trigger);
    const input = screen.getByPlaceholderText('Search voices...');
    fireEvent.change(input, { target: { value: 'zzzzz' } });
    expect(screen.getByText('No voices found')).toBeInTheDocument();
  });

  it('handles null voices gracefully', () => {
    render(
      <VoiceSelector
        voices={null as unknown as TtsVoice[]}
        selectedVoice="tr-TR-EmelNeural"
        onVoiceChange={vi.fn()}
      />,
    );
    expect(screen.getByText('tr-TR-EmelNeural')).toBeInTheDocument();
  });

  it('highlights the selected voice', () => {
    render(<VoiceSelector {...defaultProps} />);
    const trigger = screen.getByLabelText('Select voice');
    fireEvent.click(trigger);
    const allButtons = screen.getAllByText('tr-TR-EmelNeural');
    // The one in the dropdown list (inside a button) should be selected
    const dropdownButton = allButtons.find(
      (el) => el.closest('button')?.className.includes('bg-white/20'),
    );
    expect(dropdownButton).toBeTruthy();
  });

  it('shows gender indicator', () => {
    render(<VoiceSelector {...defaultProps} />);
    const trigger = screen.getByLabelText('Select voice');
    fireEvent.click(trigger);
    expect(screen.getAllByText('F').length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText('M').length).toBeGreaterThanOrEqual(1);
  });

  it('clears search on clear button click', () => {
    render(<VoiceSelector {...defaultProps} />);
    const trigger = screen.getByLabelText('Select voice');
    fireEvent.click(trigger);
    const input = screen.getByPlaceholderText('Search voices...');
    fireEvent.change(input, { target: { value: 'test' } });
    const clearButton = screen.getByLabelText('Clear search');
    fireEvent.click(clearButton);
    expect(input).toHaveValue('');
  });

  it('closes on Escape key', () => {
    render(<VoiceSelector {...defaultProps} />);
    const trigger = screen.getByLabelText('Select voice');
    fireEvent.click(trigger);
    expect(screen.getByPlaceholderText('Search voices...')).toBeInTheDocument();
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(
      screen.queryByPlaceholderText('Search voices...'),
    ).not.toBeInTheDocument();
  });

  it('closes on click outside', () => {
    render(
      <div>
        <div data-testid="outside">outside</div>
        <VoiceSelector {...defaultProps} />
      </div>,
    );
    const trigger = screen.getByLabelText('Select voice');
    fireEvent.click(trigger);
    expect(screen.getByPlaceholderText('Search voices...')).toBeInTheDocument();
    fireEvent.mouseDown(screen.getByTestId('outside'));
    expect(
      screen.queryByPlaceholderText('Search voices...'),
    ).not.toBeInTheDocument();
  });

  it('sorts selected locale group first', () => {
    render(<VoiceSelector {...defaultProps} />);
    const trigger = screen.getByLabelText('Select voice');
    fireEvent.click(trigger);
    const turkishHeader = screen.getByText('Turkish');
    const englishHeader = screen.getByText('English');
    expect(turkishHeader.compareDocumentPosition(englishHeader)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
  });
});