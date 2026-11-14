import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  DndContext,
  DragOverlay,
  KeyboardSensor,
  PointerSensor,
  TouchSensor,
  closestCenter,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import type { DragEndEvent, DragStartEvent } from "@dnd-kit/core";
import { Link, useParams } from "react-router-dom";

import { ApiError, api } from "../../api/client";
import type { Task, TaskStatus } from "../../api/types";
import { usePageTitle } from "../../shared/lib/usePageTitle";
import { AppBar } from "../../shared/ui/AppBar";
import { BoardColumn } from "./BoardColumn";
import { TaskDialog } from "./TaskDialog";
import {
  BOARD_COLUMNS,
  applyOptimisticMove,
  areOrdered,
  groupByStatus,
  projectNumbers,
  resolveDrop,
} from "./boardModel";

export default function BoardPage() {
  const { projectId } = useParams();
  const id = Number(projectId);
  const queryClient = useQueryClient();
  const taskKey = ["tasks", id] as const;

  const projects = useQuery({ queryKey: ["projects"], queryFn: api.listProjects });
  const project = projects.data?.find((item) => item.id === id);
  usePageTitle(project ? `${project.name} — todooshka` : "Доска — todooshka");

  const tasksQuery = useQuery({
    queryKey: taskKey,
    queryFn: () => api.listTasks(id),
    enabled: Number.isInteger(id) && id > 0,
  });

  const tasks = tasksQuery.data ?? [];
  const columns = groupByStatus(tasks);
  const numbers = projectNumbers(tasks);

  const [dragged, setDragged] = useState<Task | null>(null);
  const [activeStatus, setActiveStatus] = useState<TaskStatus>("backlog");
  const [dialogTask, setDialogTask] = useState<Task | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  function errorText(error: unknown): string {
    return error instanceof ApiError ? error.message : "не удалось сохранить изменения";
  }

  const moveTask = useMutation({
    mutationFn: (input: { id: number; status: TaskStatus; index: number }) =>
      api.moveTask(input.id, input.status, input.index),
    onMutate: async (input) => {
      setActionError(null);
      await queryClient.cancelQueries({ queryKey: taskKey });
      const previous = queryClient.getQueryData<Task[]>(taskKey);
      queryClient.setQueryData<Task[]>(taskKey, (current = []) =>
        applyOptimisticMove(
          current,
          input.id,
          input.status,
          input.index,
          Math.floor(Date.now() / 1000),
        ),
      );
      return { previous };
    },
    onError: (error, _input, context) => {
      if (context?.previous) {
        queryClient.setQueryData(taskKey, context.previous);
      }
      setActionError(errorText(error));
    },
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: taskKey });
      void queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });

  const createTask = useMutation({
    mutationFn: (input: { status: TaskStatus; title: string }) =>
      api.createTask(id, { title: input.title, status: input.status }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: taskKey });
      void queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
    onError: (error) => setActionError(errorText(error)),
  });

  const updateTask = useMutation({
    mutationFn: (input: { id: number; title: string; description: string }) =>
      api.updateTask(input.id, { title: input.title, description: input.description }),
    onSuccess: (updated) => {
      setDialogTask(updated);
      void queryClient.invalidateQueries({ queryKey: taskKey });
    },
    onError: (error) => setActionError(errorText(error)),
  });

  const deleteTask = useMutation({
    mutationFn: (taskId: number) => api.deleteTask(taskId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: taskKey });
      void queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
    onError: (error) => setActionError(errorText(error)),
  });

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 6 } }),
    useSensor(TouchSensor, { activationConstraint: { delay: 180, tolerance: 6 } }),
    useSensor(KeyboardSensor),
  );

  function handleDragStart(event: DragStartEvent) {
    const task = tasks.find((item) => item.id === Number(event.active.id));
    setDragged(task ?? null);
  }

  function handleDragEnd(event: DragEndEvent) {
    setDragged(null);
    const { active, over } = event;
    if (!over) return;

    const activeTaskId = Number(active.id);
    const moved = tasks.find((task) => task.id === activeTaskId);
    if (!moved) return;

    const target = resolveDrop(tasks, activeTaskId, String(over.id));
    if (!target) return;

    if (moved.status === target.status) {
      const column = tasks.filter((task) => task.status === moved.status).sort(areOrdered);
      if (column[target.index]?.id === moved.id) return;
    }

    moveTask.mutate({ id: activeTaskId, status: target.status, index: target.index });
  }

  const busy = updateTask.isPending || moveTask.isPending || deleteTask.isPending;

  return (
    <div className="flex min-h-full flex-col">
      <AppBar
        left={
          <>
            <Link
              to="/"
              aria-label="К проектам"
              className="shrink-0 border-b border-transparent pb-px text-[13px] text-steel transition-colors hover:border-rule hover:text-ink"
            >
              ←<span className="hidden sm:inline"> проекты</span>
            </Link>
            <h1 className="min-w-0 truncate font-display text-[18px] font-semibold tracking-[-0.01em]">
              {project?.name ?? "Доска"}
            </h1>
          </>
        }
      />

      <main className="flex flex-1 flex-col">
        {actionError && (
          <p role="alert" className="border-l-2 border-signal px-4 py-2 text-[13px] md:mx-3">
            {actionError}
          </p>
        )}

        <nav className="flex gap-5 overflow-x-auto border-b border-rule px-4 md:hidden">
          {BOARD_COLUMNS.map(({ status, title }) => (
            <button
              key={status}
              type="button"
              onClick={() => setActiveStatus(status)}
              className={`-mb-px border-b-2 py-3 text-[13px] whitespace-nowrap transition-colors ${
                activeStatus === status
                  ? "border-signal text-ink"
                  : "border-transparent text-steel"
              }`}
            >
              {title}
              <span className="ml-1 text-steel tabular-nums">{columns[status].length}</span>
            </button>
          ))}
        </nav>

        {tasksQuery.isPending && (
          <p className="px-4 py-10 text-[14px] text-steel md:px-7">загружаю доску…</p>
        )}

        {tasksQuery.isError && (
          <div className="px-4 py-8 md:px-7">
            <p role="alert" className="border-l-2 border-signal pl-3 text-[13px]">
              {errorText(tasksQuery.error)}
            </p>
            <button
              type="button"
              onClick={() => void tasksQuery.refetch()}
              className="mt-3 text-[13px] text-steel hover:text-ink"
            >
              повторить
            </button>
          </div>
        )}

        {!tasksQuery.isPending && !tasksQuery.isError && (
          <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragStart={handleDragStart}
            onDragEnd={handleDragEnd}
            onDragCancel={() => setDragged(null)}
          >
            <div className="flex-1 md:grid md:grid-cols-3 md:divide-x md:divide-rule">
              {BOARD_COLUMNS.map(({ status, title }) => (
                <BoardColumn
                  key={status}
                  status={status}
                  title={title}
                  tasks={columns[status]}
                  numbers={numbers}
                  accent={status === "in_progress"}
                  activeOnMobile={activeStatus === status}
                  creating={createTask.isPending}
                  onOpenTask={setDialogTask}
                  onCreate={async (column, title) => {
                    await createTask.mutateAsync({ status: column, title });
                  }}
                />
              ))}
            </div>

            <DragOverlay>
              {dragged && (
                <div className="border border-ink bg-paper px-4 py-3 text-[15px] font-medium">
                  {dragged.title}
                </div>
              )}
            </DragOverlay>
          </DndContext>
        )}
      </main>

      {dialogTask && (
        <TaskDialog
          key={dialogTask.id}
          task={dialogTask}
          busy={busy}
          onClose={() => setDialogTask(null)}
          onSave={async (patch) => {
            await updateTask.mutateAsync({ id: dialogTask.id, ...patch });
          }}
          onMove={async (status) => {
            const index = tasks.filter((task) => task.status === status).length;
            await moveTask.mutateAsync({ id: dialogTask.id, status, index });
          }}
          onDelete={async () => {
            await deleteTask.mutateAsync(dialogTask.id);
          }}
        />
      )}
    </div>
  );
}
