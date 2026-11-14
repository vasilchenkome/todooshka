import { useEffect, useState } from "react";
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
import { Link, useParams, useSearchParams } from "react-router-dom";

import { ApiError, api } from "../../api/client";
import type { Task, TaskPlanning, TaskPriority, TaskStatus } from "../../api/types";
import { useI18n } from "../../i18n";
import { usePageTitle } from "../../shared/lib/usePageTitle";
import { AppBar } from "../../shared/ui/AppBar";
import { BoardColumn } from "./BoardColumn";
import { TaskDialog } from "./TaskDialog";
import {
  BOARD_COLUMNS,
  applyOptimisticMove,
  areOrdered,
  filterTasks,
  groupByStatus,
  priorityLabel,
  projectNumbers,
  resolveDrop,
  statusLabel,
  type BoardFilters,
} from "./boardModel";

export default function BoardPage() {
  const { projectId } = useParams();
  const id = Number(projectId);
  const queryClient = useQueryClient();
  const { t, locale } = useI18n();
  const taskKey = ["tasks", id] as const;

  const projects = useQuery({ queryKey: ["projects"], queryFn: api.listProjects });
  const project = projects.data?.find((item) => item.id === id);
  usePageTitle(project ? t("page.project", { name: project.name }) : t("page.board"));

  const tasksQuery = useQuery({
    queryKey: taskKey,
    queryFn: () => api.listTasks(id),
    enabled: Number.isInteger(id) && id > 0,
  });

  const subtasksQuery = useQuery({
    queryKey: ["subtasks", id],
    queryFn: () => api.listSubtasks(id),
    enabled: Number.isInteger(id) && id > 0,
  });

  const commentsQuery = useQuery({
    queryKey: ["comments", id],
    queryFn: () => api.listComments(id),
    enabled: Number.isInteger(id) && id > 0,
  });

  const attachmentsQuery = useQuery({
    queryKey: ["attachments", id],
    queryFn: () => api.listAttachments(id),
    enabled: Number.isInteger(id) && id > 0,
  });

  const tasks = tasksQuery.data ?? [];
  const subtasks = subtasksQuery.data ?? [];
  const comments = commentsQuery.data ?? [];
  const attachments = attachmentsQuery.data ?? [];

  const attachmentCounts = new Map<number, number>();
  for (const file of attachments) {
    attachmentCounts.set(file.task_id, (attachmentCounts.get(file.task_id) ?? 0) + 1);
  }

  const commentCounts = new Map<number, number>();
  for (const comment of comments) {
    commentCounts.set(comment.task_id, (commentCounts.get(comment.task_id) ?? 0) + 1);
  }
  const numbers = projectNumbers(tasks);

  const subtaskCounts = new Map<number, { done: number; total: number }>();
  for (const subtask of subtasks) {
    const current = subtaskCounts.get(subtask.task_id) ?? { done: 0, total: 0 };
    current.total += 1;
    if (subtask.done) current.done += 1;
    subtaskCounts.set(subtask.task_id, current);
  }

  const [searchParams, setSearchParams] = useSearchParams();
  const filters: BoardFilters = {
    query: searchParams.get("q") ?? "",
    priority: (searchParams.get("priority") as TaskPriority | null) ?? "",
    due: (searchParams.get("due") as BoardFilters["due"] | null) ?? "",
    tag: searchParams.get("tag") ?? "",
  };
  const filterActive =
    filters.query !== "" || filters.priority !== "" || filters.due !== "" || filters.tag !== "";

  const tagLinksQuery = useQuery({
    queryKey: ["task-tags", id],
    queryFn: () => api.listTaskTags(id),
    enabled: Number.isInteger(id) && id > 0,
  });
  const allTagsQuery = useQuery({ queryKey: ["tags"], queryFn: api.listTags });

  const tagsByTask = new Map<number, string[]>();
  for (const link of tagLinksQuery.data ?? []) {
    const current = tagsByTask.get(link.task_id) ?? [];
    current.push(link.name);
    tagsByTask.set(link.task_id, current);
  }

  const visibleTasks = filterTasks(tasks, filters, tagsByTask);
  const visibleColumns = groupByStatus(visibleTasks);

  const [dragged, setDragged] = useState<Task | null>(null);
  const [activeStatus, setActiveStatus] = useState<TaskStatus>("backlog");
  const [mobileAddSignal, setMobileAddSignal] = useState(0);
  const [dialogTask, setDialogTask] = useState<Task | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const linkedTaskId = Number(searchParams.get("task"));

  useEffect(() => {
    if (!Number.isInteger(linkedTaskId) || linkedTaskId <= 0) return;
    const linked = tasks.find((task) => task.id === linkedTaskId);
    if (!linked) return;
    setDialogTask(linked);
    setActiveStatus(linked.status);
  }, [linkedTaskId, tasks]);

  function closeDialog() {
    setDialogTask(null);
    if (searchParams.has("task")) {
      const next = new URLSearchParams(searchParams);
      next.delete("task");
      setSearchParams(next, { replace: true });
    }
  }

  function errorText(error: unknown, fallback = t("errors.save")): string {
    return error instanceof ApiError ? error.message : fallback;
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
    mutationFn: (input: {
      id: number;
      title: string;
      description: string;
      planning: TaskPlanning;
    }) =>
      api.updateTask(input.id, {
        title: input.title,
        description: input.description,
        planning: input.planning,
      }),
    onSuccess: (updated) => {
      setDialogTask(updated);
      void queryClient.invalidateQueries({ queryKey: taskKey });
    },
    onError: (error) => setActionError(errorText(error)),
  });

  const createSubtask = useMutation({
    mutationFn: (input: { taskId: number; title: string }) =>
      api.createSubtask(input.taskId, input.title),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["subtasks", id] });
    },
    onError: (error) => setActionError(errorText(error)),
  });

  const updateSubtask = useMutation({
    mutationFn: (input: { id: number; done: boolean }) =>
      api.updateSubtask(input.id, { done: input.done }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["subtasks", id] });
    },
    onError: (error) => setActionError(errorText(error)),
  });

  const deleteSubtask = useMutation({
    mutationFn: (subtaskId: number) => api.deleteSubtask(subtaskId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["subtasks", id] });
    },
    onError: (error) => setActionError(errorText(error)),
  });

  const uploadAttachment = useMutation({
    mutationFn: (input: { taskId: number; file: File }) =>
      api.uploadAttachment(input.taskId, input.file),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["attachments", id] });
    },
    onError: (error) => setActionError(errorText(error)),
  });

  const deleteAttachment = useMutation({
    mutationFn: (attachmentId: number) => api.deleteAttachment(attachmentId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["attachments", id] });
    },
    onError: (error) => setActionError(errorText(error)),
  });

  const createComment = useMutation({
    mutationFn: (input: { taskId: number; body: string }) =>
      api.createComment(input.taskId, input.body),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["comments", id] });
    },
    onError: (error) => setActionError(errorText(error)),
  });

  const updateComment = useMutation({
    mutationFn: (input: { id: number; body: string }) =>
      api.updateComment(input.id, input.body),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["comments", id] });
    },
    onError: (error) => setActionError(errorText(error)),
  });

  const deleteComment = useMutation({
    mutationFn: (commentId: number) => api.deleteComment(commentId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["comments", id] });
    },
    onError: (error) => setActionError(errorText(error)),
  });

  const replaceTags = useMutation({
    mutationFn: (input: { taskId: number; tags: string[] }) =>
      api.replaceTaskTags(input.taskId, input.tags),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["task-tags", id] });
      void queryClient.invalidateQueries({ queryKey: ["tags"] });
      setActionError(null);
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

  function setFilter(key: keyof BoardFilters, value: string) {
    const param = key === "query" ? "q" : key;
    const next = new URLSearchParams(searchParams);
    if (value) next.set(param, value);
    else next.delete(param);
    setSearchParams(next, { replace: true });
  }

  function resetFilters() {
    setSearchParams(new URLSearchParams(), { replace: true });
  }

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
    <div className="flex h-dvh min-h-0 flex-col overflow-hidden">
      <AppBar
        left={
          <>
            <Link
              to="/"
              aria-label={t("board.backAria")}
              className="shrink-0 border-b border-transparent pb-px text-[13px] text-steel transition-colors hover:border-rule hover:text-ink"
            >
              ←<span className="hidden sm:inline"> {t("board.back")}</span>
            </Link>
            <h1 className="min-w-0 truncate font-display text-[18px] font-semibold tracking-[-0.01em]">
              {project?.name ?? t("board.fallback")}
            </h1>
          </>
        }
        right={
          <Link
            to="/today"
            className="border-b border-transparent pb-px text-[13px] text-steel transition-colors hover:border-rule hover:text-ink"
          >
            {t("nav.today")}
          </Link>
        }
      />

      <main className="flex min-h-0 flex-1 flex-col">
        {actionError && (
          <p role="alert" className="border-l-2 border-signal px-4 py-2 text-[13px] md:mx-3">
            {actionError}
          </p>
        )}

        <div className="sticky top-[60px] z-10 shrink-0 bg-paper">
          <div className="flex border-b border-rule md:hidden">
            <nav className="mobile-tabs flex min-w-0 flex-1 gap-5 overflow-x-auto overflow-y-hidden px-4" aria-label={t("board.statusAria")}>
              {BOARD_COLUMNS.map((status) => (
                <button
                  key={status}
                  type="button"
                  onClick={() => {
                    setActiveStatus(status);
                    setMobileAddSignal(0);
                  }}
                  className={`-mb-px border-b-2 py-3 text-[13px] whitespace-nowrap transition-colors ${
                    activeStatus === status
                      ? "border-signal text-ink"
                      : "border-transparent text-steel"
                  }`}
                >
                  {statusLabel(status, locale)}
                  <span className="ml-1 text-steel tabular-nums">{visibleColumns[status].length}</span>
                </button>
              ))}
            </nav>
            <button
              type="button"
              aria-label={t("board.addColumnAria", { column: statusLabel(activeStatus, locale) })}
              onClick={() => setMobileAddSignal((value) => value + 1)}
              className="grid h-11 w-11 shrink-0 place-items-center"
            >
              <span aria-hidden="true" className="grid h-7 w-7 place-items-center border border-rule font-display text-[18px] font-semibold leading-none">
                +
              </span>
            </button>
          </div>

          <div className="flex flex-wrap items-center gap-2 px-4 py-2.5 md:px-6">
          <input
            value={filters.query}
            onChange={(event) => setFilter("query", event.target.value)}
            placeholder={t("board.searchPlaceholder")}
            aria-label={t("board.searchAria")}
            className="min-w-[150px] flex-1 border-b border-rule bg-transparent py-1 text-[13px] outline-none focus:border-signal md:max-w-[280px]"
          />
          <select
            value={filters.priority}
            onChange={(event) => setFilter("priority", event.target.value)}
            aria-label={t("dialog.priority")}
            className="border border-rule bg-paper px-2 py-1.5 text-[12px] text-steel outline-none focus:border-ink"
          >
            <option value="">{t("board.priorityAny")}</option>
            {(["low", "normal", "high", "urgent"] as const).map((value) => (
              <option key={value} value={value}>
                {priorityLabel(value, locale)}
              </option>
            ))}
          </select>
          <select
            value={filters.due}
            onChange={(event) => setFilter("due", event.target.value)}
            aria-label={t("dialog.due")}
            className="border border-rule bg-paper px-2 py-1.5 text-[12px] text-steel outline-none focus:border-ink"
          >
            <option value="">{t("board.dueAny")}</option>
            <option value="overdue">{t("board.dueOverdue")}</option>
            <option value="today">{t("board.dueToday")}</option>
            <option value="week">{t("board.dueWeek")}</option>
            <option value="none">{t("board.dueNone")}</option>
          </select>
          <select
            value={filters.tag}
            onChange={(event) => setFilter("tag", event.target.value)}
            aria-label={t("dialog.tags")}
            className="max-w-[150px] border border-rule bg-paper px-2 py-1.5 text-[12px] text-steel outline-none focus:border-ink"
          >
            <option value="">{t("board.tagAll")}</option>
            {(allTagsQuery.data ?? []).map((tag) => (
              <option key={tag.id} value={tag.name}>
                {tag.name}
              </option>
            ))}
          </select>
          {filterActive && (
            <button
              type="button"
              onClick={resetFilters}
              className="ml-auto text-[12px] text-steel hover:text-ink"
            >
              {t("board.reset")}
            </button>
          )}
          </div>
        </div>

        {filterActive && visibleTasks.length === 0 && !tasksQuery.isPending && (
          <p className="px-4 py-8 text-[14px] text-steel md:px-7">
            {t("board.noMatches")}
          </p>
        )}

        {tasksQuery.isPending && (
          <p className="px-4 py-10 text-[14px] text-steel md:px-7">{t("board.loading")}</p>
        )}

        {tasksQuery.isError && (
          <div className="px-4 py-8 md:px-7">
            <p role="alert" className="border-l-2 border-signal pl-3 text-[13px]">
              {errorText(tasksQuery.error, t("errors.load"))}
            </p>
            <button
              type="button"
              onClick={() => void tasksQuery.refetch()}
              className="mt-3 text-[13px] text-steel hover:text-ink"
            >
              {t("common.retry")}
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
            <div className="flex min-h-0 flex-1 overflow-hidden md:grid md:grid-cols-3 md:divide-x md:divide-rule">
              {BOARD_COLUMNS.map((status) => (
                <BoardColumn
                  key={status}
                  status={status}
                  title={statusLabel(status, locale)}
                  tasks={visibleColumns[status]}
                  numbers={numbers}
                  subtasks={subtaskCounts}
                  tags={tagsByTask}
                  comments={commentCounts}
                  attachments={attachmentCounts}
                  filtered={filterActive}
                  accent={status === "in_progress"}
                  activeOnMobile={activeStatus === status}
                  creating={createTask.isPending}
                  addRequest={status === activeStatus ? mobileAddSignal : 0}
                  onOpenTask={setDialogTask}
                  onCreate={async (column, title) => {
                    await createTask.mutateAsync({ status: column, title });
                  }}
                />
              ))}
            </div>

            <DragOverlay className="z-30">
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
          onClose={closeDialog}
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
          attachments={attachments.filter((file) => file.task_id === dialogTask.id)}
          onUploadAttachment={async (file) => {
            await uploadAttachment.mutateAsync({ taskId: dialogTask.id, file });
          }}
          onDeleteAttachment={async (attachmentId) => {
            await deleteAttachment.mutateAsync(attachmentId);
          }}
          comments={comments.filter((comment) => comment.task_id === dialogTask.id)}
          onCreateComment={async (body) => {
            await createComment.mutateAsync({ taskId: dialogTask.id, body });
          }}
          onEditComment={async (commentId, body) => {
            await updateComment.mutateAsync({ id: commentId, body });
          }}
          onDeleteComment={async (commentId) => {
            await deleteComment.mutateAsync(commentId);
          }}
          tags={tagsByTask.get(dialogTask.id) ?? []}
          tagSuggestions={(allTagsQuery.data ?? []).map((tag) => tag.name)}
          onReplaceTags={async (nextTags) => {
            await replaceTags.mutateAsync({ taskId: dialogTask.id, tags: nextTags });
          }}
          subtasks={subtasks.filter((subtask) => subtask.task_id === dialogTask.id)}
          onCreateSubtask={async (title) => {
            await createSubtask.mutateAsync({ taskId: dialogTask.id, title });
          }}
          onToggleSubtask={async (subtaskId, done) => {
            await updateSubtask.mutateAsync({ id: subtaskId, done });
          }}
          onDeleteSubtask={async (subtaskId) => {
            await deleteSubtask.mutateAsync(subtaskId);
          }}
        />
      )}
    </div>
  );
}
