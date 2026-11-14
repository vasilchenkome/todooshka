import { useState } from "react";
import type { FormEvent } from "react";
import { useDroppable } from "@dnd-kit/core";
import { SortableContext, verticalListSortingStrategy } from "@dnd-kit/sortable";

import type { Task, TaskStatus } from "../../api/types";
import { TaskCard } from "./TaskCard";

type BoardColumnProps = {
  status: TaskStatus;
  title: string;
  tasks: Task[];
  numbers: Map<number, number>;
  accent?: boolean;
  activeOnMobile: boolean;
  creating: boolean;
  onOpenTask: (task: Task) => void;
  onCreate: (status: TaskStatus, title: string) => Promise<void>;
};

const DONE_PREVIEW = 5;

export function BoardColumn({
  status,
  title,
  tasks,
  numbers,
  accent = false,
  activeOnMobile,
  creating,
  onOpenTask,
  onCreate,
}: BoardColumnProps) {
  const { setNodeRef, isOver } = useDroppable({ id: `column:${status}` });
  const [adding, setAdding] = useState(false);
  const [draft, setDraft] = useState("");
  const [showAll, setShowAll] = useState(false);

  const visible =
    status === "done" && !showAll && tasks.length > DONE_PREVIEW
      ? tasks.slice(0, DONE_PREVIEW)
      : tasks;
  const hiddenCount = tasks.length - visible.length;

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmed = draft.trim();
    if (!trimmed) return;

    try {
      await onCreate(status, trimmed);
      setDraft("");
      setAdding(false);
    } catch {
      // Ошибку показывает страница доски, форма остаётся открытой для повтора.
    }
  }

  return (
    <section
      ref={setNodeRef}
      aria-label={title}
      className={`${activeOnMobile ? "flex" : "hidden"} min-h-[360px] flex-col px-3 pt-6 pb-8 transition-colors md:flex md:min-h-[calc(100vh-60px)] ${
        isOver ? "bg-hover" : ""
      }`}
    >
      <div
        className={`mx-1 flex items-end justify-between border-b pb-2.5 md:mx-3 ${
          accent ? "border-b-2 border-signal" : "border-ink"
        }`}
      >
        <h2 className="font-display text-[18px] font-semibold tracking-[-0.01em]">{title}</h2>
        <span
          className={`font-display text-[40px] leading-[0.9] font-semibold tracking-[-0.03em] tabular-nums ${
            accent ? "text-signal" : "text-ink"
          }`}
        >
          {tasks.length}
        </span>
      </div>

      <SortableContext items={visible.map((task) => task.id)} strategy={verticalListSortingStrategy}>
        <ul className="mt-0 flex-1">
          {visible.map((task) => (
            <TaskCard
              key={task.id}
              task={task}
              number={numbers.get(task.id) ?? task.id}
              onOpen={onOpenTask}
            />
          ))}
        </ul>
      </SortableContext>

      {hiddenCount > 0 && (
        <button
          type="button"
          onClick={() => setShowAll(true)}
          className="mx-4 mt-3.5 self-start border-b border-rule pb-0.5 text-[12px] text-steel hover:border-ink hover:text-ink"
        >
          показать ещё {hiddenCount}
        </button>
      )}

      {adding ? (
        <form onSubmit={submit} className="mx-4 mt-3.5 flex flex-wrap items-center gap-3">
          <input
            autoFocus
            value={draft}
            aria-label="Название задачи"
            placeholder="что нужно сделать"
            onChange={(event) => setDraft(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                setDraft("");
                setAdding(false);
              }
            }}
            className="w-full border-b border-rule bg-transparent py-2 text-[15px] outline-none focus:border-signal"
          />
          <button
            type="submit"
            disabled={creating || !draft.trim()}
            className="text-[13px] text-ink hover:text-signal disabled:text-steel"
          >
            добавить
          </button>
          <button
            type="button"
            onClick={() => {
              setDraft("");
              setAdding(false);
            }}
            className="text-[13px] text-steel hover:text-ink"
          >
            отмена
          </button>
        </form>
      ) : (
        <button
          type="button"
          onClick={() => setAdding(true)}
          className="mx-4 mt-3.5 flex items-center gap-2 self-start text-[13px] text-steel transition-colors hover:text-signal"
        >
          <span aria-hidden="true" className="font-display text-[18px] leading-none text-ink">
            +
          </span>
          задача
        </button>
      )}
    </section>
  );
}
