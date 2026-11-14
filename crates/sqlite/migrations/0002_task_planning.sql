ALTER TABLE tasks ADD COLUMN due_on TEXT
    CHECK (due_on IS NULL OR due_on GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]');

ALTER TABLE tasks ADD COLUMN priority TEXT NOT NULL DEFAULT 'normal'
    CHECK (priority IN ('low', 'normal', 'high', 'urgent'));

ALTER TABLE tasks ADD COLUMN estimate_minutes INTEGER
    CHECK (estimate_minutes IS NULL OR (estimate_minutes BETWEEN 1 AND 100000));
