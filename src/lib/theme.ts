import type { Theme } from './api';

const darkQuery = () =>
  typeof window.matchMedia === 'function'
    ? window.matchMedia('(prefers-color-scheme: dark)')
    : null;

export function resolveTheme(theme: Theme): 'light' | 'dark' {
  if (theme !== 'system') return theme;
  return darkQuery()?.matches ? 'dark' : 'light';
}

/**
 * Sets `data-theme` on <html>. For "system" it also follows OS changes;
 * the returned function stops listening.
 */
export function applyTheme(theme: Theme): () => void {
  const root = document.documentElement;
  root.dataset.theme = resolveTheme(theme);
  const query = darkQuery();
  if (theme !== 'system' || !query) return () => {};
  const onChange = () => {
    root.dataset.theme = resolveTheme('system');
  };
  query.addEventListener('change', onChange);
  return () => query.removeEventListener('change', onChange);
}
