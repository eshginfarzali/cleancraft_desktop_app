import { useState } from 'react';
import './App.css';
import {
  cleanTargets,
  type CleanReport,
  type ScanResult,
  scanTargets,
} from './lib/api';
import { formatBytes, pluralize } from './lib/format';

type Phase = 'idle' | 'scanning' | 'confirming' | 'cleaning';

function App() {
  const [phase, setPhase] = useState<Phase>('idle');
  const [scans, setScans] = useState<ScanResult[] | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [reports, setReports] = useState<CleanReport[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  const chosen = (scans ?? []).filter((s) => selected.has(s.id));
  const chosenBytes = chosen.reduce((sum, s) => sum + s.totalBytes, 0);
  const busy = phase === 'scanning' || phase === 'cleaning';

  async function scan() {
    setPhase('scanning');
    setError(null);
    setReports(null);
    try {
      const results = await scanTargets();
      setScans(results);
      setSelected(
        new Set(results.filter((r) => r.fileCount > 0).map((r) => r.id)),
      );
    } catch (e) {
      setError(String(e));
    } finally {
      setPhase('idle');
    }
  }

  async function clean() {
    setPhase('cleaning');
    setError(null);
    try {
      setReports(await cleanTargets(chosen.map((s) => s.id)));
      setScans(null);
      setSelected(new Set());
    } catch (e) {
      setError(String(e));
    } finally {
      setPhase('idle');
    }
  }

  const toggle = (id: string) =>
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });

  const freed = reports?.reduce((sum, r) => sum + r.freedBytes, 0) ?? 0;
  const failed = reports?.reduce((sum, r) => sum + r.failed, 0) ?? 0;

  return (
    <main className="container">
      <header>
        <h1>CleanCraft</h1>
        <p className="muted">
          Removes cache and temp files that have not been modified for 24 hours.
          Nothing outside the folders listed below is ever touched.
        </p>
      </header>

      <button className="primary" onClick={scan} disabled={busy}>
        {phase === 'scanning' ? 'Scanning…' : scans ? 'Scan again' : 'Scan'}
      </button>

      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}

      {scans && (
        <section aria-label="Scan results">
          <ul className="targets">
            {scans.map((s) => (
              <li key={s.id}>
                <label>
                  <input
                    type="checkbox"
                    checked={selected.has(s.id)}
                    disabled={busy || s.fileCount === 0}
                    onChange={() => toggle(s.id)}
                  />
                  <span className="target">
                    <strong>{s.label}</strong>
                    <code>{s.path}</code>
                    <span className="muted">
                      {pluralize(s.fileCount, 'file')} ·{' '}
                      {formatBytes(s.totalBytes)}
                      {s.skipped > 0 &&
                        ` · ${pluralize(s.skipped, 'folder')} not readable`}
                    </span>
                  </span>
                </label>
              </li>
            ))}
          </ul>

          {phase === 'confirming' ? (
            <div
              className="confirm"
              role="alertdialog"
              aria-label="Confirm clean"
            >
              <p>
                Permanently delete {formatBytes(chosenBytes)} from{' '}
                {chosen.length === 1 ? 'this folder' : 'these folders'}? This
                cannot be undone.
              </p>
              <div className="row">
                <button className="danger" onClick={clean}>
                  Delete
                </button>
                <button onClick={() => setPhase('idle')}>Cancel</button>
              </div>
            </div>
          ) : (
            <button
              className="danger"
              disabled={busy || chosen.length === 0 || chosenBytes === 0}
              onClick={() => setPhase('confirming')}
            >
              {phase === 'cleaning'
                ? 'Cleaning…'
                : `Clean selected (${formatBytes(chosenBytes)})`}
            </button>
          )}
        </section>
      )}

      {reports && (
        <section aria-label="Clean results" className="result">
          <p>
            Freed <strong>{formatBytes(freed)}</strong> by deleting{' '}
            {pluralize(
              reports.reduce((sum, r) => sum + r.deletedFiles, 0),
              'file',
            )}
            .
          </p>
          {failed > 0 && (
            <p className="muted">
              {pluralize(failed, 'file')} could not be deleted (in use or
              protected) and were left in place.
            </p>
          )}
        </section>
      )}
    </main>
  );
}

export default App;
