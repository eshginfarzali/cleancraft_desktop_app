import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App';
import type { CleanReport, Settings } from './lib/api';
import * as fx from './test/fixtures';

let current: Settings;
const handler = vi.fn((cmd: string, args?: unknown): unknown => {
  switch (cmd) {
    case 'get_settings':
      return current;
    case 'save_settings':
      current = (args as { settings: Settings }).settings;
      return current;
    case 'get_targets':
      return fx.targets.map((t) => ({
        ...t,
        enabled: current.enabledTargets
          ? current.enabledTargets.includes(t.id)
          : t.enabled,
      }));
    case 'scan_targets':
      return fx.scans;
    case 'preview_target':
      return fx.preview;
    case 'clean_targets':
      return (args as { ids: string[] }).ids.map((id): CleanReport => ({
        id,
        deletedFiles: 3,
        freedBytes: 3 * 1024 ** 2,
        failed: 1,
        rejected: 0,
      }));
    case 'clean_selected': {
      const { id, paths } = args as { id: string; paths: string[] };
      return {
        id,
        deletedFiles: paths.length,
        freedBytes: 2 * 1024 ** 2,
        failed: 0,
        rejected: 0,
      };
    }
    case 'get_history':
      return fx.history;
    case 'clear_history':
      return null;
    // listen()/unlisten() for the history-updated event
    case 'plugin:event|listen':
      return 1;
    case 'plugin:event|unlisten':
      return null;
    default:
      throw new Error(`unexpected command ${cmd}`);
  }
});

beforeEach(() => {
  current = fx.settings();
  handler.mockClear();
  mockIPC(handler);
});

afterEach(async () => {
  cleanup();
  // Let unmount cleanups (unlisten) finish before the IPC mock is removed.
  await new Promise((resolve) => setTimeout(resolve, 0));
  clearMocks();
  delete document.documentElement.dataset.theme;
});

const calls = (cmd: string) => handler.mock.calls.filter(([c]) => c === cmd);

async function renderApp() {
  render(<App />);
  await screen.findByRole('tab', { name: 'Clean' });
}

describe('Clean tab', () => {
  it('scans, pre-selects folders with files and shows the freeable total', async () => {
    await renderApp();
    fireEvent.click(screen.getByRole('button', { name: 'Scan' }));

    expect(
      await screen.findByText('Up to 3.0 MB can be freed from 3 files.'),
    ).toBeTruthy();
    expect(screen.getByText('3 files · 3.0 MB')).toBeTruthy();
    expect(screen.getByText(/2 folders not readable/)).toBeTruthy();
    const [temp, cache] = screen.getAllByRole('checkbox') as HTMLInputElement[];
    expect(temp.checked).toBe(true);
    expect(cache.disabled).toBe(true);
  });

  it('cleans whole targets only after confirmation and sends ids only', async () => {
    await renderApp();
    fireEvent.click(screen.getByRole('button', { name: 'Scan' }));
    fireEvent.click(
      await screen.findByRole('button', { name: 'Clean selected (3.0 MB)' }),
    );
    expect(calls('clean_targets')).toHaveLength(0);

    fireEvent.click(screen.getByRole('button', { name: 'Delete' }));

    expect(
      await screen.findByText('Freed 3.0 MB by deleting 3 files.'),
    ).toBeTruthy();
    expect(screen.getByText(/1 files could not be deleted/)).toBeTruthy();
    expect(calls('clean_targets')[0]?.[1]).toEqual(
      expect.objectContaining({ ids: ['temp'] }),
    );
  });

  it('previews files and deletes only the selected ones', async () => {
    await renderApp();
    fireEvent.click(screen.getByRole('button', { name: 'Scan' }));
    const [previewTemp] = await screen.findAllByRole('button', {
      name: 'Preview files',
    });
    fireEvent.click(previewTemp!);

    const panel = await screen.findByRole('region', {
      name: 'Preview: Temporary files',
    });
    expect(within(panel).getByText('big.log')).toBeTruthy();
    expect(within(panel).getByText('a/small.tmp')).toBeTruthy();
    const deleteButton = within(panel).getByRole('button', {
      name: 'Delete selected (0 B)',
    });
    expect((deleteButton as HTMLButtonElement).disabled).toBe(true);

    fireEvent.click(within(panel).getByRole('checkbox', { name: 'big.log' }));
    expect(within(panel).getByText('1 of 2 selected · 2.0 MB')).toBeTruthy();
    fireEvent.click(
      within(panel).getByRole('button', { name: 'Delete selected (2.0 MB)' }),
    );
    fireEvent.click(within(panel).getByRole('button', { name: 'Delete' }));

    await waitFor(() => expect(calls('clean_selected')).toHaveLength(1));
    expect(calls('clean_selected')[0]?.[1]).toEqual(
      expect.objectContaining({ id: 'temp', paths: ['/tmp/big.log'] }),
    );
    expect(
      await screen.findByText('Freed 2.0 MB by deleting 1 files.'),
    ).toBeTruthy();
  });

  it('select all and select none in the preview', async () => {
    await renderApp();
    fireEvent.click(screen.getByRole('button', { name: 'Scan' }));
    fireEvent.click(
      (await screen.findAllByRole('button', { name: 'Preview files' }))[0]!,
    );
    const panel = await screen.findByRole('region', {
      name: 'Preview: Temporary files',
    });

    fireEvent.click(within(panel).getByRole('button', { name: 'Select all' }));
    expect(within(panel).getByText('2 of 2 selected · 3.0 MB')).toBeTruthy();
    fireEvent.click(within(panel).getByRole('button', { name: 'Select none' }));
    expect(within(panel).getByText('0 of 2 selected · 0 B')).toBeTruthy();
  });

  it('shows backend errors', async () => {
    handler.mockImplementationOnce(() => current); // get_settings
    await renderApp();
    handler.mockImplementationOnce(() => {
      throw new Error('permission denied');
    });
    fireEvent.click(screen.getByRole('button', { name: 'Scan' }));
    expect((await screen.findByRole('alert')).textContent).toContain(
      'permission denied',
    );
  });
});

describe('History tab', () => {
  it('lists cleans newest first with the running total', async () => {
    await renderApp();
    fireEvent.click(screen.getByRole('tab', { name: 'History' }));

    expect(
      await screen.findByText('Freed so far: 2.0 MB in 2 cleans.'),
    ).toBeTruthy();
    const items = within(
      screen.getByRole('list', { name: 'Clean history' }),
    ).getAllByRole('listitem');
    expect(items[0]?.textContent).toContain('Manual');
    expect(items[0]?.textContent).toContain('1 failed');
    expect(items[1]?.textContent).toContain('Scheduled');
  });

  it('clears history after confirmation', async () => {
    await renderApp();
    fireEvent.click(screen.getByRole('tab', { name: 'History' }));
    fireEvent.click(
      await screen.findByRole('button', { name: 'Clear history' }),
    );
    fireEvent.click(screen.getByRole('button', { name: 'Delete' }));
    expect(await screen.findByText('No cleans yet.')).toBeTruthy();
    expect(calls('clear_history')).toHaveLength(1);
  });
});

describe('Settings tab', () => {
  it('turns targets on with an explicit id list and cannot enable missing ones', async () => {
    await renderApp();
    fireEvent.click(screen.getByRole('tab', { name: 'Settings' }));

    const npm = (await screen.findByRole('checkbox', {
      name: /npm cache/,
    })) as HTMLInputElement;
    expect(npm.checked).toBe(false);
    fireEvent.click(npm);
    await waitFor(() => expect(calls('save_settings')).toHaveLength(1));
    expect(current.enabledTargets).toEqual(['npm', 'temp', 'user-cache']);

    const firefox = screen.getByRole('checkbox', {
      name: /Firefox cache/,
    }) as HTMLInputElement;
    expect(firefox.disabled).toBe(true);
    expect(screen.getByText('Not found on this computer')).toBeTruthy();
    // There is no way to type a path anywhere in Settings.
    expect(screen.queryByRole('textbox')).toBeNull();
  });

  it('saves the weekly schedule', async () => {
    await renderApp();
    fireEvent.click(screen.getByRole('tab', { name: 'Settings' }));
    fireEvent.click(
      await screen.findByRole('checkbox', {
        name: 'Clean turned-on targets automatically every week',
      }),
    );
    await waitFor(() => expect(current.schedule.enabled).toBe(true));
    fireEvent.change(screen.getByRole('combobox', { name: 'weekday' }), {
      target: { value: '2' },
    });
    fireEvent.change(screen.getByRole('combobox', { name: 'hour' }), {
      target: { value: '18' },
    });
    await waitFor(() =>
      expect(current.schedule).toEqual({ enabled: true, weekday: 2, hour: 18 }),
    );
  });

  it('switches theme and language', async () => {
    await renderApp();
    fireEvent.click(screen.getByRole('tab', { name: 'Settings' }));

    fireEvent.change(await screen.findByRole('combobox', { name: 'Theme' }), {
      target: { value: 'dark' },
    });
    await waitFor(() =>
      expect(document.documentElement.dataset.theme).toBe('dark'),
    );

    fireEvent.change(screen.getByRole('combobox', { name: 'Language' }), {
      target: { value: 'az' },
    });
    expect(
      await screen.findByRole('tab', { name: 'Parametrlər' }),
    ).toBeTruthy();
    expect(document.documentElement.lang).toBe('az');
    expect(current.language).toBe('az');
  });
});
