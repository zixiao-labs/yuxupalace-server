-- Initial schema (PostgreSQL)

CREATE TABLE IF NOT EXISTS users (
    id           TEXT PRIMARY KEY,
    username     TEXT NOT NULL UNIQUE,
    email        TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL DEFAULT '',
    avatar_url   TEXT NOT NULL DEFAULT '',
    bio          TEXT NOT NULL DEFAULT '',
    password_hash TEXT NOT NULL,
    is_admin     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at   BIGINT NOT NULL,
    updated_at   BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS repositories (
    id             TEXT PRIMARY KEY,
    owner_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name           TEXT NOT NULL,
    full_name      TEXT NOT NULL UNIQUE,
    description    TEXT NOT NULL DEFAULT '',
    is_private     BOOLEAN NOT NULL DEFAULT FALSE,
    default_branch TEXT NOT NULL DEFAULT 'main',
    created_at     BIGINT NOT NULL,
    updated_at     BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS repo_members (
    id            TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role          TEXT NOT NULL,
    joined_at     BIGINT NOT NULL,
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
    number        INT NOT NULL,
    title         TEXT NOT NULL,
    body          TEXT NOT NULL DEFAULT '',
    state         TEXT NOT NULL DEFAULT 'open',
    author_id     TEXT NOT NULL REFERENCES users(id),
    assignee_id   TEXT REFERENCES users(id),
    created_at    BIGINT NOT NULL,
    updated_at    BIGINT NOT NULL,
    closed_at     BIGINT,
    UNIQUE (repository_id, number)
);

CREATE TABLE IF NOT EXISTS merge_requests (
    id             TEXT PRIMARY KEY,
    repository_id  TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    number         INT NOT NULL,
    title          TEXT NOT NULL,
    body           TEXT NOT NULL DEFAULT '',
    source_branch  TEXT NOT NULL,
    target_branch  TEXT NOT NULL,
    state          TEXT NOT NULL DEFAULT 'open',
    author_id      TEXT NOT NULL REFERENCES users(id),
    ci_status      TEXT NOT NULL DEFAULT 'pending',
    approval_count INT NOT NULL DEFAULT 0,
    created_at     BIGINT NOT NULL,
    updated_at     BIGINT NOT NULL,
    merged_at      BIGINT,
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
    triggered_by     TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    status           TEXT NOT NULL DEFAULT 'pending',
    duration_secs    INT,
    created_at       BIGINT NOT NULL,
    started_at       BIGINT,
    finished_at      BIGINT
);

CREATE TABLE IF NOT EXISTS rooms (
    id            BIGSERIAL PRIMARY KEY,
    host_user_id  TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    live_kit_room TEXT NOT NULL,
    channel_id    TEXT,
    created_at    BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS projects (
    id               BIGSERIAL PRIMARY KEY,
    room_id          BIGINT NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    host_user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_ssh           BOOLEAN NOT NULL DEFAULT FALSE,
    created_at       BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS project_collaborators (
    id            BIGSERIAL PRIMARY KEY,
    project_id    BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    replica_id    INT NOT NULL,
    is_host       BOOLEAN NOT NULL DEFAULT FALSE,
    joined_at     BIGINT NOT NULL,
    UNIQUE (project_id, user_id)
);
