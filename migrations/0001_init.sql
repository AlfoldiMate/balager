-- Balager initial schema (PostgreSQL)

CREATE TABLE users (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name          TEXT NOT NULL,
    email         TEXT NOT NULL UNIQUE, -- stored lowercased
    password_hash TEXT NOT NULL,
    color         TEXT NOT NULL DEFAULT '#3f8aa3',
    role          TEXT NOT NULL DEFAULT 'normal' CHECK (role IN ('normal', 'approver')),
    active        BIGINT NOT NULL DEFAULT 1,
    created_at    BIGINT NOT NULL
);

CREATE TABLE sessions (
    token      TEXT PRIMARY KEY,
    user_id    BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at BIGINT NOT NULL
);

CREATE TABLE reservations (
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    title      TEXT NOT NULL,
    day_from   TEXT NOT NULL, -- ISO yyyy-mm-dd, inclusive
    day_to     TEXT NOT NULL, -- ISO yyyy-mm-dd, inclusive
    owner_id   BIGINT NOT NULL REFERENCES users(id),
    access     TEXT NOT NULL DEFAULT 'closed' CHECK (access IN ('closed', 'open')),
    note       TEXT NOT NULL DEFAULT '',
    created_at BIGINT NOT NULL
);

CREATE TABLE reservation_approvals (
    reservation_id BIGINT NOT NULL REFERENCES reservations(id) ON DELETE CASCADE,
    approver_id    BIGINT NOT NULL REFERENCES users(id),
    status         TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    comment        TEXT NOT NULL DEFAULT '',
    decided_at     BIGINT,
    PRIMARY KEY (reservation_id, approver_id)
);

CREATE TABLE reservation_attendees (
    reservation_id BIGINT NOT NULL REFERENCES reservations(id) ON DELETE CASCADE,
    user_id        BIGINT NOT NULL REFERENCES users(id),
    PRIMARY KEY (reservation_id, user_id)
);

CREATE TABLE task_groups (
    id       BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name     TEXT NOT NULL,
    position BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE tasks (
    id             BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    group_id       BIGINT NOT NULL REFERENCES task_groups(id) ON DELETE CASCADE,
    title          TEXT NOT NULL,
    done           BIGINT NOT NULL DEFAULT 0,
    due            TEXT,            -- ISO yyyy-mm-dd
    assignee_id    BIGINT REFERENCES users(id),
    recurring      TEXT CHECK (recurring IN ('weekly', 'biweekly', 'monthly', 'yearly')),
    reservation_id BIGINT REFERENCES reservations(id) ON DELETE SET NULL,
    created_at     BIGINT NOT NULL
);

CREATE TABLE subtasks (
    id      BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    task_id BIGINT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    title   TEXT NOT NULL,
    done    BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE threads (
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    title      TEXT NOT NULL,
    kind       TEXT NOT NULL DEFAULT 'general' CHECK (kind IN ('general', 'reservation', 'task')),
    link_id    BIGINT,
    author_id  BIGINT NOT NULL REFERENCES users(id),
    closed     BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL
);

CREATE TABLE messages (
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    thread_id  BIGINT NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
    parent_id  BIGINT REFERENCES messages(id) ON DELETE CASCADE,
    author_id  BIGINT REFERENCES users(id),  -- NULL for system messages
    body       TEXT NOT NULL,
    system     BIGINT NOT NULL DEFAULT 0,
    pinned     BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL
);

CREATE TABLE message_votes (
    message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id    BIGINT NOT NULL REFERENCES users(id),
    value      BIGINT NOT NULL CHECK (value IN (-1, 1)),
    PRIMARY KEY (message_id, user_id)
);

CREATE TABLE polls (
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    message_id BIGINT NOT NULL UNIQUE REFERENCES messages(id) ON DELETE CASCADE,
    question   TEXT NOT NULL,
    ptype      TEXT NOT NULL DEFAULT 'list' CHECK (ptype IN ('date', 'list')),
    mode       TEXT NOT NULL DEFAULT 'single' CHECK (mode IN ('single', 'multi'))
);

CREATE TABLE poll_options (
    id      BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    poll_id BIGINT NOT NULL REFERENCES polls(id) ON DELETE CASCADE,
    label   TEXT NOT NULL,
    sub     TEXT NOT NULL DEFAULT ''
);

CREATE TABLE poll_votes (
    option_id BIGINT NOT NULL REFERENCES poll_options(id) ON DELETE CASCADE,
    user_id   BIGINT NOT NULL REFERENCES users(id),
    PRIMARY KEY (option_id, user_id)
);

CREATE TABLE notifications (
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id    BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    icon       TEXT NOT NULL DEFAULT 'info',
    tone       TEXT NOT NULL DEFAULT '',
    text       TEXT NOT NULL,
    link_kind  TEXT,    -- 'reservation' | 'task' | 'thread'
    link_id    BIGINT,
    read       BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL
);

CREATE TABLE notif_prefs (
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key     TEXT NOT NULL,
    email   BIGINT NOT NULL DEFAULT 1,
    push    BIGINT NOT NULL DEFAULT 1,
    PRIMARY KEY (user_id, key)
);

CREATE INDEX idx_reservations_days ON reservations(day_from, day_to);
CREATE INDEX idx_messages_thread ON messages(thread_id);
CREATE INDEX idx_notifications_user ON notifications(user_id, read);
CREATE INDEX idx_threads_link ON threads(kind, link_id);
