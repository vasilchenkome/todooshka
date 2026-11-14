import {
  errorBodySchema,
  loginResponseSchema,
  projectListSchema,
  projectSchema,
  userSchema,
  type Project,
  type ProjectInput,
  type ProjectPatch,
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

async function request<T>(
  path: string,
  init: RequestInit,
  parse: (data: unknown) => T,
): Promise<T> {
  const response = await fetch(path, { credentials: "same-origin", ...init });

  if (!response.ok) {
    const body = await response.json().catch(() => null);
    const parsed = errorBodySchema.safeParse(body);
    if (parsed.success) {
      throw new ApiError(response.status, parsed.data.error.code, parsed.data.error.message);
    }
    throw new ApiError(response.status, "unknown", `запрос не удался (${response.status})`);
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
};
