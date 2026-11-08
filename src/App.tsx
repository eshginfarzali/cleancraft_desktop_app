import { useCallback, useEffect, useState } from 'react';
import './App.css';
import CleanView from './components/CleanView';
import HistoryView from './components/HistoryView';
import SettingsView from './components/SettingsView';
import { I18nProvider, translate } from './i18n';
import {
  getSettings,
  getTargets,
  saveSettings,
  type Settings,
  type TargetInfo,
} from './lib/api';
import { applyTheme } from './lib/theme';

type Tab = 'clean' | 'history' | 'settings';
const TABS: Tab[] = ['clean', 'history', 'settings'];

function App() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [targets, setTargets] = useState<TargetInfo[]>([]);
  const [tab, setTab] = useState<Tab>('clean');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([getSettings(), getTargets()])
      .then(([s, list]) => {
        setSettings(s);
        setTargets(list);
      })
      .catch((e: unknown) => setError(String(e)));
  }, []);

  const theme = settings?.theme ?? 'system';
  useEffect(() => applyTheme(theme), [theme]);

  const language = settings?.language ?? 'en';
  useEffect(() => {
    document.documentElement.lang = language;
  }, [language]);

  const updateSettings = useCallback(async (next: Settings) => {
    setSettings(next); // optimistic, so toggles feel instant
    try {
      const saved = await saveSettings(next);
      setSettings(saved);
      setTargets(await getTargets());
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const t = (key: Parameters<typeof translate>[1]) => translate(language, key);

  return (
    <I18nProvider language={language}>
      <main className="container">
        <header>
          <h1>CleanCraft</h1>
          <p className="muted">{t('app.tagline')}</p>
          <nav role="tablist" className="tabs">
            {TABS.map((name) => (
              <button
                key={name}
                role="tab"
                aria-selected={tab === name}
                className={tab === name ? 'active' : undefined}
                onClick={() => setTab(name)}
              >
                {t(`tab.${name}`)}
              </button>
            ))}
          </nav>
        </header>

        {error && (
          <p role="alert" className="error">
            {translate(language, 'error.prefix', { message: error })}
          </p>
        )}

        {settings && (
          <div role="tabpanel">
            {tab === 'clean' && <CleanView targets={targets} />}
            {tab === 'history' && <HistoryView />}
            {tab === 'settings' && (
              <SettingsView
                settings={settings}
                targets={targets}
                onChange={updateSettings}
              />
            )}
          </div>
        )}
      </main>
    </I18nProvider>
  );
}

export default App;
