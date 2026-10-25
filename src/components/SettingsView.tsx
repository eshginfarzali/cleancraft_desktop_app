import { targetLabel, useI18n } from '../i18n';
import type {
  Category,
  Language,
  Settings,
  TargetInfo,
  Theme,
} from '../lib/api';

interface Props {
  settings: Settings;
  targets: TargetInfo[];
  onChange: (next: Settings) => void;
}

const CATEGORIES: Category[] = ['system', 'browser', 'developer'];
const THEMES: Theme[] = ['system', 'light', 'dark'];
const LANGUAGES: { value: Language; label: string }[] = [
  { value: 'en', label: 'English' },
  { value: 'az', label: 'Azərbaycan' },
];

export default function SettingsView({ settings, targets, onChange }: Props) {
  const { t } = useI18n();
  const enabled = new Set(targets.filter((x) => x.enabled).map((x) => x.id));

  const toggleTarget = (id: string) => {
    const next = new Set(enabled);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    onChange({ ...settings, enabledTargets: [...next].sort() });
  };

  return (
    <div className="stack">
      <section className="panel stack" aria-labelledby="targets-heading">
        <h2 id="targets-heading">{t('settings.targets')}</h2>
        <p className="muted">{t('settings.targets.help')}</p>
        {CATEGORIES.map((category) => {
          const inCategory = targets.filter((x) => x.category === category);
          if (inCategory.length === 0) return null;
          return (
            <fieldset key={category}>
              <legend>{t(`settings.category.${category}`)}</legend>
              {inCategory.map((target) => (
                <label key={target.id} className="toggle">
                  <input
                    type="checkbox"
                    checked={enabled.has(target.id)}
                    disabled={!target.available && !enabled.has(target.id)}
                    onChange={() => toggleTarget(target.id)}
                  />
                  <span className="target">
                    <strong>{targetLabel(t, target.id)}</strong>
                    {target.available ? (
                      target.roots.map((root) => <code key={root}>{root}</code>)
                    ) : (
                      <span className="muted">{t('settings.unavailable')}</span>
                    )}
                  </span>
                </label>
              ))}
            </fieldset>
          );
        })}
      </section>

      <section className="panel stack" aria-labelledby="schedule-heading">
        <h2 id="schedule-heading">{t('settings.schedule')}</h2>
        <label className="toggle">
          <input
            type="checkbox"
            checked={settings.schedule.enabled}
            onChange={(e) =>
              onChange({
                ...settings,
                schedule: { ...settings.schedule, enabled: e.target.checked },
              })
            }
          />
          <span>{t('settings.schedule.enabled')}</span>
        </label>
        <div
          className="row"
          role="group"
          aria-label={t('settings.schedule.when')}
        >
          <select
            aria-label="weekday"
            value={settings.schedule.weekday}
            disabled={!settings.schedule.enabled}
            onChange={(e) =>
              onChange({
                ...settings,
                schedule: {
                  ...settings.schedule,
                  weekday: Number(e.target.value),
                },
              })
            }
          >
            {[0, 1, 2, 3, 4, 5, 6].map((d) => (
              <option key={d} value={d}>
                {t(`weekday.${d}` as `weekday.${0 | 1 | 2 | 3 | 4 | 5 | 6}`)}
              </option>
            ))}
          </select>
          <select
            aria-label="hour"
            value={settings.schedule.hour}
            disabled={!settings.schedule.enabled}
            onChange={(e) =>
              onChange({
                ...settings,
                schedule: {
                  ...settings.schedule,
                  hour: Number(e.target.value),
                },
              })
            }
          >
            {Array.from({ length: 24 }, (_, h) => (
              <option key={h} value={h}>
                {`${String(h).padStart(2, '0')}:00`}
              </option>
            ))}
          </select>
        </div>
        <p className="muted">{t('settings.schedule.help')}</p>
      </section>

      <section className="panel stack" aria-labelledby="appearance-heading">
        <h2 id="appearance-heading">{t('settings.appearance')}</h2>
        <label className="row">
          <span>{t('settings.theme')}</span>
          <select
            value={settings.theme}
            onChange={(e) =>
              onChange({ ...settings, theme: e.target.value as Theme })
            }
          >
            {THEMES.map((theme) => (
              <option key={theme} value={theme}>
                {t(`settings.theme.${theme}`)}
              </option>
            ))}
          </select>
        </label>
        <label className="row">
          <span>{t('settings.language')}</span>
          <select
            value={settings.language}
            onChange={(e) =>
              onChange({ ...settings, language: e.target.value as Language })
            }
          >
            {LANGUAGES.map((l) => (
              <option key={l.value} value={l.value}>
                {l.label}
              </option>
            ))}
          </select>
        </label>
      </section>
    </div>
  );
}
