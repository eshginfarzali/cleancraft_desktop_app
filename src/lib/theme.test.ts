import { afterEach, describe, expect, it, vi } from 'vitest';
import { applyTheme, resolveTheme } from './theme';

function mockSystemDark(dark: boolean) {
  const listeners = new Set<() => void>();
  const query = {
    matches: dark,
    addEventListener: (_: string, fn: () => void) => listeners.add(fn),
    removeEventListener: (_: string, fn: () => void) => listeners.delete(fn),
  };
  vi.stubGlobal('matchMedia', () => query);
  return {
    query,
    listeners,
    flip(next: boolean) {
      query.matches = next;
      listeners.forEach((fn) => fn());
    },
  };
}

afterEach(() => {
  vi.unstubAllGlobals();
  delete document.documentElement.dataset.theme;
});

describe('theme', () => {
  it('uses explicit choices as-is', () => {
    mockSystemDark(true);
    expect(resolveTheme('light')).toBe('light');
    applyTheme('dark');
    expect(document.documentElement.dataset.theme).toBe('dark');
  });

  it('follows the OS for "system" and stops listening on cleanup', () => {
    const os = mockSystemDark(false);
    const stop = applyTheme('system');
    expect(document.documentElement.dataset.theme).toBe('light');
    os.flip(true);
    expect(document.documentElement.dataset.theme).toBe('dark');
    stop();
    expect(os.listeners.size).toBe(0);
  });
});
