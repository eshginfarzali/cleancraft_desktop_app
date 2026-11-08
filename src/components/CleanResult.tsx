import { useI18n } from '../i18n';
import type { CleanReport } from '../lib/api';
import { formatBytes } from '../lib/format';

export default function CleanResult({ reports }: { reports: CleanReport[] }) {
  const { t } = useI18n();
  const sum = (key: 'freedBytes' | 'deletedFiles' | 'failed' | 'rejected') =>
    reports.reduce((total, r) => total + r[key], 0);
  return (
    <section aria-label="Clean results" className="panel result">
      <p>
        {t('result.freed', {
          size: formatBytes(sum('freedBytes')),
          count: sum('deletedFiles'),
        })}
      </p>
      {sum('failed') > 0 && (
        <p className="muted">{t('result.failed', { count: sum('failed') })}</p>
      )}
      {sum('rejected') > 0 && (
        <p className="muted">
          {t('result.rejected', { count: sum('rejected') })}
        </p>
      )}
    </section>
  );
}
