import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";

import type { Task } from "../../api/types";
import { PRIORITY_LABELS, dueLabel } from "./boardModel";

type TaskCardProps = {
  task: Task;
  number: number;
  onOpen: (task: Task) => void;
  subtasks?: { done: number; total: number };
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

/** Карточка задачи: строка, а не «облако»; номер и дата — табличными цифрами. */
export function TaskCard({ task, number, onOpen, subtasks }: TaskCardProps) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: task.id,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    touchAction: "manipulation" as const,
  };

  const due = dueLabel(task.due_on);
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
              чек-лист {subtasks.done}/{subtasks.total}
            </span>
          )}
          {task.priority !== "normal" && (
            <span
              className={`ml-auto flex items-center gap-1.5 whitespace-nowrap ${PRIORITY_TEXT[task.priority]}`}
            >
              <span className={`h-[7px] w-[7px] ${PRIORITY_DOT[task.priority]}`} />
              {PRIORITY_LABELS[task.priority]}
            </span>
          )}
        </p>
      </div>
    </li>
  );
}
