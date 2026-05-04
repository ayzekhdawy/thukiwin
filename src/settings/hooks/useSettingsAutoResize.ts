/**
 * Auto-sizes the Settings window to the active tab's content.
 */

import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';

const ANIMATE_MS = 220;
const MIN_HEIGHT = 280;
const MAX_HEIGHT = 700;
const SETTINGS_WIDTH = 580;
const NEGLIGIBLE_DELTA_PX = 4;

const easeOutCubic = (t: number) => 1 - Math.pow(1 - t, 3);

function clampHeight(h: number): number {
  return Math.max(MIN_HEIGHT, Math.min(MAX_HEIGHT, h));
}

export function useSettingsAutoResize(
  el: HTMLElement | null,
  chromeHeight: number,
  revision: unknown,
): boolean {
  const [isClamped, setIsClamped] = useState(false);
  const rafRef = useRef<number | null>(null);
  const initialisedRef = useRef(false);
  const lastSentRef = useRef<number | null>(null);
  const startTimeRef = useRef(0);
  const fromRef = useRef(0);
  const toRef = useRef(0);
  const chromeRef = useRef(chromeHeight);
  chromeRef.current = chromeHeight;

  const handleResizeRef = useRef<() => void>(() => {});

  useEffect(() => {
    if (!el) return;

    const cancelAnim = () => {
      if (rafRef.current !== null) {
        cancelAnimationFrame(rafRef.current);
        rafRef.current = null;
      }
    };

    const tick = (now: number) => {
      const elapsed = now - startTimeRef.current;
      const t = Math.min(1, elapsed / ANIMATE_MS);
      const eased = easeOutCubic(t);
      const h = Math.round(
        fromRef.current + (toRef.current - fromRef.current) * eased,
      );
      if (h !== lastSentRef.current) {
        lastSentRef.current = h;
        void getCurrentWindow().setSize(new LogicalSize(SETTINGS_WIDTH, h));
      }
      if (t < 1) {
        rafRef.current = requestAnimationFrame(tick);
      } else {
        rafRef.current = null;
      }
    };

    const handleResize = () => {
      const natural = el.scrollHeight + chromeRef.current;
      const target = clampHeight(natural);
      const shouldClamp = natural > MAX_HEIGHT;
      setIsClamped((prev) => (prev === shouldClamp ? prev : shouldClamp));
      if (!initialisedRef.current) {
        initialisedRef.current = true;
        lastSentRef.current = target;
        void getCurrentWindow().setSize(
          new LogicalSize(SETTINGS_WIDTH, target),
        );
        return;
      }
      const last = lastSentRef.current as number;
      if (Math.abs(target - last) < NEGLIGIBLE_DELTA_PX) return;
      cancelAnim();
      fromRef.current = last;
      toRef.current = target;
      startTimeRef.current = performance.now();
      rafRef.current = requestAnimationFrame(tick);
    };

    handleResizeRef.current = handleResize;
    const observer = new ResizeObserver(handleResize);
    observer.observe(el);
    handleResize();

    return () => {
      observer.disconnect();
      cancelAnim();
    };
  }, [el]);

  useLayoutEffect(() => {
    handleResizeRef.current();
  }, [revision, chromeHeight]);

  return isClamped;
}