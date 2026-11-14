import { translate } from "./translate";
import type { Locale } from "./types";

function intlLocale(locale: Locale): string {
  return locale === "ru" ? "ru-RU" : "en-US";
}

export function formatNumber(value: number, locale: Locale, maximumFractionDigits = 1): string {
  return new Intl.NumberFormat(intlLocale(locale), { maximumFractionDigits }).format(value);
}

export function formatDateTime(unixSeconds: number, locale: Locale): string {
  return new Intl.DateTimeFormat(intlLocale(locale), {
    day: "numeric",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
  })
    .format(new Date(unixSeconds * 1000))
    .replace(/\./g, "");
}

export function formatDay(unixSeconds: number, locale: Locale): string {
  return new Intl.DateTimeFormat(intlLocale(locale), { day: "numeric", month: "short" })
    .format(new Date(unixSeconds * 1000))
    .replace(/\./g, "");
}

export function formatShortDate(date: Date, locale: Locale, includeYear: boolean): string {
  return new Intl.DateTimeFormat(
    intlLocale(locale),
    includeYear
      ? { day: "numeric", month: "short", year: "numeric" }
      : { day: "numeric", month: "short" },
  )
    .format(date)
    .replace(/\./g, "");
}

export function formatBytes(size: number, locale: Locale): string {
  if (size < 1024) return `${size} ${translate(locale, "unit.byte")}`;
  const kilobytes = size / 1024;
  if (kilobytes < 1024) {
    return `${formatNumber(kilobytes, locale)} ${translate(locale, "unit.kilobyte")}`;
  }
  return `${formatNumber(kilobytes / 1024, locale)} ${translate(locale, "unit.megabyte")}`;
}
