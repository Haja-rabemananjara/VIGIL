# VIGIL

VIGIL is a collaborative operational control room that handles both realities of production operations in real time: **Releases** (planned deployments, validated step by step) and **Incidents** (detected problems, triaged and resolved). The two are connected - a Release can automatically trigger an Incident, and an active Incident can block an ongoing Release.

> Epitech T-DEV-700 - Solo project - Hajatiana Rabemananjara

---

## Stack

| Component | Technology | Justification |
|-----------|-----------|---------------|
| Application server | **Rust (Axum)** | Imposed |
| Web client | **Next.js** (TypeScript) | Imposed |
| Desktop client | **Tauri** | Chosen - see below |
| Persistence | **PostgreSQL** | Chosen - see below |
| Real-time | **WebSockets** | Imposed |
| Containerization | **Docker Compose** | Imposed |


### Why Tauri over Electron

With Rust already chosen for the server, Tauri shares the same language for native/desktop code - no context switch, and architectural concepts (traits, ownership) carry over directly.


### Why PostgreSQL over SQLite

VIGIL's schema leans on three Postgres features that SQLite supports weakly or not at all:

- **Partial unique indexes** are the backbone of several core invariants - exactly one active Manager per team, one active ban per (team, user), one active assignee per incident, one active link per (release, incident). SQLite supports partial indexes, but Postgres's are more mature and battle-tested at this kind of constraint density.
- **JSONB** with native indexing and operators is used for `rules.trigger_filters`, `rules.reaction_payload`, `webhook_deliveries.payload`, and `audit_log.metadata`. SQLite's JSON support is function-based over TEXT, not a real binary type.
- **Concurrent writers**: SQLite serializes writes at the database level (single-writer), which doesn't fit a tool whose entire premise is multiple operators acting on the same incident/release simultaneously. Postgres handles concurrent writes with row-level locking.

PostgreSQL also fits the target deployment story naturally - `db` is one of the four services in the final `docker-compose.yml`, which is a more native shape for Postgres-as-a-service-container than for a SQLite file shared across containers.

---

## Architecture

```
                        ┌──────────────────────────────┐
                        │     External services          │
                        │  (GitHub, GitLab, webhooks)    │
                        └───────────────┬─────────────────┘
                                        │ POST /webhooks/{service}
                                        ▼
┌────────────────────────────────────────────────────────────────────┐
│                  VIGIL Application Server (Rust / Axum)              │
│                                                                       │
│   routes/   →   handlers/    →   services/    →   domain/           │
│  (wiring)      (parse/format)   (business logic)  (pure rules)      │
│                                        │                              │
│                                        ▼                              │
│                                    repo/ (sqlx)                       │
│                                        │                              │
│                                        ▼                              │
│                              PostgreSQL (19 tables)                  │
│                                                                       │
│   services/ also call  ──────────►  ws/broadcaster                   │
│                                      to_team() / to_user()           │
└──────────────────────────────────┬──────────────────────────────────┘
                                    │  REST (writes) + WebSocket (truth)
                     ┌──────────────┴───────────────┐
                     ▼                               ▼
           ┌───────────────────┐          ┌──────────────────────┐
           │    Web client        │          │    Desktop client       │
           │    Next.js, CSR       │          │    Tauri, standalone    │
           │    :8081               │          │    embeds the same      │
           │                        │          │    static export        │
           └───────────────────┘          └──────────────────────┘
```

**Core principle**: writes go up via REST, truth comes back down via WebSocket. A client never trusts its own HTTP response to update its UI - it waits for the broadcast, exactly like every other connected client. This guarantees all clients (web and desktop) converge on the same state.

### Codebase navigation

| Layer | Location | Responsibility |
|-------|----------|-----------------|
| Routes | `server/src/routes/` | HTTP route definitions - wiring only, no logic |
| Handlers | `server/src/handlers/` | Request extraction, calling a service, response formatting - no SQL, no business rules |
| Services | `server/src/services/` | Business logic - orchestrates repo calls + broadcaster calls + audit log |
| Domain | `server/src/domain/` | Pure types and rules (e.g. `can_transition`) - zero I/O, zero infrastructure dependencies |
| Repo | `server/src/repo/` | The only layer that touches SQL (via `sqlx`) |
| WebSocket | `server/src/ws/` | Broadcaster - transport only; services decide *what* and *to whom*, the broadcaster just delivers |

The crate is split into `lib.rs` (declares all modules, re-exports `AppError`/`AppState`) and a thin `main.rs` (wires config, DB pool, router, and calls `axum::serve`). This split exists so integration tests (`server/tests/`) can import the application as a library and spin up real instances against a disposable database, without duplicating the server bootstrap logic.

---

## Installation & Local Setup

### Prerequisites

- Rust (stable) - version pinned via `rust-toolchain.toml`
- Docker & Docker Compose
- `sqlx-cli`: `cargo install sqlx-cli --no-default-features --features postgres`
- Node.js 20+ (for the web client, once initialized)

### Quick start

```bash

# 1. Start the database (Postgres + Adminer)
docker compose -f docker-compose.dev.yml up -d
docker compose -f docker-compose.dev.yml ps   # wait for "healthy"

# 2. Run migrations
cd server
export DATABASE_URL=postgres://vigil:vigil_dev@localhost:5432/vigil
sqlx migrate run

# 3. Build and run the server
cargo run
```

The server listens on `http://localhost:8080`. Verify it's alive:

```bash
curl http://localhost:8080/health
# { "status": "ok", "version": "0.1.0" }
```

Adminer (database inspection UI) is available at `http://localhost:8888` - system: PostgreSQL, server: `db`, user/password/database from `.env`.

### Running tests

```bash
cd server
cargo test
```

Each test spins up its own disposable database (via `spawn_app()` in `tests/common/mod.rs`), runs all migrations against it, and tears it down afterward - full isolation, safe to run in parallel.

### Linting & formatting

```bash
cargo fmt --check
cargo clippy
```

---

## REST API

> This section grows with each ticket. Current state below; see the backlog for the full planned surface.

### Implemented

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/health` | none | Liveness check - returns `{ status, version }` |

### Planned (next tickets)

| Method | Path | Auth |
|--------|------|------|
| POST | `/auth/signup` | none |
| POST | `/auth/signin` | none |
| GET | `/me` | session |
| POST | `/auth/signout` | session |

All error responses share a uniform shape, regardless of endpoint:

```json
{ "error": { "code": "VALIDATION_ERROR", "message": "Password must be at least 8 characters" } }
```

| HTTP Status | `code` | Meaning |
|-------------|--------|---------|
| 401 | `UNAUTHORIZED` | Missing or invalid session token |
| 403 | `FORBIDDEN` | Authenticated but not permitted |
| 404 | `NOT_FOUND` | Resource doesn't exist (or access is hidden as not-found) |
| 409 | `CONFLICT` | Unique constraint violation (e.g. duplicate email) |
| 422 | `VALIDATION_ERROR` | Input failed validation |
| 500 | `INTERNAL_ERROR` | Unexpected server error (details logged server-side only) |

---

## Database Schema

Full commented DDL lives in [`server/migrations/001_initial_schema.sql`](./server/migrations/001_initial_schema.sql); the DBML diagram lives in [`docs/`](./docs/). Nineteen tables, grouped into seven functional blocks:

**Core identity** - `users`, `sessions`, `teams`, `team_members`
The authentication and workspace root. `team_members` carries the 3-role system (`observer`/`responder`/`manager`) with a partial unique index guaranteeing exactly one active Manager per team.

**Membership lifecycle** - `invitations`, `team_bans`
Closes the loop on how someone enters or is barred from a team. `invitations.code` is globally unique (resolved without a `team_id` at join time); `team_bans` uses a partial unique index per `(team_id, user_id)` so a lifted ban can later be re-applied without a schema conflict.

**Incidents** - `incidents`, `incident_assignments`, `timeline_entries`, `timeline_reactions`
`incidents` holds current state (status + severity as independent axes); `timeline_entries` is the append-only event log of what happened. `incident_assignments` and `timeline_reactions` each enforce their own invariant via a partial/composite unique index (one active assignee; one reaction per user/entry/emoji).

**Releases** - `releases`, `release_steps`, `release_incident_links`
Mirrors the incidents block structurally (state + transition timestamps), but models a planned, sequential process instead of a reactive one. `release_incident_links` is the N-N table whose presence drives the automatic `blocked` state.

**Social** - `private_messages`
Strictly bilateral, never grouped. No `team_id` - access is checked at send time via a shared-team query, not stored as a property of the message.

**Automation** - `service_connections`, `rules`, `rule_executions`, `webhook_deliveries`
The Action → REAction pipeline. `service_connections` stores AES-256-GCM–encrypted tokens (reversible, unlike session hashing, because the server must reuse them to call external APIs). `rules` separates filterable columns (service/event) from free-form JSONB (filters/payload templates). `webhook_deliveries` and `rule_executions` are append-only logs of what arrived and what happened.

**Audit** - `audit_log`
A decoupled observer: no foreign keys to any other table, so it survives deletions elsewhere and never blocks them. Append-only by design - moderation and configuration changes never rewrite history.

### Conventions applied throughout

- UUID v4 primary keys, generated server-side
- `TIMESTAMPTZ` for every timestamp; `DEFAULT now()` on every `created_at`
- Enums are `TEXT` + `CHECK`, never native Postgres enum types (cheaper to extend without a migration touching existing rows)
- History is preserved via a `status` column, never via `DELETE` - the only tables that use real `DELETE` are `timeline_reactions` (toggling a reaction) and `sessions` (sign out)
- Foreign keys to `users` never cascade (users are never deleted); foreign keys to `teams` cascade (deleting a team legitimately removes its data)

---

## WebSocket Events

Full specification - connection handshake, envelope format, delivery modes, reconnection strategy, and the complete event catalog - lives in [WEBSOCKET_SPEC.md](./WEBSOCKET_SPEC.md).

---

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | - |
| `SERVER_HOST` | Bind address | `0.0.0.0` |
| `SERVER_PORT` | Bind port | `8080` |
| `POSTGRES_USER` / `POSTGRES_PASSWORD` / `POSTGRES_DB` | Used by `docker-compose.dev.yml` to provision the dev database | `vigil` / `vigil_dev` / `vigil` |
| `SESSION_DURATION_DAYS` | Session token lifetime | `30` |
| `MASTER_KEY` | AES-256-GCM key for encrypting service tokens | - |

---

## Design Decisions

A few cross-cutting decisions worth calling out explicitly, beyond the per-table rationale above:

- **Uniform error shape.** Every error response - across every endpoint - has the same `{ error: { code, message } }` JSON body. This is implemented once, in `AppError`'s `IntoResponse` impl, so handlers never hand-roll error formatting.
- **`lib.rs` / `main.rs` split.** The application logic lives in a library crate; `main.rs` is a thin binary entry point. This is what makes `tests/` able to spin up a real, fully-wired server instance per test without duplicating bootstrap code.
- **Session tokens are hashed (SHA-256); service tokens are encrypted (AES-256-GCM).** Different threat models: a session token only needs to be *verified* (hash comparison is enough, and irreversible by design), while a service token (GitHub, Discord) must be *reused* to call the external API, so it must be decryptable.
- **WebSocket broadcaster is transport-only.** It exposes `to_team(team_id, event)` and `to_user(user_id, event)`; only services call it, never handlers. Adding a new event type is a new enum variant in `WsEvent` plus a call site in a service - the broadcaster itself never changes.

---

## Target OS

**Linux** - desktop binary delivered as `.AppImage`.

---

## Private Messages

- Maximum message length: **2000 characters** (enforced server-side)

## Reactions

- Available emojis: `+1`, `-1`, `eyes`, `warning`, `check`, `fire`

---

## License

Author: Hajatiana Rabemananjara.