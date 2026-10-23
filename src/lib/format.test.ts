import { describe, expect, it } from 'vitest';
import { formatBytes, pluralize } from './format';

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

describe('pluralize', () => {
  it('adds an s except for one', () => {
    expect(pluralize(1, 'file')).toBe('1 file');
    expect(pluralize(1200, 'file')).toBe('1,200 files');
  });
});
