import { describe, expect, it } from 'vitest';
import { formatBytes, formatDateTime, relativeToRoots } from './format';

describe('formatBytes', () => {
  it.each([
    [0, '0 B'],
    [-5, '0 B'],
    [512, '512 B'],
    [1536, '1.5 KB'],
    [10 * 1024, '10 KB'],
    [5.25 * 1024 ** 2, '5.3 MB'],
    [3 * 1024 ** 3, '3.0 GB'],
    [2 * 1024 ** 5, '2048 TB'],
  ])('formats %d as %s', (bytes, expected) => {
    expect(formatBytes(bytes)).toBe(expected);
  });
});

describe('relativeToRoots', () => {
  it('strips the matching root on POSIX and Windows paths', () => {
    expect(
      relativeToRoots('/Users/me/Library/Caches/app/a.db', [
        '/Users/me/Library/Caches',
      ]),
    ).toBe('app/a.db');
    expect(relativeToRoots('C:\\Temp\\x\\y.tmp', ['C:\\Temp'])).toBe(
      'x\\y.tmp',
    );
  });

  it('does not treat a sibling folder with the same prefix as inside the root', () => {
    expect(relativeToRoots('/tmp2/file', ['/tmp'])).toBe('/tmp2/file');
  });
});

describe('formatDateTime', () => {
  it('formats with the given locale', () => {
    const ms = Date.UTC(2026, 0, 5, 12, 0);
    expect(formatDateTime(ms, 'en-US')).toMatch(/2026/);
  });
});
