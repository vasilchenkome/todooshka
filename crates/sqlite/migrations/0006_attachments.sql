CREATE TABLE attachments (
    id           INTEGER PRIMARY KEY,
    task_id      INTEGER NOT NULL REFERENCES tasks (id) ON DELETE CASCADE,
    filename     TEXT    NOT NULL,
    content_type TEXT    NOT NULL,
    size         INTEGER NOT NULL CHECK (size > 0),
    data         BLOB    NOT NULL,
    created_at   INTEGER NOT NULL
);

CREATE INDEX attachments_task ON attachments (task_id, id);
