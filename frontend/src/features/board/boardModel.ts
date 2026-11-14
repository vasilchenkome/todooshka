import type { Task, TaskStatus } from "../../api/types";

export const BOARD_COLUMNS: { status: TaskStatus; title: string }[] = [
  { status: "backlog", title: "Бэклог" },
  { status: "in_progress", title: "В работе" },
  { status: "done", title: "Готово" },
];

export type BoardColumns = Record<TaskStatus, Task[]>;

export function areOrdered(left: Task, right: Task): number {
  return left.position - right.position || left.id - right.id;
}

/** Задачи по колонкам в порядке доски; исходный массив не меняется. */
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

/** Номер задачи внутри проекта: сквозной по id, как читается на доске. */
export function projectNumbers(tasks: Task[]): Map<number, number> {
  const ordered = [...tasks].sort((left, right) => left.id - right.id);
  return new Map(ordered.map((task, index) => [task.id, index + 1]));
}

/**
 * Оптимистичный перенос: карточка встаёт на место до ответа сервера.
 * Позиции в целевой колонке перенумеровываются, чтобы порядок совпадал с серверным.
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

/** Куда встанет карточка при броске на `overId`; `null` — мимо. */
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
    // Внутри колонки индекс считается по полному списку: так же работает arrayMove.
    const column = tasks.filter((task) => task.status === overTask.status).sort(areOrdered);
    return { status: overTask.status, index: column.findIndex((task) => task.id === overTask.id) };
  }

  const target = tasks.filter((task) => task.status === overTask.status).sort(areOrdered);
  return { status: overTask.status, index: target.findIndex((task) => task.id === overTask.id) };
}

export function formatDay(unixSeconds: number): string {
  return new Intl.DateTimeFormat("ru-RU", { day: "numeric", month: "short" })
    .format(new Date(unixSeconds * 1000))
    .replace(".", "");
}
