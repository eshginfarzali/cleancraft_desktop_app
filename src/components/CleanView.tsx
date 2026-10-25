import { useCallback, useState } from 'react';
import { targetLabel, useI18n } from '../i18n';
import {
  type CleanReport,
  cleanTargets,
  type ScanResult,
  scanTargets,
  type TargetInfo,
} from '../lib/api';
import { formatBytes } from '../lib/format';
import CleanResult from './CleanResult';
import Confirm from './Confirm';
import PreviewPanel from './PreviewPanel';

type Phase = 'idle' | 'scanning' | 'confirming' | 'cleaning';

export default function CleanView({ targets }: { targets: TargetInfo[] }) {
  const { t } = useI18n();
  const [phase, setPhase] = useState<Phase>('idle');
  const [scans, setScans] = useState<ScanResult[] | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [reports, setReports] = useState<CleanReport[] | null>(null);
  const [previewId, setPreviewId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const onError = useCallback((message: string) => setError(message), []);

  const chosen = (scans ?? []).filter((s) => selected.has(s.id));
  const chosenBytes = chosen.reduce((sum, s) => sum + s.totalBytes, 0);
  const totalBytes = (scans ?? []).reduce((sum, s) => sum + s.totalBytes, 0);
  const totalFiles = (scans ?? []).reduce((sum, s) => sum + s.fileCount, 0);
  const busy = phase === 'scanning' || phase === 'cleaning';
  const anyEnabled = targets.some(
    (target) => target.enabled && target.available,
  );

  async function scan() {
    setPhase('scanning');
    setError(null);
    setReports(null);
    setPreviewId(null);
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

  const previewRoots = scans?.find((s) => s.id === previewId)?.roots ?? [];

  return (
    <div className="stack">
      <button className="primary" onClick={scan} disabled={busy || !anyEnabled}>
        {phase === 'scanning'
          ? t('scan.scanning')
          : scans
            ? t('scan.again')
            : t('scan.button')}
      </button>
      {!anyEnabled && <p className="muted">{t('scan.empty')}</p>}

      {error && (
        <p role="alert" className="error">
          {t('error.prefix', { message: error })}
        </p>
      )}

      {scans && (
        <section aria-label="Scan results" className="stack">
          <p className="summary">
            {t('scan.summary', {
              size: formatBytes(totalBytes),
              count: totalFiles,
            })}
          </p>
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
                    <strong>{targetLabel(t, s.id)}</strong>
                    {s.roots.map((root) => (
                      <code key={root}>{root}</code>
                    ))}
                    <span className="muted">
                      {t('scan.target.detail', {
                        count: s.fileCount,
                        size: formatBytes(s.totalBytes),
                      })}
                      {s.skipped > 0 &&
                        ` · ${t('scan.target.unreadable', { count: s.skipped })}`}
                    </span>
                  </span>
                </label>
                <button
                  className="link"
                  disabled={busy || s.fileCount === 0}
                  onClick={() => setPreviewId(s.id)}
                >
                  {t('scan.preview')}
                </button>
              </li>
            ))}
          </ul>

          {phase === 'confirming' ? (
            <Confirm
              message={t('confirm.wholeTargets', {
                size: formatBytes(chosenBytes),
              })}
              onConfirm={clean}
              onCancel={() => setPhase('idle')}
            />
          ) : (
            <button
              className="danger"
              disabled={busy || chosen.length === 0 || chosenBytes === 0}
              onClick={() => setPhase('confirming')}
            >
              {phase === 'cleaning'
                ? t('clean.cleaning')
                : t('clean.selected', { size: formatBytes(chosenBytes) })}
            </button>
          )}
        </section>
      )}

      {previewId && (
        <PreviewPanel
          key={previewId}
          targetId={previewId}
          roots={previewRoots}
          onClose={() => setPreviewId(null)}
          onCleaned={(report) => setReports([report])}
          onError={onError}
        />
      )}

      {reports && <CleanResult reports={reports} />}
    </div>
  );
}
