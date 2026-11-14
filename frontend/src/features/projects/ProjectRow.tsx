import { useState } from "react";

import type { Project } from "../../api/types";

type ProjectRowProps = {
  project: Project;
  archived: boolean;
  busy?: boolean;
  onOpen: () => void;
  onRename: (name: string) => void;
  onToggleArchive: () => void;
};

/** Строка проекта: имя, описание, счётчики задач и прогресс. */
export function ProjectRow({
  project,
  archived,
  busy = false,
  onOpen,
  onRename,
  onToggleArchive,
}: ProjectRowProps) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(project.name);

  const counts = project.counts ?? { backlog: 0, in_progress: 0, done: 0 };
  const total = counts.backlog + counts.in_progress + counts.done;
  const doneShare = total === 0 ? 0 : Math.round((counts.done / total) * 100);

  function commitRename() {
    setEditing(false);
    const trimmed = draft.trim();
    if (trimmed && trimmed !== project.name) {
      onRename(trimmed);
    } else {
      setDraft(project.name);
    }
  }

  return (
    <li className="group relative grid gap-x-6 gap-y-3 border-b border-rule px-4 py-4 transition-colors before:absolute before:inset-y-0 before:left-0 before:w-0.5 before:bg-signal before:opacity-0 before:content-[''] hover:bg-hover hover:before:opacity-100 md:grid-cols-[minmax(240px,1.6fr)_repeat(3,72px)_minmax(150px,0.8fr)] md:items-center md:py-[18px]">
      <div>
        {editing ? (
          <input
            autoFocus
            value={draft}
            aria-label="Название проекта"
            onChange={(event) => setDraft(event.target.value)}
            onBlur={commitRename}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                commitRename();
              }
              if (event.key === "Escape") {
                setDraft(project.name);
                setEditing(false);
              }
            }}
            className="w-full border-b border-signal bg-transparent pb-1 font-display text-[20px] font-semibold tracking-[-0.01em] outline-none"
          />
        ) : (
          <button
            type="button"
            onClick={onOpen}
            className="text-left font-display text-[20px] font-semibold tracking-[-0.01em] transition-colors hover:text-signal"
          >
            {project.name}
          </button>
        )}

        {project.description && (
          <p className="mt-1 text-[13px] text-steel">{project.description}</p>
        )}

        <div className="mt-2 flex gap-4 text-[12px] text-steel transition-opacity md:opacity-0 md:group-focus-within:opacity-100 md:group-hover:opacity-100">
          {archived ? (
            <button
              type="button"
              onClick={onToggleArchive}
              disabled={busy}
              className="hover:text-ink disabled:opacity-60"
            >
              вернуть из архива
            </button>
          ) : (
            <>
              <button
                type="button"
                onClick={() => setEditing(true)}
                className="hover:text-ink"
              >
                переименовать
              </button>
              <button
                type="button"
                onClick={onToggleArchive}
                disabled={busy}
                className="hover:text-ink disabled:opacity-60"
              >
                в архив
              </button>
            </>
          )}
        </div>
      </div>

      <div className="grid grid-cols-3 gap-4 md:contents">
        <Stat value={counts.backlog} label="бэклог" />
        <Stat value={counts.in_progress} label="в работе" />
        <Stat value={counts.done} label="готово" />
      </div>

      <div>
        <span className="block h-0.5 w-full bg-rule">
          <span className="block h-0.5 bg-ink" style={{ width: `${doneShare}%` }} />
        </span>
        <em className="mt-1.5 block text-[11px] text-steel not-italic tabular-nums">
          {counts.done} из {total} задач
        </em>
      </div>
    </li>
  );
}

function Stat({ value, label }: { value: number; label: string }) {
  return (
    <div className="md:text-right">
      <b className="block font-display text-[20px] font-semibold tracking-[-0.01em] tabular-nums">
        {value}
      </b>
      <span className="text-[11px] text-steel">{label}</span>
    </div>
  );
}
