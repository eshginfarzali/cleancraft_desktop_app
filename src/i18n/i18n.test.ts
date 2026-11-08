import { describe, expect, it } from 'vitest';
import { az } from './az';
import { en } from './en';
import { targetLabel, translate } from './index';

const placeholders = (s: string) =>
  [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();

describe('dictionaries', () => {
  it('Azerbaijani defines every English key and nothing else', () => {
    expect(Object.keys(az).sort()).toEqual(Object.keys(en).sort());
  });

  it('every translation is non-empty and keeps the same placeholders', () => {
    for (const key of Object.keys(en) as (keyof typeof en)[]) {
      expect(az[key].trim(), key).not.toBe('');
      expect(placeholders(az[key]), key).toEqual(placeholders(en[key]));
    }
  });

  it('has a label for every target the Rust catalog defines', () => {
    const rustTargetIds = [
      'temp',
      'user-cache',
      'chrome',
      'edge',
      'firefox',
      'npm',
      'yarn',
      'pnpm',
      'xcode-derived-data',
    ];
    for (const id of rustTargetIds) {
      expect(`target.${id}` in en, id).toBe(true);
    }
  });
});

describe('translate', () => {
  it('fills placeholders and formats numbers for the language', () => {
    expect(
      translate('en', 'scan.target.detail', { count: 1200, size: '3 MB' }),
    ).toBe('1,200 files · 3 MB');
    expect(translate('az', 'tab.history')).toBe('Tarixçə');
  });

  it('leaves unknown placeholders visible', () => {
    expect(translate('en', 'scan.target.detail', {})).toBe(
      '{count} files · {size}',
    );
  });

  it('falls back to the id for unknown targets', () => {
    const t = (key: Parameters<typeof translate>[1]) => translate('en', key);
    expect(targetLabel(t, 'npm')).toBe('npm cache');
    expect(targetLabel(t, 'something-new')).toBe('something-new');
  });
});
