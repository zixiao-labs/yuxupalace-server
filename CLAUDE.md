# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Workspace layout

Cargo workspace (edition 2024) with four crates under `crates/`:

- `raidian` — shared protobuf types. Two packages: `raidian` (business API: auth, repo, issue, MR, pipeline, member, dashboard) and `raidian.collab` (Zed-style collab: envelope, session, room, project, worktree, buffer, collaborator, lsp, git). Generated files (`src/generated.rs`, `src/generated_collab.rs`) are committed so downstream consumers (Zed/Logos/CLI) don't need `protoc`.
- `yuxu-core` — shared domain logic: JWT/argon2 auth, ACL roles, and a **hand-written Zed-style CRDT engine** in `src/crdt/` (Lamport clock + VersionVector + Fragment list + UndoMap + deferred ops). Note: the top-level README mentions `yrs`, but the actual engine is hand-rolled — do not pull in `yrs`.
- `yuxu-server` — Axum HTTP + WebSocket backend. Router wiring lives in `src/routes/mod.rs` (REST) plus `/rpc` (collab WS) and `/health` in `src/main.rs`. Note: README says the WS endpoint is `/api/v1/collab/ws`; the code wires it at `/rpc`.
- `yuxu-cli` — clap-based CLI, binary name `yuxu` (not `yuxu-cli`).

## Build / test / lint

CI runs with `RUSTFLAGS=-D warnings`, so any warning breaks the build locally too once that is exported. Match CI exactly:

```bash
# Format check
cargo fmt --all -- --check

# Clippy (yuxu-server is feature-gated — run both)
cargo clippy -p yuxu-server --no-default-features --features sqlite   -- -D warnings
cargo clippy -p yuxu-server --no-default-features --features postgres -- -D warnings
cargo clippy -p raidian -p yuxu-core -p yuxu-cli -- -D warnings

# Tests
cargo test -p yuxu-core --all-targets
cargo test -p yuxu-server --no-default-features --features sqlite   --all-targets
cargo test -p yuxu-server --no-default-features --features postgres --all-targets
```

## Feature flags: sqlite vs postgres

`yuxu-server` has mutually-exclusive backend features. `sqlite` is the default, `postgres` is opt-in.

- `src/db/mod.rs` picks `DbPool` (`SqlitePool` or `PgPool`) at compile time via `#[cfg]`.
- `sqlx::migrate!` points at `migrations/sqlite/` or `migrations/postgres/`. Keep the two migration trees in sync when adding schema.
- To build/run the Postgres variant, always pass `--no-default-features --features postgres`; otherwise `sqlite` leaks in and the `#[cfg]` picks the wrong pool type.

## Protobuf regeneration

`.proto` edits under `crates/raidian/proto/` require regenerating the committed Rust:

```bash
bash crates/raidian/generate.sh   # requires `protoc` on PATH
```

The script produces `crates/raidian/src/generated.rs` and `generated_collab.rs`. Both are committed — don't gitignore them; downstream consumers depend on the crate without a build script.

## Server runtime config

`yuxu-server` reads env vars in `src/config.rs` (these are the truth — the README's `JWT_SECRET`/`HOST`/`PORT`/`GIT_ROOT` names are stale):

| Env var | Required | Notes |
|---|---|---|
| `YUXU_BIND` | no (defaults `0.0.0.0:8080`) | `SocketAddr` |
| `DATABASE_URL` | no | defaults to `sqlite://yuxu.db?mode=rwc` or `postgres://...` depending on feature |
| `YUXU_JWT_SECRET` | **yes** | must be ≥ 32 bytes, or set `YUXU_DEV_MODE=1` to auto-generate an ephemeral one (logs a warning; tokens don't survive restart) |
| `YUXU_JWT_TTL_SECS` | no (default 3600) | positive integer |
| `YUXU_LIVEKIT_URL` | no | empty string if unset |

## Collab hub architecture

`yuxu-server/src/collab/hub.rs` is an in-memory registry (`DashMap`) holding `connections`, `rooms`, `projects`, and user↔connection mappings. All collab state is ephemeral — nothing is persisted to the DB. The `/rpc` WebSocket handler in `collab/ws.rs` speaks the binary envelope protocol from `raidian.collab` (not the legacy byte-prefixed format the README describes).

Disconnect cleanup is subtle: `CollabHub::deregister` returns `DisconnectEffects` listing per-project guest lists to notify. Callers are expected to broadcast `UnshareProject` only to each project's own remaining guests — don't broadcast globally (see recent commits on guest-scoping fixes).
