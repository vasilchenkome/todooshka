import { describe, expect, it } from "vitest";

import type { Task } from "../../api/types";
import { filterTasks, type BoardFilters } from "./boardModel";

const now = new Date(2026, 10, 14, 12, 0);

function task(id: number, due: string | null, overrides: Partial<Task> = {}): Task {
  return {
    id,
    project_id: 1,
    title: `Задача ${id}`,
    description: "",
    status: "backlog",
    position: id,
    created_at: 0,
    updated_at: 0,
    completed_at: null,
    due_on: due,
    priority: "normal",
    estimate_minutes: null,
    ...overrides,
  };
}

function filters(due: BoardFilters["due"]): BoardFilters {
  return { query: "", priority: "", due, tag: "" };
}

const tasks = [
  task(1, "2026-11-13"),
  task(2, "2026-11-14"),
  task(3, "2026-11-15"),
  task(4, "2026-11-21"),
  task(5, "2026-11-22"),
  task(6, null),
];

describe("board due filters", () => {
  it("keeps overdue, today and the next seven days separate", () => {
    expect(filterTasks(tasks, filters("overdue"), new Map(), now).map((item) => item.id)).toEqual([
      1,
    ]);
    expect(filterTasks(tasks, filters("today"), new Map(), now).map((item) => item.id)).toEqual([
      2,
    ]);
    expect(filterTasks(tasks, filters("week"), new Map(), now).map((item) => item.id)).toEqual([
      3, 4,
    ]);
    expect(filterTasks(tasks, filters("none"), new Map(), now).map((item) => item.id)).toEqual([
      6,
    ]);
  });

  it("does not let a later date into the week filter", () => {
    expect(filterTasks(tasks, filters("week"), new Map(), now).every((item) => item.id !== 5)).toBe(
      true,
    );
  });
});
