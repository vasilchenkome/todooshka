import { useEffect, useState } from "react";

import type {
  Attachment,
  Comment,
  Subtask,
  Task,
  TaskPlanning,
  TaskPriority,
  TaskStatus,
} from "../../api/types";
import {
  BOARD_COLUMNS,
  PRIORITY_LABELS,
  estimateLabel,
  formatBytes,
  formatDateTime,
} from "./boardModel";

const PRIORITIES: TaskPriority[] = ["low", "normal", "high", "urgent"];

type TaskDialogProps = {
  task: Task;
  busy: boolean;
  onClose: () => void;
  onSave: (patch: { title: string; description: string; planning: TaskPlanning }) => Promise<void>;
  onMove: (status: TaskStatus) => Promise<void>;
  onDelete: () => Promise<void>;
  subtasks: Subtask[];
  onCreateSubtask: (title: string) => Promise<void>;
  onToggleSubtask: (id: number, done: boolean) => Promise<void>;
  onDeleteSubtask: (id: number) => Promise<void>;
  tags: string[];
  tagSuggestions: string[];
  onReplaceTags: (tags: string[]) => Promise<void>;
  comments: Comment[];
  onCreateComment: (body: string) => Promise<void>;
  onEditComment: (id: number, body: string) => Promise<void>;
  onDeleteComment: (id: number) => Promise<void>;
  attachments: Attachment[];
  onUploadAttachment: (file: File) => Promise<void>;
  onDeleteAttachment: (id: number) => Promise<void>;
};

/** Диалог задачи: правка текста, планирование, смена колонки и удаление. */
export function TaskDialog({
  task,
  busy,
  onClose,
  onSave,
  onMove,
  onDelete,
  subtasks,
  onCreateSubtask,
  onToggleSubtask,
  onDeleteSubtask,
  tags,
  tagSuggestions,
  onReplaceTags,
  comments,
  onCreateComment,
  onEditComment,
  onDeleteComment,
  attachments,
  onUploadAttachment,
  onDeleteAttachment,
}: TaskDialogProps) {
  const [title, setTitle] = useState(task.title);
  const [description, setDescription] = useState(task.description);
  const [status, setStatus] = useState<TaskStatus>(task.status);
  const [dueOn, setDueOn] = useState(task.due_on ?? "");
  const [priority, setPriority] = useState<TaskPriority>(task.priority);
  const [estimate, setEstimate] = useState(
    task.estimate_minutes === null ? "" : String(task.estimate_minutes),
  );
  const [estimateError, setEstimateError] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [subtaskDraft, setSubtaskDraft] = useState("");
  const [subtaskBusy, setSubtaskBusy] = useState(false);
  const [tagDraft, setTagDraft] = useState("");
  const [commentDraft, setCommentDraft] = useState("");
  const [editingComment, setEditingComment] = useState<number | null>(null);
  const [editingBody, setEditingBody] = useState("");
  const [uploadBusy, setUploadBusy] = useState(false);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onClose();
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onClose]);

  function estimateMinutes(): number | null {
    if (estimate.trim() === "") return null;
    const minutes = Number(estimate);
    if (!Number.isInteger(minutes) || minutes < 1 || minutes > 100_000) {
      return Number.NaN;
    }
    return minutes;
  }

  async function save() {
    const minutes = estimateMinutes();
    if (Number.isNaN(minutes)) {
      setEstimateError("оценка — целое число минут от 1 до 100000");
      return;
    }
    setEstimateError(null);

    try {
      await onSave({
        title,
        description,
        planning: {
          due_on: dueOn || null,
          priority,
          estimate_minutes: minutes,
        },
      });
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

  const minutes = estimateMinutes();
  const doneSubtasks = subtasks.filter((item) => item.done).length;

  async function addTag() {
    const name = tagDraft.trim().replace(/^#/, "").trim();
    if (!name) return;
    if (tags.some((tag) => tag.toLowerCase() === name.toLowerCase())) {
      setTagDraft("");
      return;
    }
    try {
      await onReplaceTags([...tags, name]);
      setTagDraft("");
    } catch {
      // Ошибку показывает доска.
    }
  }

  async function removeTag(name: string) {
    try {
      await onReplaceTags(tags.filter((tag) => tag !== name));
    } catch {
      // Ошибку показывает доска.
    }
  }

  async function uploadFile(file: File) {
    setUploadBusy(true);
    try {
      await onUploadAttachment(file);
    } catch {
      // Ошибку показывает доска.
    } finally {
      setUploadBusy(false);
    }
  }

  async function addComment() {
    const body = commentDraft.trim();
    if (!body) return;
    try {
      await onCreateComment(body);
      setCommentDraft("");
    } catch {
      // Ошибку показывает доска.
    }
  }

  async function saveComment() {
    if (editingComment === null) return;
    const body = editingBody.trim();
    if (!body) return;
    try {
      await onEditComment(editingComment, body);
      setEditingComment(null);
      setEditingBody("");
    } catch {
      // Ошибку показывает доска.
    }
  }

  async function addSubtask() {
    const title = subtaskDraft.trim();
    if (!title || subtaskBusy) return;
    setSubtaskBusy(true);
    try {
      await onCreateSubtask(title);
      setSubtaskDraft("");
    } catch {
      // Ошибку показывает доска, поле остаётся с текстом.
    } finally {
      setSubtaskBusy(false);
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
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                if (title.trim() && !busy) void save();
              }
            }}
            className="w-full border-b border-rule bg-transparent pb-2 font-display text-[20px] font-semibold tracking-[-0.01em] outline-none focus:border-signal"
          />
        </label>

        <label className="mt-6 block">
          <span className="mb-1 block text-[12px] text-steel">Описание</span>
          <textarea
            value={description}
            rows={4}
            onChange={(event) => setDescription(event.target.value)}
            onKeyDown={(event) => {
              if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
                event.preventDefault();
                if (title.trim() && !busy) void save();
              }
            }}
            placeholder="детали, ссылки, что учесть"
            className="w-full resize-y border border-rule bg-transparent p-3 text-[14px] outline-none focus:border-signal"
          />
        </label>

        <div className="mt-6 grid gap-x-5 gap-y-4 sm:grid-cols-2">
          <label className="block">
            <span className="mb-1 block text-[12px] text-steel">Срок</span>
            <div className="flex items-end gap-2 border-b border-rule pb-2">
              <input
                type="date"
                value={dueOn}
                onChange={(event) => setDueOn(event.target.value)}
                className="w-full bg-transparent text-[14px] tabular-nums outline-none"
              />
              {dueOn && (
                <button
                  type="button"
                  onClick={() => setDueOn("")}
                  className="shrink-0 text-[11px] text-steel hover:text-ink"
                >
                  очистить
                </button>
              )}
            </div>
          </label>

          <label className="block">
            <span className="mb-1 block text-[12px] text-steel">Оценка, минут</span>
            <div className="flex items-end gap-2 border-b border-rule pb-2">
              <input
                type="number"
                min={1}
                max={100000}
                step={1}
                inputMode="numeric"
                placeholder="90"
                value={estimate}
                onChange={(event) => {
                  setEstimate(event.target.value);
                  setEstimateError(null);
                }}
                className="w-full bg-transparent text-[14px] tabular-nums outline-none"
              />
              <span className="shrink-0 text-[11px] whitespace-nowrap text-steel">
                {Number.isNaN(minutes) ? "" : minutes === null ? "не задана" : estimateLabel(minutes)}
              </span>
            </div>
          </label>
        </div>

        {estimateError && (
          <p role="alert" className="mt-2 border-l-2 border-signal pl-2 text-[12px]">
            {estimateError}
          </p>
        )}

        <div className="mt-5">
          <span className="mb-2 block text-[12px] text-steel">Приоритет</span>
          <div className="flex flex-wrap gap-x-5 gap-y-2 border-b border-rule">
            {PRIORITIES.map((value) => {
              const active = priority === value;
              const tone = active
                ? value === "urgent"
                  ? "border-signal text-signal"
                  : "border-ink text-ink"
                : "border-transparent text-steel hover:text-ink";
              return (
                <button
                  key={value}
                  type="button"
                  onClick={() => setPriority(value)}
                  className={`-mb-px border-b-2 pb-2 text-[13px] transition-colors ${tone}`}
                >
                  {PRIORITY_LABELS[value]}
                </button>
              );
            })}
          </div>
        </div>

        <div className="mt-5">
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

        <div className="mt-5">
          <span className="mb-2 block text-[12px] text-steel">Теги</span>
          <div className="flex flex-wrap items-center gap-2">
            {tags.map((tag) => (
              <span
                key={tag}
                className="inline-flex items-center gap-1.5 border border-rule px-2 py-0.5 text-[12px]"
              >
                {tag}
                <button
                  type="button"
                  aria-label={`Убрать тег ${tag}`}
                  onClick={() => void removeTag(tag)}
                  className="text-steel hover:text-ink"
                >
                  ×
                </button>
              </span>
            ))}
            <input
              list="tag-suggestions"
              value={tagDraft}
              onChange={(event) => setTagDraft(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  event.preventDefault();
                  void addTag();
                }
              }}
              placeholder="добавить тег"
              aria-label="Новый тег"
              className="min-w-[130px] border-b border-rule bg-transparent py-1 text-[13px] outline-none focus:border-signal"
            />
            <datalist id="tag-suggestions">
              {tagSuggestions
                .filter(
                  (tag) =>
                    !tags.some((selected) => selected.toLowerCase() === tag.toLowerCase()),
                )
                .map((tag) => (
                  <option key={tag} value={tag} />
                ))}
            </datalist>
          </div>
        </div>

        <div className="mt-6">
          <div className="mb-2 flex items-baseline justify-between text-[12px] text-steel">
            <span>Чек-лист</span>
            <span className="tabular-nums">
              {doneSubtasks} / {subtasks.length}
            </span>
          </div>

          {subtasks.length > 0 && (
            <ul>
              {subtasks.map((item) => (
                <li
                  key={item.id}
                  className="grid grid-cols-[22px_1fr_auto] items-center gap-2 border-b border-rule py-1.5 last:border-b-0"
                >
                  <button
                    type="button"
                    role="checkbox"
                    aria-checked={item.done}
                    aria-label={item.title}
                    onClick={() => void onToggleSubtask(item.id, !item.done)}
                    className={`flex h-[18px] w-[18px] items-center justify-center border ${
                      item.done ? "border-ink bg-ink text-paper" : "border-ink"
                    }`}
                  >
                    {item.done && <span aria-hidden="true">✓</span>}
                  </button>
                  <span
                    className={`text-[14px] ${item.done ? "text-steel line-through" : ""}`}
                  >
                    {item.title}
                  </span>
                  <button
                    type="button"
                    onClick={() => void onDeleteSubtask(item.id)}
                    className="text-[12px] text-steel hover:text-ink"
                  >
                    удалить
                  </button>
                </li>
              ))}
            </ul>
          )}

          <form
            onSubmit={(event) => {
              event.preventDefault();
              void addSubtask();
            }}
            className="mt-2 flex items-center gap-3"
          >
            <input
              value={subtaskDraft}
              onChange={(event) => setSubtaskDraft(event.target.value)}
              placeholder="добавить пункт"
              aria-label="Новый пункт чек-листа"
              className="w-full border-b border-rule bg-transparent py-1.5 text-[13px] outline-none focus:border-signal"
            />
            <button
              type="submit"
              disabled={!subtaskDraft.trim() || subtaskBusy}
              className="shrink-0 text-[13px] text-ink hover:text-signal disabled:text-steel"
            >
              добавить
            </button>
          </form>
        </div>

        <div className="mt-6">
          <div className="mb-2 flex items-baseline justify-between text-[12px] text-steel">
            <span>Вложения</span>
            <span className="tabular-nums">{attachments.length} / 10</span>
          </div>

          {attachments.length > 0 && (
            <ul>
              {attachments.map((file) => (
                <li
                  key={file.id}
                  className="grid grid-cols-[1fr_auto_auto] items-center gap-3 border-b border-rule py-1.5 text-[13px] last:border-b-0"
                >
                  <a
                    href={`/api/attachments/${file.id}`}
                    download={file.filename}
                    className="truncate hover:text-signal"
                  >
                    {file.filename}
                  </a>
                  <span className="text-[11px] whitespace-nowrap text-steel tabular-nums">
                    {formatBytes(file.size)}
                  </span>
                  <button
                    type="button"
                    onClick={() => void onDeleteAttachment(file.id)}
                    className="text-[12px] text-steel hover:text-ink"
                  >
                    удалить
                  </button>
                </li>
              ))}
            </ul>
          )}

          <label className="mt-2 inline-flex cursor-pointer items-center gap-2 text-[13px] text-steel hover:text-signal">
            <span aria-hidden="true" className="font-display text-[16px] leading-none text-ink">
              +
            </span>
            {uploadBusy ? "загружаю…" : "прикрепить файл"}
            <input
              type="file"
              className="hidden"
              disabled={uploadBusy || attachments.length >= 10}
              onChange={(event) => {
                const file = event.target.files?.[0];
                event.target.value = "";
                if (file) void uploadFile(file);
              }}
            />
          </label>
          <p className="mt-1 text-[11px] text-steel">любые файлы до 10 МБ, не больше 10 на задачу</p>
        </div>

        <div className="mt-6">
          <div className="mb-2 flex items-baseline justify-between text-[12px] text-steel">
            <span>Комментарии</span>
            <span className="tabular-nums">{comments.length}</span>
          </div>

          {comments.length > 0 && (
            <div className="space-y-3">
              {comments.map((comment) =>
                editingComment === comment.id ? (
                  <div key={comment.id} className="border-l-2 border-signal pl-3">
                    <textarea
                      value={editingBody}
                      rows={3}
                      onChange={(event) => setEditingBody(event.target.value)}
                      aria-label="Редактировать комментарий"
                      className="w-full resize-y border border-rule bg-transparent p-2 text-[13px] outline-none focus:border-signal"
                    />
                    <div className="mt-1.5 flex gap-4 text-[12px]">
                      <button type="button" onClick={() => void saveComment()} className="text-ink">
                        сохранить
                      </button>
                      <button
                        type="button"
                        onClick={() => {
                          setEditingComment(null);
                          setEditingBody("");
                        }}
                        className="text-steel"
                      >
                        отмена
                      </button>
                    </div>
                  </div>
                ) : (
                  <div key={comment.id} className="border-l-2 border-rule pl-3">
                    <div className="flex flex-wrap items-baseline gap-3 text-[11px] text-steel">
                      <span className="tabular-nums">{formatDateTime(comment.created_at)}</span>
                      <button
                        type="button"
                        onClick={() => {
                          setEditingComment(comment.id);
                          setEditingBody(comment.body);
                        }}
                        className="hover:text-ink"
                      >
                        редактировать
                      </button>
                      <button
                        type="button"
                        onClick={() => void onDeleteComment(comment.id)}
                        className="hover:text-ink"
                      >
                        удалить
                      </button>
                    </div>
                    <p className="mt-1 text-[14px] whitespace-pre-wrap">{comment.body}</p>
                  </div>
                ),
              )}
            </div>
          )}

          <div className="mt-3 border border-rule p-2">
            <textarea
              value={commentDraft}
              rows={2}
              onChange={(event) => setCommentDraft(event.target.value)}
              placeholder="написать комментарий"
              aria-label="Новый комментарий"
              className="w-full resize-y bg-transparent text-[13px] outline-none"
            />
            <div className="mt-1 flex justify-end">
              <button
                type="button"
                disabled={!commentDraft.trim()}
                onClick={() => void addComment()}
                className="text-[13px] text-ink hover:text-signal disabled:text-steel"
              >
                отправить
              </button>
            </div>
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
