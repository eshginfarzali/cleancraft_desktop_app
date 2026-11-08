import { useEffect, useMemo, useState } from 'react';
import { locale, targetLabel, useI18n } from '../i18n';
import {
  type CleanReport,
  cleanSelected,
  type Preview,
  previewTarget,
} from '../lib/api';
import { formatBytes, formatDateTime, relativeToRoots } from '../lib/format';
import Confirm from './Confirm';

interface Props {
  targetId: string;
  roots: string[];
  onClose: () => void;
  onCleaned: (report: CleanReport) => void;
  onError: (message: string) => void;
}

/** Dry run for one target: list what would be deleted and delete a selection. */
export default function PreviewPanel({
  targetId,
  roots,
  onClose,
  onCleaned,
  onError,
}: Props) {
  const { t, language } = useI18n();
  const [preview, setPreview] = useState<Preview | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [confirming, setConfirming] = useState(false);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let active = true;
    previewTarget(targetId)
      .then((p) => {
        if (active) setPreview(p);
      })
      .catch((e: unknown) => onError(String(e)));
    return () => {
      active = false;
    };
  }, [targetId, onError]);

  const chosen = useMemo(
    () => (preview?.entries ?? []).filter((e) => selected.has(e.path)),
    [preview, selected],
  );
  const chosenBytes = chosen.reduce((sum, e) => sum + e.size, 0);

  const toggle = (path: string) =>
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });

  async function deleteSelected() {
    setConfirming(false);
    setBusy(true);
    try {
      const report = await cleanSelected(
        targetId,
        chosen.map((e) => e.path),
      );
      onCleaned(report);
      // Refresh the list so deleted items disappear.
      setSelected(new Set());
      setPreview(await previewTarget(targetId));
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  }

  const title = t('preview.title', { target: targetLabel(t, targetId) });

  return (
    <section className="panel preview" aria-label={title}>
      <header className="row spread">
        <h2>{title}</h2>
        <button onClick={onClose}>{t('preview.close')}</button>
      </header>

      {!preview ? (
        <p className="muted">{t('scan.scanning')}</p>
      ) : preview.entries.length === 0 ? (
        <p className="muted">{t('preview.empty')}</p>
      ) : (
        <>
          <div className="row spread">
            <div className="row">
              <button
                onClick={() =>
                  setSelected(new Set(preview.entries.map((e) => e.path)))
                }
              >
                {t('preview.selectAll')}
              </button>
              <button onClick={() => setSelected(new Set())}>
                {t('preview.selectNone')}
              </button>
            </div>
            <span className="muted">
              {t('preview.selected', {
                count: chosen.length,
                total: preview.entries.length,
                size: formatBytes(chosenBytes),
              })}
            </span>
          </div>
          {preview.truncated && (
            <p className="muted">
              {t('preview.truncated', {
                shown: preview.entries.length,
                total: preview.totalFiles,
              })}
            </p>
          )}
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th />
                  <th>{t('preview.column.path')}</th>
                  <th className="num">{t('preview.column.size')}</th>
                  <th>{t('preview.column.modified')}</th>
                </tr>
              </thead>
              <tbody>
                {preview.entries.map((entry) => {
                  const shown = relativeToRoots(entry.path, roots);
                  return (
                    <tr key={entry.path}>
                      <td>
                        <input
                          type="checkbox"
                          aria-label={shown}
                          checked={selected.has(entry.path)}
                          disabled={busy}
                          onChange={() => toggle(entry.path)}
                        />
                      </td>
                      <td>
                        <code title={entry.path}>{shown}</code>
                        {entry.files > 1 && (
                          <span className="muted">
                            {' '}
                            ({t('preview.folderOf', { count: entry.files })})
                          </span>
                        )}
                      </td>
                      <td className="num">{formatBytes(entry.size)}</td>
                      <td>
                        {formatDateTime(entry.modifiedMs, locale(language))}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
          {confirming ? (
            <Confirm
              message={t('preview.confirm', {
                count: chosen.length,
                size: formatBytes(chosenBytes),
              })}
              onConfirm={deleteSelected}
              onCancel={() => setConfirming(false)}
            />
          ) : (
            <button
              className="danger"
              disabled={busy || chosen.length === 0}
              onClick={() => setConfirming(true)}
            >
              {t('preview.deleteSelected', { size: formatBytes(chosenBytes) })}
            </button>
          )}
        </>
      )}
    </section>
  );
}
