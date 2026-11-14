import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";

import type { Task } from "../../api/types";
import { useI18n } from "../../i18n";
import { dueLabel, priorityLabel } from "./boardModel";

type TaskCardProps = {
  task: Task;
  number: number;
  onOpen: (task: Task) => void;
  subtasks?: { done: number; total: number };
  tags?: string[];
  comments?: number;
  attachments?: number;
};

const PRIORITY_DOT: Record<Task["priority"], string> = {
  low: "bg-steel",
  normal: "",
  high: "bg-ink",
  urgent: "bg-signal",
};

const PRIORITY_TEXT: Record<Task["priority"], string> = {
  low: "text-steel",
  normal: "text-steel",
  high: "text-ink",
  urgent: "text-signal",
};

/** Task card: a row, not a "cloud"; the number and date use tabular figures. */
export function TaskCard({
  task,
  number,
  onOpen,
  subtasks,
  tags,
  comments,
  attachments,
}: TaskCardProps) {
  const { t, plural, locale } = useI18n();
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: task.id,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    touchAction: "manipulation" as const,
  };

  const due = dueLabel(task.due_on, locale);
  const dueClass =
    due.tone === "over"
      ? "text-signal"
      : due.tone === "today"
        ? "font-medium text-ink"
        : due.tone === "plain"
          ? "text-ink"
          : "";

  return (
    <li
      ref={setNodeRef}
      style={style}
      {...attributes}
      {...listeners}
      className={`grid cursor-grab grid-cols-[44px_1fr] gap-3 border-b border-rule px-4 py-4 transition-colors hover:bg-hover active:cursor-grabbing ${
        isDragging ? "opacity-40" : ""
      }`}
    >
      <span className="pt-0.5 text-[11px] tracking-[0.02em] tabular-nums text-steel">
        {String(number).padStart(3, "0")}
      </span>
      <div>
        <button
          type="button"
          onClick={() => onOpen(task)}
          className="block text-left text-[15px] leading-[1.35] font-medium hover:text-signal"
        >
          {task.title}
        </button>
        <p className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-[12px]">
          <span className={`tabular-nums ${dueClass}`}>{due.text}</span>
          {subtasks && subtasks.total > 0 && (
            <span className="whitespace-nowrap tabular-nums">
              {t("card.checklist", { done: subtasks.done, total: subtasks.total })}
            </span>
          )}
          {attachments !== undefined && attachments > 0 && (
            <span className="whitespace-nowrap">
              {plural("plural.files", attachments)}
            </span>
          )}
          {comments !== undefined && comments > 0 && (
            <span className="whitespace-nowrap">
              {plural("plural.comments", comments)}
            </span>
          )}
          {task.priority !== "normal" && (
            <span
              className={`ml-auto flex items-center gap-1.5 whitespace-nowrap ${PRIORITY_TEXT[task.priority]}`}
            >
              <span className={`h-[7px] w-[7px] ${PRIORITY_DOT[task.priority]}`} />
              {priorityLabel(task.priority, locale)}
            </span>
          )}
        </p>
        {tags && tags.length > 0 && (
          <div className="mt-1.5 flex flex-wrap gap-x-2 text-[11px] text-steel">
            {tags.slice(0, 2).map((tag) => (
              <span key={tag}>#{tag}</span>
            ))}
            {tags.length > 2 && <span>+{tags.length - 2}</span>}
          </div>
        )}
      </div>
    </li>
  );
}
