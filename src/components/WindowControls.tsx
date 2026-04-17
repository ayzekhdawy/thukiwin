/**
 * Window controls bar for the overlay.
 *
 * Renders a thin header bar with functional buttons on the left
 * (save, new conversation, history) and Windows-style window controls
 * on the right (minimize, maximize, close).
 *
 * Window dragging is handled by the application root container via event
 * bubbling — mousedown events from the bar surface propagate up naturally.
 * A subtle divider at the bottom visually separates the controls from
 * the chat messages area below.
 */

import { memo } from 'react';
import { Tooltip } from './Tooltip';

/** Hoisted bookmark icon — save/saved state toggled via fill class. */
const BOOKMARK_ICON_EMPTY = (
  <svg
    width="13"
    height="13"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
  >
    <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />
  </svg>
);

const BOOKMARK_ICON_FILLED = (
  <svg
    width="13"
    height="13"
    viewBox="0 0 24 24"
    fill="currentColor"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
  >
    <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />
  </svg>
);

/** Hoisted new-conversation (plus) icon. */
const NEW_CONVERSATION_ICON = (
  <svg
    width="13"
    height="13"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
  >
    <line x1="12" y1="5" x2="12" y2="19" />
    <line x1="5" y1="12" x2="19" y2="12" />
  </svg>
);

/** Hoisted history (clock) icon. */
const HISTORY_ICON = (
  <svg
    width="13"
    height="13"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
  >
    <circle cx="12" cy="12" r="10" />
    <polyline points="12 6 12 12 16 14" />
  </svg>
);

interface WindowControlsProps {
  /** Triggers the overlay hide animation sequence. */
  onClose: () => void;
  /** Minimizes the window. */
  onMinimize: () => void;
  /**
   * Called when the user clicks the bookmark (save) icon.
   * Omit to hide the save button entirely.
   */
  onSave?: () => void;
  /**
   * True when the conversation has been saved to SQLite.
   * Renders the bookmark in its filled/confirmed state and disables the button.
   */
  isSaved?: boolean;
  /**
   * True when there is at least one completed AI response to save.
   * When false, the save button is disabled.
   */
  canSave?: boolean;
  /**
   * Called when the user clicks the "History ▾" button.
   * Omit to hide the history button entirely.
   */
  onHistoryOpen?: () => void;
  /**
   * Called when the user clicks the new-conversation (+) button.
   * Omit to hide the button entirely.
   */
  onNewConversation?: () => void;
}

export const WindowControls = memo(function WindowControls({
  onClose,
  onMinimize,
  onSave,
  isSaved = false,
  canSave = false,
  onHistoryOpen,
  onNewConversation,
}: WindowControlsProps) {
  const saveDisabled = !isSaved && !canSave;

  return (
    <div className="shrink-0">
      <div className="flex items-center h-8 px-2">
        {/* Left side: functional buttons */}
        <div className="flex items-center gap-1">
          {onSave !== undefined && (
            <Tooltip
              label={isSaved ? 'Remove from history (Ctrl+S)' : 'Save conversation (Ctrl+S)'}
            >
              <button
                type="button"
                onClick={onSave}
                disabled={saveDisabled}
                aria-label={
                  isSaved ? 'Remove from history' : 'Save conversation'
                }
                className={`w-7 h-7 flex items-center justify-center rounded-lg transition-colors duration-150 cursor-pointer disabled:cursor-default ${
                  isSaved
                    ? 'text-primary hover:text-text-secondary hover:bg-white/5'
                    : canSave
                      ? 'text-text-secondary hover:text-primary hover:bg-primary/8'
                      : 'text-text-secondary opacity-30'
                }`}
              >
                {isSaved ? BOOKMARK_ICON_FILLED : BOOKMARK_ICON_EMPTY}
              </button>
            </Tooltip>
          )}

          {onNewConversation !== undefined && (
            <Tooltip label="New conversation (Ctrl+N)">
              <button
                type="button"
                onClick={onNewConversation}
                aria-label="New conversation"
                data-history-toggle
                className="w-7 h-7 flex items-center justify-center rounded-lg text-text-secondary hover:text-text-primary hover:bg-white/5 transition-colors duration-150 cursor-pointer"
              >
                {NEW_CONVERSATION_ICON}
              </button>
            </Tooltip>
          )}

          {onHistoryOpen !== undefined && (
            <Tooltip label="Conversation history (Ctrl+H)">
              <button
                type="button"
                onClick={onHistoryOpen}
                aria-label="Open history"
                data-history-toggle
                className="w-7 h-7 flex items-center justify-center rounded-lg text-text-secondary hover:text-text-primary hover:bg-white/5 transition-colors duration-150 cursor-pointer"
              >
                {HISTORY_ICON}
              </button>
            </Tooltip>
          )}
        </div>

        {/* Right side: Windows window controls */}
        <div className="ml-auto flex items-center">
          <button
            type="button"
            onClick={onMinimize}
            className="win-title-btn win-title-btn-minimize"
            aria-label="Minimize"
          >
            <svg width="10" height="1" viewBox="0 0 10 1">
              <path d="M0 0.5h10" stroke="currentColor" strokeWidth="1" />
            </svg>
          </button>
          <button
            type="button"
            onClick={onClose}
            className="win-title-btn win-title-btn-close"
            aria-label="Close window"
          >
            <svg width="10" height="10" viewBox="0 0 10 10">
              <path
                d="M0 0L10 10M10 0L0 10"
                stroke="currentColor"
                strokeWidth="1.2"
              />
            </svg>
          </button>
        </div>
      </div>

      {/* Divider between controls and chat area */}
      <div className="h-px bg-surface-border" />
    </div>
  );
});
