-- Balager initial schema

CREATE TABLE users (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL,
    email         TEXT NOT NULL UNIQUE COLLATE NOCASE,
    password_hash TEXT NOT NULL,
    color         TEXT NOT NULL DEFAULT '#3f8aa3',
    role          TEXT NOT NULL DEFAULT 'normal' CHECK (role IN ('normal', 'approver')),
    active        INTEGER NOT NULL DEFAULT 1,
    created_at    INTEGER NOT NULL
);

CREATE TABLE sessions (
    token      TEXT PRIMARY KEY,
    user_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at INTEGER NOT NULL
);

CREATE TABLE reservations (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT NOT NULL,
    day_from   TEXT NOT NULL, -- ISO yyyy-mm-dd, inclusive
    day_to     TEXT NOT NULL, -- ISO yyyy-mm-dd, inclusive
    owner_id   INTEGER NOT NULL REFERENCES users(id),
    access     TEXT NOT NULL DEFAULT 'closed' CHECK (access IN ('closed', 'open')),
    note       TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL
);

CREATE TABLE reservation_approvals (
    reservation_id INTEGER NOT NULL REFERENCES reservations(id) ON DELETE CASCADE,
    approver_id    INTEGER NOT NULL REFERENCES users(id),
    status         TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    comment        TEXT NOT NULL DEFAULT '',
    decided_at     INTEGER,
    PRIMARY KEY (reservation_id, approver_id)
);

CREATE TABLE reservation_attendees (
    reservation_id INTEGER NOT NULL REFERENCES reservations(id) ON DELETE CASCADE,
    user_id        INTEGER NOT NULL REFERENCES users(id),
    PRIMARY KEY (reservation_id, user_id)
);

CREATE TABLE task_groups (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    name     TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE tasks (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id       INTEGER NOT NULL REFERENCES task_groups(id) ON DELETE CASCADE,
    title          TEXT NOT NULL,
    done           INTEGER NOT NULL DEFAULT 0,
    due            TEXT,            -- ISO yyyy-mm-dd
    assignee_id    INTEGER REFERENCES users(id),
    recurring      TEXT CHECK (recurring IN ('weekly', 'biweekly', 'monthly', 'yearly')),
    reservation_id INTEGER REFERENCES reservations(id) ON DELETE SET NULL,
    created_at     INTEGER NOT NULL
);

CREATE TABLE subtasks (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    title   TEXT NOT NULL,
    done    INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE threads (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT NOT NULL,
    kind       TEXT NOT NULL DEFAULT 'general' CHECK (kind IN ('general', 'reservation', 'task')),
    link_id    INTEGER,
    author_id  INTEGER NOT NULL REFERENCES users(id),
    closed     INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE TABLE messages (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    thread_id  INTEGER NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
    parent_id  INTEGER REFERENCES messages(id) ON DELETE CASCADE,
    author_id  INTEGER REFERENCES users(id),  -- NULL for system messages
    body       TEXT NOT NULL,
    system     INTEGER NOT NULL DEFAULT 0,
    pinned     INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE TABLE message_votes (
    message_id INTEGER NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id    INTEGER NOT NULL REFERENCES users(id),
    value      INTEGER NOT NULL CHECK (value IN (-1, 1)),
    PRIMARY KEY (message_id, user_id)
);

CREATE TABLE polls (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL UNIQUE REFERENCES messages(id) ON DELETE CASCADE,
    question   TEXT NOT NULL,
    ptype      TEXT NOT NULL DEFAULT 'list' CHECK (ptype IN ('date', 'list')),
    mode       TEXT NOT NULL DEFAULT 'single' CHECK (mode IN ('single', 'multi'))
);

CREATE TABLE poll_options (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    poll_id INTEGER NOT NULL REFERENCES polls(id) ON DELETE CASCADE,
    label   TEXT NOT NULL,
    sub     TEXT NOT NULL DEFAULT ''
);

CREATE TABLE poll_votes (
    option_id INTEGER NOT NULL REFERENCES poll_options(id) ON DELETE CASCADE,
    user_id   INTEGER NOT NULL REFERENCES users(id),
    PRIMARY KEY (option_id, user_id)
);

CREATE TABLE notifications (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    icon       TEXT NOT NULL DEFAULT 'info',
    tone       TEXT NOT NULL DEFAULT '',
    text       TEXT NOT NULL,
    link_kind  TEXT,    -- 'reservation' | 'task' | 'thread'
    link_id    INTEGER,
    read       INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE TABLE notif_prefs (
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key     TEXT NOT NULL,
    email   INTEGER NOT NULL DEFAULT 1,
    push    INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (user_id, key)
);

CREATE INDEX idx_reservations_days ON reservations(day_from, day_to);
CREATE INDEX idx_messages_thread ON messages(thread_id);
CREATE INDEX idx_notifications_user ON notifications(user_id, read);
CREATE INDEX idx_threads_link ON threads(kind, link_id);
