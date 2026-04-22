import { describe, it, expect } from 'vitest';
import { detectComputerUseIntent } from '../intentDetection';

describe('detectComputerUseIntent', () => {
  it('returns null for empty string', () => {
    expect(detectComputerUseIntent('')).toBeNull();
  });

  it('returns null for whitespace-only string', () => {
    expect(detectComputerUseIntent('   ')).toBeNull();
  });

  it('returns null for normal chat', () => {
    expect(detectComputerUseIntent('Hello, how are you?')).toBeNull();
    expect(detectComputerUseIntent('What is 2+2?')).toBeNull();
    expect(detectComputerUseIntent('Explain quantum computing')).toBeNull();
  });

  // Vision intent — Turkish
  it('detects Turkish vision intent: ekrandaki ne', () => {
    expect(detectComputerUseIntent('ekrandaki ne görüyorsun')).toBe('vision');
  });

  it('detects Turkish vision intent: ekranda ne var', () => {
    expect(detectComputerUseIntent('ekranda ne var')).toBe('vision');
  });

  it('detects Turkish vision intent: ne görüyorsun', () => {
    expect(detectComputerUseIntent('ne görüyorsun')).toBe('vision');
  });

  it('detects Turkish vision intent: ekran görüntüsü', () => {
    expect(detectComputerUseIntent('ekran görüntüsü al')).toBe('vision');
  });

  // Vision intent — English
  it('detects English vision intent: what do you see', () => {
    expect(detectComputerUseIntent('What do you see on screen?')).toBe('vision');
  });

  it('detects English vision intent: what is on the screen', () => {
    expect(detectComputerUseIntent("What's on the screen?")).toBe('vision');
  });

  it('detects English vision intent: look at my screen', () => {
    expect(detectComputerUseIntent('Look at my screen')).toBe('vision');
  });

  it('detects English vision intent: describe the screen', () => {
    expect(detectComputerUseIntent('Describe the screen')).toBe('vision');
  });

  it('detects English vision intent: read the screen', () => {
    expect(detectComputerUseIntent('Read the screen')).toBe('vision');
  });

  it('detects English vision intent: what can you see', () => {
    expect(detectComputerUseIntent('What can you see?')).toBe('vision');
  });

  // Agent intent — Turkish with suffixes
  it('detects Turkish agent intent: not defterini açar mısın', () => {
    expect(detectComputerUseIntent('not defterini açar mısın')).toBe('agent');
  });

  it('detects Turkish agent intent: aç notepad', () => {
    expect(detectComputerUseIntent('Notepad aç')).toBe('agent');
  });

  it('detects Turkish agent intent: aç not defteri (suffix forms)', () => {
    expect(detectComputerUseIntent('not defterini aç')).toBe('agent');
    expect(detectComputerUseIntent('not defterini açsana')).toBe('agent');
  });

  it('detects Turkish agent intent: hesap makinesini aç', () => {
    expect(detectComputerUseIntent('hesap makinesini açar mısın')).toBe('agent');
  });

  // Agent intent — English
  it('detects agent intent: open notepad', () => {
    expect(detectComputerUseIntent('Open notepad for me')).toBe('agent');
  });

  it('detects agent intent: open calculator', () => {
    expect(detectComputerUseIntent('Open the calculator')).toBe('agent');
  });

  it('detects agent intent: click on the button', () => {
    expect(detectComputerUseIntent('Click on the button')).toBe('agent');
  });

  it('detects agent intent: type in the search', () => {
    expect(detectComputerUseIntent('Type in the search box')).toBe('agent');
  });

  it('detects agent intent: launch browser', () => {
    expect(detectComputerUseIntent('Launch the browser')).toBe('agent');
  });

  it('detects agent intent: open chrome', () => {
    expect(detectComputerUseIntent('open chrome')).toBe('agent');
  });

  it('detects /do command', () => {
    expect(detectComputerUseIntent('/do Open notepad')).toBe('agent');
  });

  it('agent intent overrides vision when both match', () => {
    expect(detectComputerUseIntent('Can you open notepad')).toBe('agent');
  });

  // False positives — should be null
  it('does not match "see" alone as vision', () => {
    expect(detectComputerUseIntent('I see what you mean')).toBeNull();
  });

  it('does not match "screen" alone without intent', () => {
    expect(detectComputerUseIntent('The screen resolution is 1920x1080')).toBeNull();
  });

  it('does not match unrelated "type" usage', () => {
    expect(detectComputerUseIntent('What type of music do you like?')).toBeNull();
    expect(detectComputerUseIntent('I like this type of book')).toBeNull();
  });
});