import { z } from "zod";

import {
  errorBodySchema,
  loginResponseSchema,
  projectListSchema,
  projectSchema,
  attachmentSchema,
  commentSchema,
  subtaskListSchema,
  subtaskSchema,
  tagSchema,
  taskListSchema,
  taskSchema,
  taskTagSchema,
  userSchema,
  type NewTask,
  type Project,
  type ProjectInput,
  type ProjectPatch,
  type Attachment,
  type Comment,
  type Subtask,
  type Tag,
  type Task,
  type TaskPatch,
  type TaskTag,
  type TaskStatus,
  type User,
} from "./types";

export class ApiError extends Error {
  readonly status: number;
  readonly code: string;

  constructor(status: number, code: string, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.code = code;
  }
}

const JSON_HEADERS = { "Content-Type": "application/json" } as const;

let apiLocale: "ru" | "en" = "ru";
try {
  const stored = localStorage.getItem("todooshka.locale");
  if (stored === "en") apiLocale = "en";
} catch {
  // localStorage may be unavailable; Russian remains the safe default.
}

export function setApiLocale(locale: "ru" | "en") {
  apiLocale = locale;
}

async function request<T>(
  path: string,
  init: RequestInit,
  parse: (data: unknown) => T,
): Promise<T> {
  const headers = new Headers(init.headers);
  headers.set("Accept-Language", apiLocale);
  const response = await fetch(path, { credentials: "same-origin", ...init, headers });

  if (!response.ok) {
    const body = await response.json().catch(() => null);
    const parsed = errorBodySchema.safeParse(body);
    if (parsed.success) {
      throw new ApiError(response.status, parsed.data.error.code, parsed.data.error.message);
    }
    throw new ApiError(
      response.status,
      "unknown",
      apiLocale === "en"
        ? `request failed (${response.status})`
        : `запрос не удался (${response.status})`,
    );
  }

  if (response.status === 204) {
    return parse(undefined);
  }

  return parse(await response.json());
}

export const api = {
  me: (): Promise<User> =>
    request("/api/auth/me", { method: "GET" }, (data) => userSchema.parse(data)),

  login: (username: string, password: string): Promise<User> =>
    request(
      "/api/auth/login",
      { method: "POST", headers: JSON_HEADERS, body: JSON.stringify({ username, password }) },
      (data) => loginResponseSchema.parse(data).user,
    ),

  logout: (): Promise<void> =>
    request("/api/auth/logout", { method: "POST" }, () => undefined),

  listProjects: (): Promise<Project[]> =>
    request("/api/projects?include_archived=true", { method: "GET" }, (data) =>
      projectListSchema.parse(data),
    ),

  createProject: (input: ProjectInput): Promise<Project> =>
    request(
      "/api/projects",
      { method: "POST", headers: JSON_HEADERS, body: JSON.stringify(input) },
      (data) => projectSchema.parse(data),
    ),

  updateProject: (id: number, patch: ProjectPatch): Promise<Project> =>
    request(
      `/api/projects/${id}`,
      { method: "PATCH", headers: JSON_HEADERS, body: JSON.stringify(patch) },
      (data) => projectSchema.parse(data),
    ),

  listTasks: (projectId: number): Promise<Task[]> =>
    request(`/api/projects/${projectId}/tasks`, { method: "GET" }, (data) =>
      taskListSchema.parse(data),
    ),

  createTask: (projectId: number, input: NewTask): Promise<Task> =>
    request(
      `/api/projects/${projectId}/tasks`,
      { method: "POST", headers: JSON_HEADERS, body: JSON.stringify(input) },
      (data) => taskSchema.parse(data),
    ),

  updateTask: (id: number, patch: TaskPatch): Promise<Task> =>
    request(
      `/api/tasks/${id}`,
      { method: "PATCH", headers: JSON_HEADERS, body: JSON.stringify(patch) },
      (data) => taskSchema.parse(data),
    ),

  moveTask: (id: number, status: TaskStatus, targetIndex: number): Promise<Task> =>
    request(
      `/api/tasks/${id}/move`,
      { method: "POST", headers: JSON_HEADERS, body: JSON.stringify({ status, target_index: targetIndex }) },
      (data) => taskSchema.parse(data),
    ),

  deleteTask: (id: number): Promise<void> =>
    request(`/api/tasks/${id}`, { method: "DELETE" }, () => undefined),

  listSubtasks: (projectId: number): Promise<Subtask[]> =>
    request(`/api/projects/${projectId}/subtasks`, { method: "GET" }, (data) =>
      subtaskListSchema.parse(data),
    ),

  createSubtask: (taskId: number, title: string): Promise<Subtask> =>
    request(
      `/api/tasks/${taskId}/subtasks`,
      { method: "POST", headers: JSON_HEADERS, body: JSON.stringify({ title }) },
      (data) => subtaskSchema.parse(data),
    ),

  updateSubtask: (
    id: number,
    patch: { title?: string; done?: boolean },
  ): Promise<Subtask> =>
    request(
      `/api/subtasks/${id}`,
      { method: "PATCH", headers: JSON_HEADERS, body: JSON.stringify(patch) },
      (data) => subtaskSchema.parse(data),
    ),

  deleteSubtask: (id: number): Promise<void> =>
    request(`/api/subtasks/${id}`, { method: "DELETE" }, () => undefined),

  listTags: (): Promise<Tag[]> =>
    request("/api/tags", { method: "GET" }, (data) => z.array(tagSchema).parse(data)),

  listTaskTags: (projectId: number): Promise<TaskTag[]> =>
    request(`/api/projects/${projectId}/task-tags`, { method: "GET" }, (data) =>
      z.array(taskTagSchema).parse(data),
    ),

  replaceTaskTags: (taskId: number, tags: string[]): Promise<Tag[]> =>
    request(
      `/api/tasks/${taskId}/tags`,
      { method: "PUT", headers: JSON_HEADERS, body: JSON.stringify({ tags }) },
      (data) => z.array(tagSchema).parse(data),
    ),

  listComments: (projectId: number): Promise<Comment[]> =>
    request(`/api/projects/${projectId}/comments`, { method: "GET" }, (data) =>
      z.array(commentSchema).parse(data),
    ),

  createComment: (taskId: number, body: string): Promise<Comment> =>
    request(
      `/api/tasks/${taskId}/comments`,
      { method: "POST", headers: JSON_HEADERS, body: JSON.stringify({ body }) },
      (data) => commentSchema.parse(data),
    ),

  updateComment: (id: number, body: string): Promise<Comment> =>
    request(
      `/api/comments/${id}`,
      { method: "PATCH", headers: JSON_HEADERS, body: JSON.stringify({ body }) },
      (data) => commentSchema.parse(data),
    ),

  deleteComment: (id: number): Promise<void> =>
    request(`/api/comments/${id}`, { method: "DELETE" }, () => undefined),

  listAttachments: (projectId: number): Promise<Attachment[]> =>
    request(`/api/projects/${projectId}/attachments`, { method: "GET" }, (data) =>
      z.array(attachmentSchema).parse(data),
    ),

  uploadAttachment: (taskId: number, file: File): Promise<Attachment> => {
    const body = new FormData();
    body.append("file", file);
    return request(
      `/api/tasks/${taskId}/attachments`,
      { method: "POST", body },
      (data) => attachmentSchema.parse(data),
    );
  },

  deleteAttachment: (id: number): Promise<void> =>
    request(`/api/attachments/${id}`, { method: "DELETE" }, () => undefined),
};
