import { render, screen, fireEvent, act } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { PermissionsStep } from '../PermissionsStep';
import { invoke } from '../../../testUtils/mocks/tauri';
import { isWindows } from '../../../utils/platform';

vi.mock('../../../utils/platform', () => ({
  isWindows: vi.fn(() => false),
}));

const mockIsWindows = isWindows as unknown as ReturnType<typeof vi.fn>;

describe('PermissionsStep', () => {
  beforeEach(() => {
    invoke.mockClear();
    invoke.mockResolvedValue(true);
    mockIsWindows.mockReturnValue(false);
  });

  it('renders the title', () => {
    render(<PermissionsStep />);
    expect(screen.getByText(/Let's get Thuki set up/i)).toBeInTheDocument();
  });

  it('renders Accessibility step label', () => {
    render(<PermissionsStep />);
    expect(screen.getByText('Accessibility')).toBeInTheDocument();
  });

  it('renders Screen Recording step label', () => {
    render(<PermissionsStep />);
    expect(screen.getByText('Screen Recording')).toBeInTheDocument();
  });

  it('shows Grant Accessibility button when not granted', () => {
    mockIsWindows.mockReturnValue(false);
    invoke.mockResolvedValue(false);
    render(<PermissionsStep />);
    expect(
      screen.getByRole('button', { name: /Grant Accessibility/i }),
    ).toBeInTheDocument();
  });

  it('auto-grants permissions on Windows', () => {
    mockIsWindows.mockReturnValue(true);
    render(<PermissionsStep />);
    expect(
      screen.getByRole('button', { name: /Quit and Reopen/i }),
    ).toBeInTheDocument();
  });

  it('calls open_accessibility_settings when Grant Accessibility clicked', async () => {
    mockIsWindows.mockReturnValue(false);
    invoke.mockResolvedValue(false);
    render(<PermissionsStep />);

    const button = screen.getByRole('button', {
      name: /Grant Accessibility/i,
    });
    await act(async () => {
      fireEvent.click(button);
    });

    expect(invoke).toHaveBeenCalledWith('open_accessibility_settings');
  });
});
