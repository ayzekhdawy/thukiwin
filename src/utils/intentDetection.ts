/**
 * Intent detection for computer-use and vision queries.
 *
 * Analyzes user messages to determine if they're asking about screen content
 * or requesting a vision-based action. Turkish is an agglutinative language
 * so patterns must account for suffixes (e.g., defterini, açar, görüyorsun).
 */

/** Turkish and English patterns that indicate the user wants a screenshot analysis. */
const VISION_INTENT_PATTERNS: readonly RegExp[] = [
  // Turkish — screen/vision queries (flexible for suffixes)
  /\bekran.{0,4}ne\b/i,
  /\bne\s*(?:var|g[oö]r)[uüiı]/i,
  /\bg[oö]r[uü]nt[uü]/i,
  /\bekran\s*g[oö]r/i,
  /\bne\s*var\s*ekran/i,
  /\bekran[ıi]\s*(?:al|g[oö]ster|oku)/i,
  /\bscreenshot/i,
  // English — screen/vision queries
  /\bwhat\s+(?:do\s+you\s+)?see\b/i,
  /\bwhat'?s?\s+on\s+(?:the\s+|my\s+)?screen\b/i,
  /\blook\s+at\s+(?:the\s+|my\s+)?screen\b/i,
  /\bshow\s+me\s+(?:what\s+you\s+)?see\b/i,
  /\bdescribe\s+(?:the\s+|my\s+)?screen\b/i,
  /\bwhat\s+(?:is|are)\s+on\s+(?:the\s+|my\s+)?screen\b/i,
  /\bread\s+(?:the\s+|my\s+)?screen\b/i,
  /\bscreen\s+content\b/i,
  /\bwhat\s+can\s+you\s+see\b/i,
];

/** App names commonly used in desktop control commands. */
const APP_NAMES_TR = [
  'not\\s*defter',
  'hesap\\s*makine',
  'notepad',
  'calculator',
  'paint',
  'chrome',
  'firefox',
  'word',
  'excel',
  'powerpoint',
  'explorer',
  'cmd',
  'terminal',
  'vscode',
  'code',
  'discord',
  'spotify',
];

const APP_NAMES_EN = [
  'notepad',
  'calculator',
  'paint',
  'chrome',
  'firefox',
  'word',
  'excel',
  'powerpoint',
  'explorer',
  'cmd',
  'terminal',
  'vscode',
  'discord',
  'spotify',
  'browser',
  'app',
  'program',
  'file',
  'the',
];

/**
 * Build a regex alternation from an array of strings.
 * Returns "(a|b|c)" pattern string.
 */
function alt(patterns: readonly string[]): string {
  return `(?:${patterns.join('|')})`;
}

/** Patterns that indicate the user wants autonomous desktop control. */
const AGENT_INTENT_PATTERNS: readonly RegExp[] = [
  // Turkish — action verbs with suffixes: aç, açar, açsın, açıyor, açmak, açabilir, etc.
  new RegExp(`\\b(?:a[cç]|a[cç][aioıu][cklrnz]|a[cç]t[iı]m|a[cç]s[aı]n|a[cç]mak|a[cç]abil[oiır])\\s+${alt(APP_NAMES_TR)}`, 'i'),
  // Turkish — SOV order: "not defterini açar mısın" etc.
  new RegExp(`${alt(APP_NAMES_TR)}[^\\n]*\\b(?:a[cç]|a[cç][aioıu][cklrnz]|a[cç]abil)`, 'i'),
  // Turkish — general aç (open) with anything
  /\ba[cç](?:ar|s[ıi]n|t[ıi]m|mak|abil|[ıi]yor)?\s+(?:[a-zçğıöşüİı]{3,})/i,
  // Turkish — kapat, sil, taşı, vb.
  /\bkapat/i,
  /\bsil\s+(?:dosyayı|klasörü|file|folder)/i,
  // English — action-oriented requests
  new RegExp(`\\bopen\\s+${alt(APP_NAMES_EN)}`, 'i'),
  /\bclick\s+on\b/i,
  /\bpress\s+(?:the\s+)?(?:button|key)/i,
  /\btype\s+(?:in|into|the\s+\w+)\b/i,
  /\blaunch\s+\w/i,
  /\bstart\s+(?:the\s+)?(?:app|program)\b/i,
  // /do command explicitly
  /\/do\b/,
];

/**
 * Classifies the user's intent from their message.
 *
 * - `'vision'` — the user wants to see/describe what's on screen
 *   (auto-capture screenshot, send to vision model for description)
 * - `'agent'` — the user wants autonomous desktop control
 *   (trigger full agent loop with /do behavior)
 * - `null` — no computer-use intent detected
 *   (normal chat query)
 */
export function detectComputerUseIntent(message: string): 'vision' | 'agent' | null {
  const trimmed = message.trim();
  if (trimmed.length === 0) return null;

  // Agent intent takes priority — if they want actions, don't downgrade to vision.
  for (const pattern of AGENT_INTENT_PATTERNS) {
    if (pattern.test(trimmed)) return 'agent';
  }

  for (const pattern of VISION_INTENT_PATTERNS) {
    if (pattern.test(trimmed)) return 'vision';
  }

  return null;
}