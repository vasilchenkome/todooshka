import { z } from "zod";

export const userSchema = z.object({
  id: z.number(),
  username: z.string(),
});
export type User = z.infer<typeof userSchema>;

export const loginResponseSchema = z.object({
  user: userSchema,
  expires_at: z.number(),
});

export const statusCountsSchema = z.object({
  backlog: z.number(),
  in_progress: z.number(),
  done: z.number(),
});
export type StatusCounts = z.infer<typeof statusCountsSchema>;

export const projectSchema = z.object({
  id: z.number(),
  name: z.string(),
  description: z.string(),
  color: z.string(),
  position: z.number(),
  archived: z.boolean(),
  created_at: z.number(),
  updated_at: z.number(),
  counts: statusCountsSchema.optional(),
});
export type Project = z.infer<typeof projectSchema>;

export const projectListSchema = z.array(projectSchema);

export const errorBodySchema = z.object({
  error: z.object({
    code: z.string(),
    message: z.string(),
  }),
});

export type ProjectInput = {
  name: string;
  description?: string;
  color?: string;
};

export type ProjectPatch = Partial<
  Pick<Project, "name" | "description" | "color" | "archived">
>;

export const taskStatusSchema = z.enum(["backlog", "in_progress", "done"]);
export type TaskStatus = z.infer<typeof taskStatusSchema>;

export const taskPrioritySchema = z.enum(["low", "normal", "high", "urgent"]);
export type TaskPriority = z.infer<typeof taskPrioritySchema>;

export type TaskPlanning = {
  due_on: string | null;
  priority: TaskPriority;
  estimate_minutes: number | null;
};

export const taskSchema = z.object({
  id: z.number(),
  project_id: z.number(),
  title: z.string(),
  description: z.string(),
  status: taskStatusSchema,
  position: z.number(),
  created_at: z.number(),
  updated_at: z.number(),
  completed_at: z.number().nullable(),
  due_on: z.string().nullable(),
  priority: taskPrioritySchema,
  estimate_minutes: z.number().nullable(),
});
export type Task = z.infer<typeof taskSchema>;

export const taskListSchema = z.array(taskSchema);

export type TaskPatch = Partial<Pick<Task, "title" | "description">> & {
  planning?: TaskPlanning;
};

export type NewTask = {
  title: string;
  description?: string;
  status?: TaskStatus;
};
