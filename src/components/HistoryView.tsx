import { useCallback, useEffect, useState } from 'react';
import { locale, targetLabel, useI18n } from '../i18n';
import {
  clearHistory,
  getHistory,
  type HistoryEntry,
  onHistoryUpdated,
} from '../lib/api';
import { formatBytes, formatDateTime } from '../lib/format';
import Confirm from './Confirm';

export default function HistoryView() {
  const { t, language } = useI18n();
  const [entries, setEntries] = useState<HistoryEntry[] | null>(null);
  const [confirming, setConfirming] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    getHistory()
      .then(setEntries)
      .catch((e: unknown) => setError(String(e)));
  }, []);

  useEffect(() => {
    load();
    // Scheduled and tray cleans happen in the background; refresh when they do.
    const unlisten = onHistoryUpdated(load);
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, [load]);

  async function clear() {
    setConfirming(false);
    try {
      await clearHistory();
      setEntries([]);
    } catch (e) {
      setError(String(e));
    }
  }

  if (error) {
    return (
      <p role="alert" className="error">
        {t('error.prefix', { message: error })}
      </p>
    );
  }
  if (!entries) return null;
  if (entries.length === 0)
    return <p className="muted">{t('history.empty')}</p>;

  const newestFirst = [...entries].sort((a, b) => b.atMs - a.atMs);
  const freed = entries.reduce((sum, e) => sum + e.freedBytes, 0);

  return (
    <div className="stack">
      <p className="summary">
        {t('history.total', {
          size: formatBytes(freed),
          count: entries.length,
        })}
      </p>
      <ol className="history" aria-label="Clean history">
        {newestFirst.map((entry) => (
          <li key={`${entry.atMs}-${entry.trigger}`}>
            <div className="row spread">
              <strong>{formatDateTime(entry.atMs, locale(language))}</strong>
              <span className={`badge ${entry.trigger}`}>
                {t(`history.trigger.${entry.trigger}`)}
              </span>
            </div>
            <span>
              {t('history.entry', {
                size: formatBytes(entry.freedBytes),
                count: entry.deletedFiles,
              })}
              {entry.failed > 0 &&
                ` · ${t('history.entryFailed', { count: entry.failed })}`}
            </span>
            <span className="muted">
              {entry.reports.map((r) => targetLabel(t, r.id)).join(', ')}
            </span>
          </li>
        ))}
      </ol>
      {confirming ? (
        <Confirm
          message={t('history.clearConfirm')}
          onConfirm={clear}
          onCancel={() => setConfirming(false)}
        />
      ) : (
        <button onClick={() => setConfirming(true)}>
          {t('history.clear')}
        </button>
      )}
    </div>
  );
}
