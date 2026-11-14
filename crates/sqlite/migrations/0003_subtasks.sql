CREATE TABLE subtasks (
    id         INTEGER PRIMARY KEY,
    task_id    INTEGER NOT NULL REFERENCES tasks (id) ON DELETE CASCADE,
    title      TEXT    NOT NULL,
    done       INTEGER NOT NULL DEFAULT 0 CHECK (done IN (0, 1)),
    position   REAL    NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX subtasks_task ON subtasks (task_id, position, id);
