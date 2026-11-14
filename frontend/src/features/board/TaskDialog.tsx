import { useEffect, useState } from "react";

import type { Task, TaskStatus } from "../../api/types";
import { BOARD_COLUMNS } from "./boardModel";

type TaskDialogProps = {
  task: Task;
  busy: boolean;
  onClose: () => void;
  onSave: (patch: { title: string; description: string }) => Promise<void>;
  onMove: (status: TaskStatus) => Promise<void>;
  onDelete: () => Promise<void>;
};

/** Диалог задачи: правка текста, смена колонки и удаление с подтверждением. */
export function TaskDialog({ task, busy, onClose, onSave, onMove, onDelete }: TaskDialogProps) {
  const [title, setTitle] = useState(task.title);
  const [description, setDescription] = useState(task.description);
  const [status, setStatus] = useState<TaskStatus>(task.status);
  const [confirmDelete, setConfirmDelete] = useState(false);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onClose();
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onClose]);

  async function save() {
    try {
      await onSave({ title, description });
      onClose();
    } catch {
      // Ошибка отображается на доске, диалог остаётся открытым.
    }
  }

  async function changeStatus(next: TaskStatus) {
    if (next === status) return;
    const previous = status;
    setStatus(next);
    try {
      await onMove(next);
    } catch {
      setStatus(previous);
    }
  }

  async function remove() {
    if (!confirmDelete) {
      setConfirmDelete(true);
      return;
    }
    try {
      await onDelete();
      onClose();
    } catch {
      // Ошибка отображается на доске.
    }
  }

  return (
    <div
      className="fixed inset-0 z-20 flex items-end justify-center bg-ink/35 p-0 md:items-center md:p-6"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-label="Задача"
        className="max-h-[92dvh] w-full max-w-[560px] overflow-y-auto overscroll-contain border border-ink bg-paper p-5 md:max-h-[85dvh] md:p-7"
      >
        <div className="mb-6 flex items-center justify-between gap-4">
          <span className="text-[12px] text-steel tabular-nums">
            {String(task.id).padStart(3, "0")}
          </span>
          <button type="button" onClick={onClose} className="text-[13px] text-steel hover:text-ink">
            закрыть
          </button>
        </div>

        <label className="block">
          <span className="mb-1 block text-[12px] text-steel">Заголовок</span>
          <input
            value={title}
            onChange={(event) => setTitle(event.target.value)}
            className="w-full border-b border-rule bg-transparent pb-2 font-display text-[20px] font-semibold tracking-[-0.01em] outline-none focus:border-signal"
          />
        </label>

        <label className="mt-6 block">
          <span className="mb-1 block text-[12px] text-steel">Описание</span>
          <textarea
            value={description}
            rows={4}
            onChange={(event) => setDescription(event.target.value)}
            placeholder="детали, ссылки, что учесть"
            className="w-full resize-y border border-rule bg-transparent p-3 text-[14px] outline-none focus:border-signal"
          />
        </label>

        <div className="mt-6">
          <span className="mb-2 block text-[12px] text-steel">Колонка</span>
          <div className="flex gap-5 border-b border-rule">
            {BOARD_COLUMNS.map(({ status: value, title: label }) => (
              <button
                key={value}
                type="button"
                onClick={() => void changeStatus(value)}
                className={`-mb-px border-b-2 pb-2 text-[13px] transition-colors ${
                  status === value
                    ? "border-signal text-ink"
                    : "border-transparent text-steel hover:text-ink"
                }`}
              >
                {label}
              </button>
            ))}
          </div>
        </div>

        <div className="mt-8 flex items-center justify-between gap-4">
          <button
            type="button"
            onClick={() => void remove()}
            disabled={busy}
            className={`text-[13px] ${
              confirmDelete ? "text-signal hover:underline" : "text-steel hover:text-ink"
            }`}
          >
            {confirmDelete ? "точно удалить?" : "удалить"}
          </button>

          <div className="flex items-center gap-4">
            <button type="button" onClick={onClose} className="text-[13px] text-steel hover:text-ink">
              отмена
            </button>
            <button
              type="button"
              onClick={() => void save()}
              disabled={busy || !title.trim()}
              className="bg-ink px-4 py-2.5 text-[14px] text-paper transition-colors hover:bg-signal disabled:cursor-not-allowed disabled:opacity-60"
            >
              {busy ? "сохраняю…" : "сохранить"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
