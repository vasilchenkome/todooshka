import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";

import { ApiError, api } from "../../api/client";
import type { ProjectPatch } from "../../api/types";
import { AppBar } from "../../shared/ui/AppBar";
import { ProjectRow } from "./ProjectRow";

export default function ProjectsPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState("");
  const [showArchived, setShowArchived] = useState(false);

  const projects = useQuery({ queryKey: ["projects"], queryFn: api.listProjects });

  const createProject = useMutation({
    mutationFn: (name: string) => api.createProject({ name }),
    onSuccess: () => {
      setNewName("");
      setCreating(false);
      void queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });

  const updateProject = useMutation({
    mutationFn: (input: { id: number; patch: ProjectPatch }) =>
      api.updateProject(input.id, input.patch),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: ["projects"] }),
  });

  const active = projects.data?.filter((project) => !project.archived) ?? [];
  const archived = projects.data?.filter((project) => project.archived) ?? [];
  const mutationError = createProject.error ?? updateProject.error;

  function errorText(error: unknown): string {
    return error instanceof ApiError ? error.message : "не удалось сохранить изменения";
  }

  function startCreating() {
    createProject.reset();
    setCreating(true);
  }

  return (
    <div className="flex min-h-full flex-col">
      <AppBar />

      <main className="flex-1 px-5 py-8 md:px-12 md:py-12">
        <div className="w-full max-w-[1180px]">
          <h1 className="font-display text-[32px] font-semibold tracking-[-0.02em]">Проекты</h1>
          <p className="mt-1.5 mb-7 text-[13px] text-steel">
            {active.length} активных, {archived.length} в архиве. Цифры — задачи по колонкам.
          </p>

          {mutationError && (
            <p role="alert" className="mb-5 border-l-2 border-signal pl-3 text-[13px]">
              {errorText(mutationError)}
            </p>
          )}

          {projects.isPending && (
            <p className="py-10 text-[14px] text-steel">загружаю проекты…</p>
          )}

          {projects.isError && (
            <div className="py-8">
              <p role="alert" className="border-l-2 border-signal pl-3 text-[13px]">
                {errorText(projects.error)}
              </p>
              <button
                type="button"
                onClick={() => void projects.refetch()}
                className="mt-3 text-[13px] text-steel underline-offset-4 hover:text-ink hover:underline"
              >
                повторить
              </button>
            </div>
          )}

          {!projects.isPending && !projects.isError && active.length > 0 && (
            <ul className="border-t border-ink">
              {active.map((project) => (
                <ProjectRow
                  key={project.id}
                  project={project}
                  archived={false}
                  busy={updateProject.isPending}
                  onOpen={() => navigate(`/projects/${project.id}`)}
                  onRename={(name) => updateProject.mutate({ id: project.id, patch: { name } })}
                  onToggleArchive={() =>
                    updateProject.mutate({ id: project.id, patch: { archived: true } })
                  }
                />
              ))}
            </ul>
          )}

          {!projects.isPending && !projects.isError && active.length === 0 && (
            <div className="border-t border-rule py-10 text-[14px] text-steel">
              Пока нет проектов. Начни с одного — доска появится сразу.
            </div>
          )}

          <div className="mt-5">
            {creating ? (
              <form
                onSubmit={(event) => {
                  event.preventDefault();
                  const name = newName.trim();
                  if (name) createProject.mutate(name);
                }}
                className="flex flex-wrap items-center gap-x-4 gap-y-2"
              >
                <input
                  autoFocus
                  value={newName}
                  aria-label="Название нового проекта"
                  placeholder="название проекта"
                  onChange={(event) => setNewName(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === "Escape") {
                      setCreating(false);
                      setNewName("");
                    }
                  }}
                  className="w-full max-w-[420px] border-b border-rule bg-transparent py-2 text-[15px] outline-none transition-colors focus:border-signal"
                />
                <button
                  type="submit"
                  disabled={!newName.trim() || createProject.isPending}
                  className="text-[13px] text-ink transition-colors hover:text-signal disabled:text-steel"
                >
                  создать
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setCreating(false);
                    setNewName("");
                  }}
                  className="text-[13px] text-steel hover:text-ink"
                >
                  отмена
                </button>
              </form>
            ) : (
              <button
                type="button"
                onClick={startCreating}
                className="flex items-center gap-2 text-[14px] text-steel transition-colors hover:text-signal"
              >
                <span aria-hidden="true" className="font-display text-[20px] leading-none text-ink">
                  +
                </span>
                новый проект
              </button>
            )}
          </div>

          {archived.length > 0 && (
            <div className="mt-8">
              <button
                type="button"
                onClick={() => setShowArchived((shown) => !shown)}
                className="text-[13px] text-steel hover:text-ink"
              >
                архив: {archived.length} {showArchived ? "свернуть" : "показать"}
              </button>

              {showArchived && (
                <ul className="mt-4 border-t border-rule">
                  {archived.map((project) => (
                    <ProjectRow
                      key={project.id}
                      project={project}
                      archived
                      busy={updateProject.isPending}
                      onOpen={() => navigate(`/projects/${project.id}`)}
                      onRename={(name) =>
                        updateProject.mutate({ id: project.id, patch: { name } })
                      }
                      onToggleArchive={() =>
                        updateProject.mutate({ id: project.id, patch: { archived: false } })
                      }
                    />
                  ))}
                </ul>
              )}
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
