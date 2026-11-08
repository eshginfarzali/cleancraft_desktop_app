import { createContext, type ReactNode, useContext, useMemo } from 'react';
import type { Language } from '../lib/api';
import { az } from './az';
import { en, type MessageKey } from './en';

const dictionaries: Record<Language, Record<MessageKey, string>> = { en, az };

export type Vars = Record<string, string | number>;
export type Translate = (key: MessageKey, vars?: Vars) => string;

export function translate(
  language: Language,
  key: MessageKey,
  vars: Vars = {},
): string {
  const template = dictionaries[language][key] ?? en[key];
  return template.replace(/\{(\w+)\}/g, (match, name: string) => {
    const value = vars[name];
    if (value === undefined) return match;
    return typeof value === 'number'
      ? value.toLocaleString(locale(language))
      : value;
  });
}

export const locale = (language: Language) =>
  language === 'az' ? 'az-AZ' : 'en-US';

const I18nContext = createContext<{ language: Language; t: Translate }>({
  language: 'en',
  t: (key, vars) => translate('en', key, vars),
});

export function I18nProvider({
  language,
  children,
}: {
  language: Language;
  children: ReactNode;
}) {
  const value = useMemo(
    () => ({
      language,
      t: ((key, vars) => translate(language, key, vars)) as Translate,
    }),
    [language],
  );
  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export const useI18n = () => useContext(I18nContext);

export type { MessageKey };

/** Label for a target id; unknown ids fall back to the id itself. */
export function targetLabel(t: Translate, id: string): string {
  const key = `target.${id}`;
  return key in en ? t(key as MessageKey) : id;
}
