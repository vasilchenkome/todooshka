import { useQuery } from "@tanstack/react-query";
import { Link, useParams } from "react-router-dom";

import { api } from "../../api/client";
import { AppBar } from "../../shared/ui/AppBar";

/** Заглушка доски: следующий шаг — колонки, задачи и перетаскивание. */
export default function BoardPage() {
  const { projectId } = useParams();
  const id = Number(projectId);

  const projects = useQuery({ queryKey: ["projects"], queryFn: api.listProjects });
  const project = projects.data?.find((item) => item.id === id);

  return (
    <div className="flex min-h-full flex-col">
      <AppBar
        left={
          <Link
            to="/"
            className="border-b border-transparent pb-px text-[13px] text-steel transition-colors hover:border-rule hover:text-ink"
          >
            ← проекты
          </Link>
        }
      />

      <main className="flex-1 px-5 py-10 md:px-12">
        <div className="max-w-[820px]">
          <h1 className="font-display text-[28px] font-semibold tracking-[-0.02em]">
            {project?.name ?? "Доска"}
          </h1>
          <p className="mt-2 text-[14px] text-steel">
            Задачи и перетаскивание между колонками — следующий шаг. Проект уже открывается по
            своему адресу, счётчики приходят с сервера.
          </p>

          {project && (
            <p className="mt-6 text-[13px] text-steel tabular-nums">
              бэклог {project.counts?.backlog ?? 0}, в работе {project.counts?.in_progress ?? 0},
              готово {project.counts?.done ?? 0}
            </p>
          )}
        </div>
      </main>
    </div>
  );
}
