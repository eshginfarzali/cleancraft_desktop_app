import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App';
import type { CleanReport, ScanResult } from './lib/api';

const scans: ScanResult[] = [
  {
    id: 'temp',
    label: 'Temporary files',
    path: '/tmp',
    fileCount: 3,
    totalBytes: 3 * 1024 ** 2,
    skipped: 0,
  },
  {
    id: 'user-cache',
    label: 'Application caches',
    path: '/home/me/.cache',
    fileCount: 0,
    totalBytes: 0,
    skipped: 2,
  },
];

const handler = vi.fn((cmd: string, args?: unknown): unknown => {
  if (cmd === 'scan_targets') return scans;
  if (cmd === 'clean_targets') {
    const { ids } = args as { ids: string[] };
    return ids.map((id): CleanReport => ({
      id,
      deletedFiles: 3,
      freedBytes: 3 * 1024 ** 2,
      failed: 1,
    }));
  }
  throw new Error(`unexpected command ${cmd}`);
});

beforeEach(() => {
  handler.mockClear();
  mockIPC(handler);
});

afterEach(() => {
  cleanup();
  clearMocks();
});

describe('App', () => {
  it('scans, pre-selects folders with files and shows their paths', async () => {
    render(<App />);
    fireEvent.click(screen.getByRole('button', { name: 'Scan' }));

    expect(await screen.findByText('/tmp')).toBeTruthy();
    expect(screen.getByText('3 files · 3.0 MB')).toBeTruthy();
    expect(screen.getByText(/2 folders not readable/)).toBeTruthy();

    const [temp, cache] = screen.getAllByRole('checkbox') as HTMLInputElement[];
    expect(temp.checked).toBe(true);
    expect(cache.checked).toBe(false);
    expect(cache.disabled).toBe(true);
  });

  it('asks for confirmation and only sends target ids to the backend', async () => {
    render(<App />);
    fireEvent.click(screen.getByRole('button', { name: 'Scan' }));
    fireEvent.click(
      await screen.findByRole('button', { name: 'Clean selected (3.0 MB)' }),
    );

    expect(screen.getByRole('alertdialog').textContent).toContain(
      'Permanently delete 3.0 MB',
    );
    expect(handler).not.toHaveBeenCalledWith(
      'clean_targets',
      expect.anything(),
    );

    fireEvent.click(screen.getByRole('button', { name: 'Delete' }));

    expect(await screen.findByText(/could not be deleted/)).toBeTruthy();
    expect(screen.getByLabelText('Clean results').textContent).toContain(
      'Freed 3.0 MB by deleting 3 files.',
    );
    expect(handler).toHaveBeenCalledWith(
      'clean_targets',
      expect.objectContaining({ ids: ['temp'] }),
    );
  });

  it('cancelling the confirmation deletes nothing', async () => {
    render(<App />);
    fireEvent.click(screen.getByRole('button', { name: 'Scan' }));
    fireEvent.click(
      await screen.findByRole('button', { name: 'Clean selected (3.0 MB)' }),
    );
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(handler).not.toHaveBeenCalledWith(
      'clean_targets',
      expect.anything(),
    );
  });

  it('disables cleaning when nothing is selected', async () => {
    render(<App />);
    fireEvent.click(screen.getByRole('button', { name: 'Scan' }));
    const [temp] = (await screen.findAllByRole(
      'checkbox',
    )) as HTMLInputElement[];
    fireEvent.click(temp);

    expect(
      (
        screen.getByRole('button', {
          name: /Clean selected/,
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
  });

  it('shows backend errors', async () => {
    handler.mockImplementationOnce(() => {
      throw new Error('permission denied');
    });
    render(<App />);
    fireEvent.click(screen.getByRole('button', { name: 'Scan' }));

    expect((await screen.findByRole('alert')).textContent).toContain(
      'permission denied',
    );
  });
});
