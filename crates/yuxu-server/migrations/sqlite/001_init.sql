-- Initial schema (SQLite)

CREATE TABLE IF NOT EXISTS users (
    id           TEXT PRIMARY KEY,
    username     TEXT NOT NULL UNIQUE,
    email        TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL DEFAULT '',
    avatar_url   TEXT NOT NULL DEFAULT '',
    bio          TEXT NOT NULL DEFAULT '',
    password_hash TEXT NOT NULL,
    is_admin     INTEGER NOT NULL DEFAULT 0,
    created_at   INTEGER NOT NULL,
    updated_at   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS repositories (
    id             TEXT PRIMARY KEY,
    owner_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name           TEXT NOT NULL,
    full_name      TEXT NOT NULL UNIQUE,
    description    TEXT NOT NULL DEFAULT '',
    is_private     INTEGER NOT NULL DEFAULT 0,
    default_branch TEXT NOT NULL DEFAULT 'main',
    created_at     INTEGER NOT NULL,
    updated_at     INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS repo_members (
    id            TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role          TEXT NOT NULL,
    joined_at     INTEGER NOT NULL,
    UNIQUE (repository_id, user_id)
);

CREATE TABLE IF NOT EXISTS labels (
    id            TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    color         TEXT NOT NULL DEFAULT '#888888',
    description   TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS issues (
    id            TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    number        INTEGER NOT NULL,
    title         TEXT NOT NULL,
    body          TEXT NOT NULL DEFAULT '',
    state         TEXT NOT NULL DEFAULT 'open',
    author_id     TEXT NOT NULL REFERENCES users(id),
    assignee_id   TEXT REFERENCES users(id),
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    closed_at     INTEGER,
    UNIQUE (repository_id, number)
);

CREATE TABLE IF NOT EXISTS merge_requests (
    id             TEXT PRIMARY KEY,
    repository_id  TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    number         INTEGER NOT NULL,
    title          TEXT NOT NULL,
    body           TEXT NOT NULL DEFAULT '',
    source_branch  TEXT NOT NULL,
    target_branch  TEXT NOT NULL,
    state          TEXT NOT NULL DEFAULT 'open',
    author_id      TEXT NOT NULL REFERENCES users(id),
    ci_status      TEXT NOT NULL DEFAULT 'pending',
    approval_count INTEGER NOT NULL DEFAULT 0,
    created_at     INTEGER NOT NULL,
    updated_at     INTEGER NOT NULL,
    merged_at      INTEGER,
    merged_by      TEXT REFERENCES users(id),
    UNIQUE (repository_id, number)
);

CREATE TABLE IF NOT EXISTS pipelines (
    id               TEXT PRIMARY KEY,
    repository_id    TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    merge_request_id TEXT REFERENCES merge_requests(id) ON DELETE SET NULL,
    branch           TEXT NOT NULL,
    commit_sha       TEXT NOT NULL,
    commit_message   TEXT NOT NULL DEFAULT '',
    triggered_by     TEXT NOT NULL,
    status           TEXT NOT NULL DEFAULT 'pending',
    duration_secs    INTEGER,
    created_at       INTEGER NOT NULL,
    started_at       INTEGER,
    finished_at      INTEGER
);

CREATE TABLE IF NOT EXISTS rooms (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    host_user_id  TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    live_kit_room TEXT NOT NULL,
    channel_id    TEXT,
    created_at    INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS projects (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    room_id          INTEGER NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    host_user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_ssh           INTEGER NOT NULL DEFAULT 0,
    created_at       INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS project_collaborators (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id    INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    replica_id    INTEGER NOT NULL,
    is_host       INTEGER NOT NULL DEFAULT 0,
    joined_at     INTEGER NOT NULL,
    UNIQUE (project_id, user_id)
);
