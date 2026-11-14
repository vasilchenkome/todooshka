import type { Task, TaskPriority, TaskStatus } from "../../api/types";
import { formatShortDate } from "../../i18n/format";
import type { MessageKey } from "../../i18n/messages.ru";
import { translate } from "../../i18n/translate";
import type { Locale } from "../../i18n/types";

export const BOARD_COLUMNS: TaskStatus[] = ["backlog", "in_progress", "done"];

export function statusLabel(status: TaskStatus, locale: Locale): string {
  return translate(locale, `status.${status}` as MessageKey);
}

export type BoardColumns = Record<TaskStatus, Task[]>;

export function areOrdered(left: Task, right: Task): number {
  return left.position - right.position || left.id - right.id;
}

/** Tasks grouped by column in board order; the source array is left untouched. */
export function groupByStatus(tasks: Task[]): BoardColumns {
  const columns: BoardColumns = { backlog: [], in_progress: [], done: [] };
  for (const task of tasks) {
    columns[task.status].push(task);
  }
  for (const column of Object.values(columns)) {
    column.sort(areOrdered);
  }
  return columns;
}

/** Task number within the project: sequential by id, as read on the board. */
export function projectNumbers(tasks: Task[]): Map<number, number> {
  const ordered = [...tasks].sort((left, right) => left.id - right.id);
  return new Map(ordered.map((task, index) => [task.id, index + 1]));
}

/**
 * Optimistic move: the card lands in place before the server responds.
 * Positions in the target column are renumbered so the order matches the server.
 */
export function applyOptimisticMove(
  tasks: Task[],
  taskId: number,
  status: TaskStatus,
  targetIndex: number,
  nowSeconds: number,
): Task[] {
  const moved = tasks.find((task) => task.id === taskId);
  if (!moved) {
    return tasks;
  }

  const others = tasks.filter((task) => task.id !== taskId);
  const target = others.filter((task) => task.status === status).sort(areOrdered);
  const index = Math.max(0, Math.min(targetIndex, target.length));

  const optimistic: Task = {
    ...moved,
    status,
    completed_at:
      status === "done" ? (moved.completed_at ?? nowSeconds) : null,
  };
  target.splice(index, 0, optimistic);

  const positions = new Map(target.map((task, position) => [task.id, position + 1]));

  return others
    .map((task) =>
      task.status === status ? { ...task, position: positions.get(task.id) ?? task.position } : task,
    )
    .concat([{ ...optimistic, position: positions.get(optimistic.id) ?? index + 1 }]);
}

/** Where the card lands when dropped on `overId`; `null` means a miss. */
export function resolveDrop(
  tasks: Task[],
  activeId: number,
  overId: string,
): { status: TaskStatus; index: number } | null {
  if (overId.startsWith("column:")) {
    const status = overId.slice("column:".length) as TaskStatus;
    const column = tasks.filter((task) => task.status === status && task.id !== activeId);
    return { status, index: column.length };
  }

  const overTaskId = Number(overId);
  const overTask = tasks.find((task) => task.id === overTaskId);
  if (!overTask) {
    return null;
  }

  if (overTask.status === tasks.find((task) => task.id === activeId)?.status) {
    // Within a column the index is counted over the full list, the same way arrayMove works.
    const column = tasks.filter((task) => task.status === overTask.status).sort(areOrdered);
    return { status: overTask.status, index: column.findIndex((task) => task.id === overTask.id) };
  }

  const target = tasks.filter((task) => task.status === overTask.status).sort(areOrdered);
  return { status: overTask.status, index: target.findIndex((task) => task.id === overTask.id) };
}

export function formatDay(unixSeconds: number, locale: Locale): string {
  return formatShortDate(new Date(unixSeconds * 1000), locale, false);
}

export type DueTone = "over" | "today" | "plain" | "none";

export type DueLabel = {
  text: string;
  tone: DueTone;
};

/** Due date in the local calendar; the string is treated as a calendar date without time. */
function parseDueDate(value: string): Date | null {
  const parts = value.split("-").map(Number);
  if (parts.length !== 3 || parts.some((part) => !Number.isInteger(part))) return null;
  return new Date(parts[0], parts[1] - 1, parts[2]);
}

function startOfToday(): Date {
  const now = new Date();
  return new Date(now.getFullYear(), now.getMonth(), now.getDate());
}

export function dueLabel(dueOn: string | null, locale: Locale): DueLabel {
  const none = translate(locale, "due.none");
  if (!dueOn) return { text: none, tone: "none" };

  const date = parseDueDate(dueOn);
  if (!date) return { text: none, tone: "none" };

  const today = startOfToday();
  const days = Math.round((date.getTime() - today.getTime()) / 86_400_000);
  const short = formatShortDate(date, locale, date.getFullYear() !== today.getFullYear());

  if (days < 0) return { text: translate(locale, "due.overdue", { date: short }), tone: "over" };
  if (days === 0) return { text: translate(locale, "due.today"), tone: "today" };
  if (days === 1) return { text: translate(locale, "due.tomorrow"), tone: "plain" };
  return { text: short, tone: "plain" };
}

export function estimateLabel(minutes: number | null, locale: Locale): string {
  if (!minutes) return translate(locale, "estimate.none");
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (hours === 0) return translate(locale, "estimate.minutes", { minutes: rest });
  if (rest === 0) return translate(locale, "estimate.hours", { hours });
  return translate(locale, "estimate.hoursMinutes", { hours, minutes: rest });
}

export function priorityLabel(priority: TaskPriority, locale: Locale): string {
  return translate(locale, `priority.${priority}` as MessageKey);
}

export type BoardFilters = {
  query: string;
  priority: TaskPriority | "";
  due: "" | "overdue" | "week" | "none";
  tag: string;
};

export function daysUntilDue(dueOn: string | null): number | null {
  const date = dueOn ? parseDueDate(dueOn) : null;
  if (!date) return null;
  return Math.round((date.getTime() - startOfToday().getTime()) / 86_400_000);
}

/** Client-side board filters: text search, priority, due date and tag. */
export function filterTasks(
  tasks: Task[],
  filters: BoardFilters,
  tagsByTask: Map<number, string[]>,
): Task[] {
  const query = filters.query.trim().toLowerCase();

  return tasks.filter((task) => {
    if (
      query &&
      !task.title.toLowerCase().includes(query) &&
      !task.description.toLowerCase().includes(query)
    ) {
      return false;
    }
    if (filters.priority && task.priority !== filters.priority) return false;

    if (filters.due) {
      const days = daysUntilDue(task.due_on);
      if (filters.due === "none" && task.due_on) return false;
      if (filters.due === "overdue" && (days === null || days >= 0)) return false;
      if (filters.due === "week" && (days === null || days < 0 || days > 7)) return false;
    }

    if (filters.tag) {
      const tags = tagsByTask.get(task.id) ?? [];
      if (!tags.some((tag) => tag.toLowerCase() === filters.tag.toLowerCase())) return false;
    }

    return true;
  });
}


