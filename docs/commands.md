# Commands

Commands are typed at the start of a message using the `/` prefix. Press `/` to open the command suggestion menu, then Tab to complete or Enter to select.

Commands that operate on text follow a consistent input priority:

1. **Highlighted text + no typed text:** highlighted text is the input
2. **No highlighted text + typed text after command:** typed text is the input
3. **Both present:** highlighted text is the primary input; typed text is appended as an additional instruction

This means you can highlight a paragraph anywhere on screen, summon Thuki with double-tap Control, type a command, and hit Enter without typing anything else.

---

## /screen

Captures your screen and attaches it as context for the current message.

**Usage:** `/screen [optional message]`

**Examples:**
- `/screen`: sends a screenshot with no additional message
- `/screen what is this error?`: attaches a screenshot and asks the question

**Behavior:** The screenshot is taken the moment you press Enter. Thuki's own window is excluded from the capture: no flicker, no hide. The image appears in your message bubble exactly like a pasted screenshot.

**Composable:** `/screen` works with all other commands. `/screen /rewrite` captures the screen and rewrites whatever text the model sees. `/screen /think` enables extended reasoning on the captured content.

**Limit:** One `/screen` capture per message. You may also attach up to 3 images manually (paste, drag, or the camera button) for a total of 4 images per message.

**Permission:** On Windows, no special permission is needed — screen capture works out of the box.

---

## /think

Enables extended reasoning before the model responds. The model works through the problem step by step internally before writing its answer.

**Usage:** `/think [optional message or highlighted text]`

**Examples:**
- `/think` (with highlighted text): reasons through the selected content
- `/think what are the tradeoffs of a monorepo vs polyrepo?`: asks a question with deep reasoning enabled

**Behavior:** A collapsible "Thinking" block appears above the response showing the model's reasoning chain. The final answer appears below it as normal.

**Composable:** `/think` works with all utility commands. `/think /tldr` summarizes with extended reasoning enabled.

---

## /translate

Translates text to another language.

**Usage:** `/translate [language] [text]` or `/translate` with highlighted text

**Examples:**
- `/translate` (with highlighted text): auto-detects language and translates. Non-English input translates to English; English input translates to Vietnamese
- `/translate ja` (with highlighted text): translates highlighted text to Japanese
- `/translate Spanish meeting notes here`: translates the typed text to Spanish

**Language format:** You can specify the target language by full name (`French`), ISO code (`fr`, `fra`), or common shorthand. The model interprets it flexibly.

**Default behavior:** If no language is specified, non-English input is translated to English and English input is translated to Vietnamese.

---

## /rewrite

Rewrites text to read more naturally and clearly.

**Usage:** `/rewrite [text]` or `/rewrite` with highlighted text

**Examples:**
- `/rewrite` (with highlighted text): rewrites the selected text
- `/rewrite so basically what happened was i was trying to fix the bug`: rewrites the typed text

**Behavior:** Preserves the original meaning while improving flow and readability. Output only: no commentary or explanation.

---

## /tldr

Summarizes text into 1-3 short, direct sentences.

**Usage:** `/tldr [text]` or `/tldr` with highlighted text

**Examples:**
- `/tldr` (with highlighted text): summarizes the selected content
- `/tldr [paste a long article]`: summarizes the typed or pasted text

**Behavior:** Captures the core message, key decision, or critical takeaway. Skips background detail and qualifications.

---

## /refine

Fixes grammar, spelling, and punctuation while preserving your voice.

**Usage:** `/refine [text]` or `/refine` with highlighted text

**Examples:**
- `/refine` (with highlighted text): corrects the selected text
- `/refine hey just wanted to follow up on the thing we discussed`: cleans up the typed text

**Behavior:** Corrects errors and smooths rough phrasing without restructuring or adding new ideas. Your original tone and meaning stay intact.

---

## /bullets

Extracts key points from text as a markdown bullet list.

**Usage:** `/bullets [text]` or `/bullets` with highlighted text

**Examples:**
- `/bullets` (with highlighted text): extracts key points from the selection
- `/bullets [paste meeting notes]`: extracts key points from the typed or pasted content

**Behavior:** Each point is a concise, self-contained statement. Ordered by importance or logical sequence. Filler and repetition are removed. Output is a `- ` prefixed markdown list.

---

## /todos

Summarizes what a piece of text is about, then extracts every task, action item, and commitment as a markdown checkbox list.

**Usage:** `/todos [text]` or `/todos` with highlighted text

**Examples:**
- `/todos` (with highlighted text): summarizes and extracts to-dos from the selected text
- `/todos [paste a conversation or notes]`: processes the typed or pasted content

**Behavior:** Responds in two parts: a short paragraph explaining the context and what is at stake, followed by a `- [ ]` checkbox list of all tasks. Each to-do includes who is responsible (if mentioned) and any deadline or timeframe. Observations and background that imply no action are excluded.

---

## /do

Enters agent mode: the AI autonomously controls your desktop to complete the described task. It takes screenshots, analyzes them with a vision model, and executes mouse/keyboard actions in a loop until the task is done.

**Usage:** `/do <task description>`

**Examples:**
- `/do Open Notepad and type Hello World`: launches Notepad and types the text
- `/do Find the Settings app and open it`: navigates the Start menu to find and open Settings
- `/do Close the current window`: presses Alt+F4 on the active window
- `/do Copy the selected text and paste it into a new document`: performs clipboard operations

**Behavior:**
1. A screenshot is captured (ThukiWin's own window is excluded)
2. The screenshot and your task are sent to a vision-capable Ollama model
3. The model returns actions (`CLICK x y`, `TYPE text`, `KEY_PRESS ctrl+c`, `LAUNCH app`, etc.)
4. Actions are executed via Win32 `SendInput` with a 300 ms delay between each
5. A new screenshot is taken and the loop repeats until the model outputs `DONE` or you stop it

**Safety:**
- Maximum 50 iterations per task (prevents infinite loops)
- 300 ms delay between actions so you can follow along
- Click the **Stop** button to cancel at any time
- A status indicator (amber banner with pulsing dot) shows the current phase: Capturing, Analyzing, or Executing

**Minibar integration:** When you switch away from ThukiWin while an agent task is running, the overlay shrinks to a minibar. The minibar shows a status dot (amber = running) and the last action. Click it to restore the full overlay and monitor progress.

**Vision model requirement:** Agent mode requires a vision-capable model such as `llama3.2-vision`. If you only have a text model installed, `/do` will fail with an error. Pull a vision model with `ollama pull llama3.2-vision`.

**Not composable:** `/do` does not combine with other commands. It is a standalone mode that takes over the input.
