/** Calendar helpers for due dates. All dates are local calendar dates, never instants. */
export const WEEK_DAYS = 7;

export type DueBucket = "overdue" | "today" | "week" | "later" | "none";

const DAY_MS = 86_400_000;

export function startOfLocalDay(now = new Date()): Date {
  return new Date(now.getFullYear(), now.getMonth(), now.getDate());
}

/** A local calendar date in `YYYY-MM-DD`, the same shape the API stores. */
export function localDateKey(date = new Date()): string {
  const year = String(date.getFullYear()).padStart(4, "0");
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function todayLocalDate(now = new Date()): string {
  return localDateKey(startOfLocalDay(now));
}

/** Parses a date key into a local midnight; invalid keys return `null`. */
export function parseDateKey(value: string): Date | null {
  const parts = value.split("-").map(Number);
  if (parts.length !== 3 || parts.some((part) => !Number.isInteger(part))) return null;
  const [year, month, day] = parts;
  const date = new Date(year, month - 1, day);
  if (
    date.getFullYear() !== year ||
    date.getMonth() !== month - 1 ||
    date.getDate() !== day
  ) {
    return null;
  }
  return date;
}

/** Difference in calendar days, immune to DST transitions. */
export function daysUntilDue(dueOn: string | null, now = new Date()): number | null {
  if (!dueOn) return null;
  const date = parseDateKey(dueOn);
  if (!date) return null;
  const today = startOfLocalDay(now);
  const localDays = (value: Date) =>
    Date.UTC(value.getFullYear(), value.getMonth(), value.getDate()) / DAY_MS;
  return Math.round(localDays(date) - localDays(today));
}

export function dueBucket(dueOn: string | null, now = new Date()): DueBucket {
  const days = daysUntilDue(dueOn, now);
  if (days === null) return "none";
  if (days < 0) return "overdue";
  if (days === 0) return "today";
  if (days <= WEEK_DAYS) return "week";
  return "later";
}
