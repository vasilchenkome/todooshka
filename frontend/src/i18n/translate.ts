import { en } from "./messages.en";
import { ru } from "./messages.ru";
import type { MessageKey, PluralKey } from "./messages.ru";
import type { Locale } from "./types";

export type MessageParams = Record<string, string | number>;

const catalogs = { ru, en };

export function interpolate(template: string, params: MessageParams = {}): string {
  return template.replace(/\{(\w+)\}/g, (_match, key: string) =>
    params[key] === undefined ? `{${key}}` : String(params[key]),
  );
}

export function translate(locale: Locale, key: MessageKey, params?: MessageParams): string {
  return interpolate(catalogs[locale][key], params);
}

export function pluralize(locale: Locale, base: PluralKey, count: number): string {
  const form = new Intl.PluralRules(locale === "ru" ? "ru-RU" : "en-US").select(count);
  const key = `${base}.${form}` as MessageKey;
  const fallback = `${base}.other` as MessageKey;
  return translate(locale, key in catalogs[locale] ? key : fallback, { count });
}
