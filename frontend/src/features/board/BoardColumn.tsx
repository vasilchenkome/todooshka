import { useEffect, useRef, useState } from "react";
import type { FormEvent } from "react";
import { useDroppable } from "@dnd-kit/core";
import { SortableContext, verticalListSortingStrategy } from "@dnd-kit/sortable";

import type { Task, TaskStatus } from "../../api/types";
import { useI18n } from "../../i18n";
import { TaskCard } from "./TaskCard";

type BoardColumnProps = {
  status: TaskStatus;
  title: string;
  tasks: Task[];
  numbers: Map<number, number>;
  subtasks: Map<number, { done: number; total: number }>;
  tags: Map<number, string[]>;
  comments: Map<number, number>;
  attachments: Map<number, number>;
  filtered?: boolean;
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
  subtasks,
  tags,
  comments,
  attachments,
  filtered = false,
  accent = false,
  activeOnMobile,
  creating,
  onOpenTask,
  onCreate,
}: BoardColumnProps) {
  const { t } = useI18n();
  const { setNodeRef, isOver } = useDroppable({ id: `column:${status}` });
  const [adding, setAdding] = useState(false);
  const [draft, setDraft] = useState("");
  const [showAll, setShowAll] = useState(false);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!adding) return;
    if (window.matchMedia("(pointer: fine)").matches) {
      inputRef.current?.focus();
    }
  }, [adding]);

  const visible =
    status === "done" && !showAll && tasks.length > DONE_PREVIEW
      ? tasks.slice(0, DONE_PREVIEW)
      : tasks;
  const hiddenCount = tasks.length - visible.length;

  function closeForm() {
    setDraft("");
    setAdding(false);
    triggerRef.current?.focus();
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmed = draft.trim();
    if (!trimmed) return;

    try {
      await onCreate(status, trimmed);
      setDraft("");
      setAdding(false);
      triggerRef.current?.focus();
    } catch {
      // The board page shows the error; the form stays open so it can be retried.
    }
  }

  return (
    <section
      ref={setNodeRef}
      aria-label={title}
      className={`${activeOnMobile ? "flex" : "hidden"} min-h-[360px] flex-col px-3 pt-6 pb-8 transition-colors md:flex md:min-h-[calc(100dvh-112px)] ${
        isOver ? "bg-hover" : ""
      }`}
    >
      <div
        className={`mx-1 flex items-end justify-between gap-3 border-b pb-3 md:mx-3 ${
          accent ? "border-b-2 border-signal" : "border-ink"
        }`}
      >
        <div className="relative flex min-w-0 max-w-[calc(100%-56px)] items-center pr-12">
          <h2 className="min-w-0 truncate font-display text-[18px] font-semibold tracking-[-0.01em]">
            {title}
          </h2>
          <button
            ref={triggerRef}
            type="button"
            aria-label={t("board.addColumnAria", { column: title })}
            aria-expanded={adding}
            aria-controls={`create-task-${status}`}
            onClick={() => {
              if (adding) {
                inputRef.current?.focus();
              } else {
                setAdding(true);
              }
            }}
            className="group absolute top-1/2 right-0 grid h-11 w-11 -translate-y-1/2 place-items-center"
          >
            <span
              aria-hidden="true"
              className={`grid h-7 w-7 place-items-center border font-display text-[18px] font-semibold leading-none transition-colors ${
                adding
                  ? "border-signal bg-sheet text-signal"
                  : "border-rule text-ink group-hover:border-signal group-hover:bg-sheet group-hover:text-signal"
              }`}
            >
              +
            </span>
          </button>
        </div>
        <span
          className={`shrink-0 font-display text-[40px] leading-[0.9] font-semibold tracking-[-0.03em] tabular-nums ${
            accent ? "text-signal" : "text-ink"
          }`}
        >
          {tasks.length}
        </span>
      </div>

      {adding && (
        <form
          id={`create-task-${status}`}
          onSubmit={submit}
          className="mx-1 mt-3 flex flex-wrap items-center gap-3 md:mx-3"
        >
          <input
            ref={inputRef}
            value={draft}
            aria-label={t("board.addAria")}
            placeholder={t("board.addPlaceholder")}
            onChange={(event) => setDraft(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                closeForm();
              }
            }}
            className="w-full border-b border-rule bg-transparent py-2 text-[15px] outline-none focus:border-signal"
          />
          <button
            type="submit"
            disabled={creating || !draft.trim()}
            className="text-[13px] text-ink hover:text-signal disabled:text-steel"
          >
            {t("board.addButton")}
          </button>
          <button
            type="button"
            onClick={closeForm}
            className="text-[13px] text-steel hover:text-ink"
          >
            {t("common.cancel")}
          </button>
        </form>
      )}

      <SortableContext items={visible.map((task) => task.id)} strategy={verticalListSortingStrategy}>
        {tasks.length === 0 && !filtered && !adding && (
          <p className="mx-4 mt-4 text-[13px] text-steel">
            {t("board.emptyColumn")}
          </p>
        )}
        <ul className="mt-0 flex-1">
          {visible.map((task) => (
            <TaskCard
              key={task.id}
              task={task}
              number={numbers.get(task.id) ?? task.id}
              subtasks={subtasks.get(task.id)}
              tags={tags.get(task.id)}
              comments={comments.get(task.id) ?? 0}
              attachments={attachments.get(task.id) ?? 0}
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
          {t("board.showMore", { count: hiddenCount })}
        </button>
      )}
    </section>
  );
}
