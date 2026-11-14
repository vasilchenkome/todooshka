import { useEffect, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";

import { ApiError, api } from "../../api/client";
import type { ProjectPatch } from "../../api/types";
import { useI18n } from "../../i18n";
import { usePageTitle } from "../../shared/lib/usePageTitle";
import { AppBar } from "../../shared/ui/AppBar";
import { ProjectRow } from "./ProjectRow";

export default function ProjectsPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { t } = useI18n();
  usePageTitle(t("page.projects"));
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState("");
  const [showArchived, setShowArchived] = useState(false);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!creating) return;
    if (window.matchMedia("(pointer: fine)").matches) {
      inputRef.current?.focus();
    }
  }, [creating]);

  const projects = useQuery({ queryKey: ["projects"], queryFn: api.listProjects });

  const createProject = useMutation({
    mutationFn: (name: string) => api.createProject({ name }),
    onSuccess: () => {
      setNewName("");
      setCreating(false);
      triggerRef.current?.focus();
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
    return error instanceof ApiError ? error.message : t("errors.save");
  }

  function startCreating() {
    if (creating) {
      inputRef.current?.focus();
      return;
    }
    createProject.reset();
    setCreating(true);
  }

  function closeCreate() {
    setCreating(false);
    setNewName("");
    triggerRef.current?.focus();
  }

  return (
    <div className="flex min-h-full flex-col">
      <AppBar />

      <main className="flex-1 px-5 py-8 md:px-12 md:py-12">
        <div className="w-full max-w-[1180px]">
          <div className="relative flex w-max max-w-full items-center pr-12">
            <h1 className="min-w-0 truncate font-display text-[32px] font-semibold tracking-[-0.02em]">
              {t("projects.title")}
            </h1>
            <button
              ref={triggerRef}
              type="button"
              aria-label={t("projects.newAria")}
              aria-expanded={creating}
              aria-controls="create-project"
              onClick={startCreating}
              className="group absolute top-1/2 right-0 grid h-11 w-11 -translate-y-1/2 place-items-center"
            >
              <span
                aria-hidden="true"
                className={`grid h-[30px] w-[30px] place-items-center border font-display text-[20px] font-semibold leading-none transition-colors ${
                  creating
                    ? "border-signal bg-sheet text-signal"
                    : "border-rule text-ink group-hover:border-signal group-hover:bg-sheet group-hover:text-signal"
                }`}
              >
                +
              </span>
            </button>
          </div>
          <p className="mt-1.5 mb-7 text-[13px] text-steel">
            {t("projects.summary", { active: active.length, archived: archived.length })}
          </p>

          {creating && (
            <form
              id="create-project"
              onSubmit={(event) => {
                event.preventDefault();
                const name = newName.trim();
                if (name) createProject.mutate(name);
              }}
              className="flex flex-wrap items-center gap-x-4 gap-y-2 pb-4"
            >
              <input
                ref={inputRef}
                value={newName}
                aria-label={t("projects.nameAria")}
                placeholder={t("projects.namePlaceholder")}
                onChange={(event) => setNewName(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === "Escape") {
                    closeCreate();
                  }
                }}
                className="w-full max-w-[420px] border-b border-rule bg-transparent py-2 text-[15px] outline-none transition-colors focus:border-signal"
              />
              <button
                type="submit"
                disabled={!newName.trim() || createProject.isPending}
                className="text-[13px] text-ink transition-colors hover:text-signal disabled:text-steel"
              >
                {t("projects.create")}
              </button>
              <button
                type="button"
                onClick={closeCreate}
                className="text-[13px] text-steel hover:text-ink"
              >
                {t("common.cancel")}
              </button>
            </form>
          )}

          {mutationError && (
            <p role="alert" className="mb-5 border-l-2 border-signal pl-3 text-[13px]">
              {errorText(mutationError)}
            </p>
          )}

          {projects.isPending && (
            <p className="py-10 text-[14px] text-steel">{t("projects.loading")}</p>
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
                {t("common.retry")}
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

          {!projects.isPending && !projects.isError && active.length === 0 && !creating && (
            <div className="border-t border-rule py-10 text-[14px] text-steel">
              {t("projects.empty")}
            </div>
          )}

          {archived.length > 0 && (
            <div className="mt-8">
              <button
                type="button"
                onClick={() => setShowArchived((shown) => !shown)}
                className="text-[13px] text-steel hover:text-ink"
              >
                {t("projects.archive", { count: archived.length })}{" "}
                {t(showArchived ? "projects.hide" : "projects.show")}
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
