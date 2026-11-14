import { describe, expect, it } from "vitest";

import {
  daysUntilDue,
  dueBucket,
  localDateKey,
  parseDateKey,
  todayLocalDate,
} from "./dueCalendar";

const now = new Date(2026, 10, 14, 15, 30);

describe("due calendar", () => {
  it("uses the browser's local calendar date", () => {
    expect(todayLocalDate(now)).toBe("2026-11-14");
    expect(localDateKey(new Date(2026, 0, 5))).toBe("2026-01-05");
  });

  it("counts whole calendar days around today and the week boundary", () => {
    expect(daysUntilDue("2026-11-13", now)).toBe(-1);
    expect(daysUntilDue("2026-11-14", now)).toBe(0);
    expect(daysUntilDue("2026-11-15", now)).toBe(1);
    expect(daysUntilDue("2026-11-21", now)).toBe(7);
    expect(daysUntilDue("2026-11-22", now)).toBe(8);
    expect(daysUntilDue(null, now)).toBeNull();
    expect(daysUntilDue("not-a-date", now)).toBeNull();
  });

  it("buckets overdue, today, the next seven days and later", () => {
    expect(dueBucket("2026-11-13", now)).toBe("overdue");
    expect(dueBucket("2026-11-14", now)).toBe("today");
    expect(dueBucket("2026-11-15", now)).toBe("week");
    expect(dueBucket("2026-11-21", now)).toBe("week");
    expect(dueBucket("2026-11-22", now)).toBe("later");
    expect(dueBucket(null, now)).toBe("none");
  });

  it("rejects impossible dates instead of rolling them over", () => {
    expect(parseDateKey("2026-02-30")).toBeNull();
    expect(parseDateKey("2026-13-01")).toBeNull();
    expect(parseDateKey("2026-02-28")?.getDate()).toBe(28);
  });
});
