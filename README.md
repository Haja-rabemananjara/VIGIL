# VIGIL

VIGIL is a collaborative operational control room that handles both realities of production operations in real time: **Releases** (planned deployments, validated step by step) and **Incidents** (detected problems, triaged and resolved). The two are connected - a Release can automatically trigger an Incident, and an active Incident can block an ongoing Release.

> Hajatiana Rabemananjara

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
│   routes/   =>   handlers/    =>   services/    =>   domain/           │
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

### Stack & structure

The project is a monorepo with two top-level directories: `server/` (Rust + Axum)
and `client/` (Next.js). The client folder hosts both the web and desktop targets:

- The Next.js codebase under `client/src/` is the **single source of truth** for the UI.
- It is built with `output: 'export'` (static export, CSR only, no Next API routes
  or server features).
- The desktop application uses Tauri, located at `client/src-tauri/` (sibling of
  `client/src/`). It embeds the statically-exported Next.js output, ensuring
  feature parity between web and desktop by construction.

This avoids any code duplication: a feature written once works on both targets.

### Frontend conventions

- **Next.js App Router** is used (not Pages Router). Routes live under `client/src/app/`.
- **TypeScript** strict mode.
- **Tailwind CSS** for styling, with design tokens documented in `UI_GUIDELINES.md`.
- **shadcn/ui** for accessible base components. Components are copied into
  `client/src/components/ui/` via the shadcn CLI and may be customized locally.
- **No hardcoded user-facing strings** : every visible label goes through a `t()`
  function from `client/src/lib/i18n.ts`. This makes the FR/EN dictionary swap
  in Phase 2 a one-file change instead of a screen-by-screen rewrite.
- **Native capabilities behind a `platform/` layer** (`client/src/lib/platform.ts`).
  Components never call Tauri APIs directly. The web build uses browser fallbacks
  (or no-ops); the Tauri build will swap implementations without touching components.

### State management

- **React Context** for the auth store (current user, token). It is small, scoped,
  and changes rarely.
- **Zustand** for richer client state as the project grows (active team selection,
  WS connection status, etc.). Introduced incrementally only where Context becomes
  cumbersome.
- **TanStack Query** is reserved for later (incidents lists, paginated timelines)
  when caching and revalidation become valuable.

### Authentication

- Opaque session tokens (32 random bytes, hex-encoded over the wire).
- Server stores SHA-256 of the token as `BYTEA` (irreversible verification).
- Passwords hashed with **Argon2** (PHC string, stored as `TEXT`).
- Token sent on every authenticated request via `Authorization: Bearer <token>`.
- **Client-side storage in `localStorage`** for VIGIL's scope. This is a deliberate
  trade-off: simpler than HttpOnly cookies (which would require server-side CORS
  credentials changes), at the cost of XSS exposure. Acceptable for an academic
  project, would be reconsidered in a production setting.

### UUID generation

All UUIDs are generated in Rust (`Uuid::new_v4()`) before INSERT, never via
`DEFAULT gen_random_uuid()` in the database. This keeps ID generation independent
from the database engine and lets the application layer know the entity ID before
persistence : useful for logging, event broadcasting, and tracing.

### Naming conventions

Each language follows its idiomatic convention:
- **Rust**: snake_case (functions/vars), PascalCase (types), enforced by `cargo fmt` + clippy.
- **TypeScript**: camelCase (functions/vars), PascalCase (types/components), enforced by ESLint + Prettier.

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

## Desktop application

The desktop client is a Tauri v2 application targeting **Linux/AppImage**
(Ubuntu 24.04 tested). It exposes exactly the same features as the web
client, plus tray icon and OS notifications.

### Standalone by construction

The AppImage is standalone: no Node runtime, no external server needed
at launch. The static Next.js export (`out/`) is embedded in the binary
and served internally by an embedded HTTP server (`tiny_http`) on
`http://localhost:9527`.

Rationale: Next.js static export produces one HTML per dynamic route
(`/teams/placeholder`, not `/teams/<uuid>`). A naive file server would
return 404 on real UUIDs. The embedded server rewrites any UUID-like
path segment to `placeholder` while preserving the real URL, so
client-side routing reads the actual identifier via a small
`useRouteParams()` hook (`client/src/lib/useRouteParams.ts`) rather
than `useParams()` (which would be frozen to `placeholder` in the RSC
payload at build time).

### Tray icon and background lifecycle

Closing the window does **not** terminate the app. The window is
hidden, the WebSocket stays connected, and the app remains
represented by a tray icon (top-right on GNOME). The tray menu
exposes `Open` (restore window) and `Quit` (real exit).

Requires the `AppIndicator and KStatusNotifierItem` GNOME extension
(preinstalled on Ubuntu 24.04).

### Native OS notifications

Three triggers, all fired from a single central hook
(`client/src/lib/useNotifications.ts`) reacting to WebSocket events:

- Incident assigned to the current user (`incident_assigned`)
- Incident escalated to critical (`incident_escalated`)
- Release blocked by a linked incident (`release_state_changed`)

Notifications are dispatched through the browser `Notification` API
from within the WebKitGTK webview. This works uniformly on web and
desktop through the `notify()` abstraction in
`client/src/lib/platform.ts`.

**Known limitation.** On GNOME 46+, notifications emitted from a
webview (whether via the Tauri notification plugin, a custom Rust
command calling `notify-send`, or the browser API) may not display
even though the API returns success. This is a documented environment
bug (see tauri-apps/tauri#14095 and
tauri-apps/plugins-workspace#2566), independent of application code,
verified by logs showing `permission: granted` and successful emission
on every trigger. On GNOME versions prior to 46 or on non-GNOME
desktops, notifications display correctly.

### Building the AppImage

```bash
cd client
npx tauri build --bundles appimage
```

Output: `client/src-tauri/target/release/bundle/appimage/*.AppImage`.

### Installing and running

Requires `libfuse2t64` on the host to mount the AppImage:

```bash
sudo apt install libfuse2t64
chmod +x vigil-desktop_*.AppImage
./vigil-desktop_*.AppImage
```

To pin VIGIL to the GNOME menu, create
`~/.local/share/applications/com.vigil.desktop.desktop`:

```
[Desktop Entry]
Type=Application
Name=VIGIL
Exec=/absolute/path/to/vigil-desktop.AppImage
Icon=vigil-desktop
Terminal=false
Categories=Development;
StartupWMClass=VIGIL
```

---

## Running with Docker

The full stack runs via Docker Compose with four services:

| Service          | Role                                      | Port |
|------------------|-------------------------------------------|------|
| `db`             | PostgreSQL 16                             | -    |
| `server`         | Rust/Axum API + WebSocket                 | 8080 |
| `client_web`     | Nginx serving the Next.js static export   | 8081 |
| `client_desktop` | Builds the AppImage into a shared volume  | -    |

`client_web` depends on `client_desktop` completing successfully; the
built AppImage is exposed for download at
`http://localhost:8081/client.AppImage`.

Two compose files are provided:

- `docker-compose.dev.yml`: db + adminer only, for local development
  with `cargo run` and `npm run dev` on the host.
- `docker-compose.yml`: the full production-like stack.

### Launch the full stack

Copy `.env.example` to `.env` and fill in the sensitive values. The
`MASTER_KEY_HEX` must be 64 hex characters, generated with:

```bash
openssl rand -hex 32
```

Then:

```bash
docker compose up --build
```

First build takes several minutes (Rust compilation + Next build +
AppImage bundling). Subsequent builds use the Docker layer cache.

- Web client: `http://localhost:8081`
- API: `http://localhost:8080`
- Desktop binary: `http://localhost:8081/client.AppImage`

### Development compose

For hot-reload development, only launch the database:

```bash
docker compose -f docker-compose.dev.yml up -d
```

Then run the server and client locally:

```bash
cd server && cargo run
cd client && npm run dev
```

Adminer is exposed on `http://localhost:8888`.

---

## REST API

All endpoints are documented below by functional domain. Error responses share a uniform shape regardless of endpoint:

```json
{ "error": { "code": "VALIDATION_ERROR", "message": "..." } }
```

| HTTP Status | `code` | Meaning |
|-------------|--------|---------|
| 401 | `UNAUTHORIZED` | Missing or invalid session token |
| 403 | `FORBIDDEN` | Authenticated but not permitted |
| 404 | `NOT_FOUND` | Resource doesn't exist (or hidden as not-found) |
| 409 | `CONFLICT` | Unique constraint violation |
| 422 | `VALIDATION_ERROR` | Input failed validation |
| 500 | `INTERNAL_ERROR` | Unexpected server error |

### Health

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/health` | none | `{ "status": "ok", "version": "0.1.0" }` |

### Authentication

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/auth/signup` | none | Create account. Body: `{ email, password, display_name }`. Returns user (201). Errors: 422 (validation), 409 (email taken) |
| POST | `/auth/signin` | none | Returns `{ token, user }` (200). Error: 401 |
| GET | `/me` | session | Current user info. Never exposes the password hash |
| POST | `/auth/signout` | session | Deletes the session. 204, token inoperative immediately |

### Teams

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/teams` | session | Create a team. Creator becomes Manager |
| GET | `/teams` | session | List teams the user belongs to |
| GET | `/teams/{team_id}` | member | Team detail. Non-member gets 404 (not 403) |
| GET | `/teams/{team_id}/members` | member | Member list with roles |
| PATCH | `/teams/{team_id}/members/{user_id}/role` | Manager | Promote/demote Observer/Responder |
| POST | `/teams/{team_id}/transfer-manager` | Manager | Body: `{ target_user_id }`. Atomic swap, former Manager becomes Responder |
| POST | `/teams/{team_id}/leave` | member | Manager without transfer gets 409 |
| POST | `/teams/{team_id}/invitations` | Manager | Returns `{ code }` |
| POST | `/teams/join` | session | Body: `{ code }`. Joins as Observer. Banned user gets 403 |

### Incidents

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/teams/{team_id}/incidents` | Manager | Body: `{ title, body, severity }`. Initial state `open` |
| GET | `/teams/{team_id}/incidents` | member | Filterable by `?status=` and `?severity=`. Returns `{ "incidents": [...] }` |
| GET | `/teams/{team_id}/incidents/{id}` | member | Returns `{ "incident": {...} }` |
| PATCH | `/teams/{team_id}/incidents/{id}/status` | Responder+ | Body: `{ status, severity? }`. State machine enforced |
| PATCH | `/teams/{team_id}/incidents/{id}/severity` | Responder+ | Body: `{ severity }` |
| POST | `/teams/{team_id}/incidents/{id}/assign` | Manager | Body: `{ user_id }`. Target must be Responder+ |
| POST | `/teams/{team_id}/incidents/{id}/timeline` | Responder+ | Body: `{ content }`. Max 2000 chars |
| GET | `/teams/{team_id}/incidents/{id}/timeline` | member | Paginated, chronological |

### Releases

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/teams/{team_id}/releases` | Manager | Body: `{ title, body, steps: ["build","staging",...] }` |
| GET | `/teams/{team_id}/releases` | member | Filterable by `?status=`. Returns `{ "releases": [...] }` |
| GET | `/teams/{team_id}/releases/{id}` | member | Includes steps with `validated_at`/`validated_by`, linked incidents, progress |
| POST | `/teams/{team_id}/releases/{id}/start` | Manager | Transitions to `in_progress` |
| POST | `/teams/{team_id}/releases/{id}/cancel` | Manager | Allowed from `created`, `in_progress`, `blocked` |
| POST | `/teams/{team_id}/releases/{id}/steps/{step_id}/validate` | Responder+ | Strict sequential order enforced. Blocked release gets 409 |
| POST | `/teams/{team_id}/releases/{id}/link` | Manager | Body: `{ incident_id }`. Auto-blocks if release is `in_progress` |
| POST | `/teams/{team_id}/releases/{id}/unlink` | Manager | Body: `{ incident_id }`. Auto-unblocks when no active linked incidents remain |

### Webhooks

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/webhooks/github` | HMAC | GitHub webhook receiver. Validates `X-Hub-Signature-256` (HMAC-SHA256, constant-time). Persists the raw payload to `webhook_deliveries` before processing. Returns 202 immediately; rule evaluation runs async. Invalid/missing signature returns 401. |

The HMAC secret is the `WEBHOOK_SECRET` environment variable. This is a global secret shared between VIGIL and GitHub, not a per-user token.

### Rules

All rule endpoints are team-scoped. Only Managers can create, update, or delete rules. Observers and Responders can list and read them.

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/teams/{team_id}/rules` | member | Returns `[...]` (bare array) |
| POST | `/teams/{team_id}/rules` | Manager | See body format below. Returns 201 + rule |
| GET | `/teams/{team_id}/rules/{id}` | member | Single rule |
| PATCH | `/teams/{team_id}/rules/{id}` | Manager | Partial update. Any subset of fields |
| DELETE | `/teams/{team_id}/rules/{id}` | Manager | 204 |

**Create/update body:**

```json
{
  "name": "CI failure -> critical incident",
  "enabled": true,
  "trigger": {
    "service": "github",
    "event": "workflow_run",
    "filters": {
      "workflow_run.conclusion": "failure"
    }
  },
  "reaction": {
    "type": "vigil_create_incident",
    "payload": {
      "title": "CI broken on {{repository.name}}",
      "severity": "high",
      "body": "Workflow {{workflow_run.name}} failed"
    }
  }
}
```

**Validation at creation:**

- `trigger.service` + `trigger.event` must exist in the `ActionCatalog`
  (the same catalog that feeds `/about.json`). Unknown trigger returns 422.
- `reaction.type` must exist in the `ReactionRegistry`. Unknown reaction
  returns 422.
- Both validations read from the same registries the engine uses at
  runtime, so a rule that passes validation can always be evaluated.

### Service Connections

Per-user encrypted storage of third-party tokens and webhook URLs.

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/me/services` | session | Returns `[{ id, service, created_at, updated_at }]`. Never exposes the token |
| POST | `/me/services/{service}` | session | Body: `{ "token": "..." }`. Encrypted AES-256-GCM at rest. Upsert semantics: reconnecting overwrites the previous token. `{service}` must match the DB CHECK constraint (`github`, `gitlab`, `discord`). Unknown service returns 404, empty token returns 422 |
| DELETE | `/me/services/{service}` | session | 204. Deletes the encrypted token |

### Discovery

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/about.json` | none | Dynamic catalog of Actions, Reactions, and the kickoff token. See the dedicated section above |

---

## Database Schema

Full commented DDL lives in [`server/migrations/20260616162323_initial_schema.sql`](./server/migrations/20260616162323_initial_schema.sql); the DBML diagram lives in [`docs/`](./docs/). Nineteen tables, grouped into seven functional blocks:

**Core identity** : `users`, `sessions`, `teams`, `team_members`
The authentication and workspace root. `team_members` carries the 3-role system (`observer`/`responder`/`manager`) with a partial unique index guaranteeing exactly one active Manager per team.

**Membership lifecycle** : `invitations`, `team_bans`
Closes the loop on how someone enters or is barred from a team. `invitations.code` is globally unique (resolved without a `team_id` at join time); `team_bans` uses a partial unique index per `(team_id, user_id)` so a lifted ban can later be re-applied without a schema conflict.

**Incidents** : `incidents`, `incident_assignments`, `timeline_entries`, `timeline_reactions`
`incidents` holds current state (status + severity as independent axes); `timeline_entries` is the append-only event log of what happened. `incident_assignments` and `timeline_reactions` each enforce their own invariant via a partial/composite unique index (one active assignee; one reaction per user/entry/emoji).

**Releases** : `releases`, `release_steps`, `release_incident_links`
Mirrors the incidents block structurally (state + transition timestamps), but models a planned, sequential process instead of a reactive one. `release_incident_links` is the N-N table whose presence drives the automatic `blocked` state.

**Social** : `private_messages`
Strictly bilateral, never grouped. No `team_id` : access is checked at send time via a shared-team query, not stored as a property of the message.

**Automation** : `service_connections`, `rules`, `rule_executions`, `webhook_deliveries`
The Action => REAction pipeline. `service_connections` stores AES-256-GCM–encrypted tokens (reversible, unlike session hashing, because the server must reuse them to call external APIs). `rules` separates filterable columns (service/event) from free-form JSONB (filters/payload templates). `webhook_deliveries` and `rule_executions` are append-only logs of what arrived and what happened.

**Audit** : `audit_log`
A decoupled observer: no foreign keys to any other table, so it survives deletions elsewhere and never blocks them. Append-only by design - moderation and configuration changes never rewrite history.

### Conventions applied throughout

- UUID v4 primary keys, generated server-side
- `TIMESTAMPTZ` for every timestamp; `DEFAULT now()` on every `created_at`
- Enums are `TEXT` + `CHECK`, never native Postgres enum types (cheaper to extend without a migration touching existing rows)
- History is preserved via a `status` column, never via `DELETE` - the only tables that use real `DELETE` are `timeline_reactions` (toggling a reaction) and `sessions` (sign out)
- Foreign keys to `users` never cascade (users are never deleted); foreign keys to `teams` cascade (deleting a team legitimately removes its data)

---

## Rule Engine

VIGIL turns external events into VIGIL actions through a rule engine built on two extension seams: a **catalog of Actions** (what services can send us) and a **registry of Reactions** (what we can do in response).
A rule wires one Action to one Reaction, optionally filtered and templated.

### Pipeline

External service VIGIL server
───────────────── ────────────
GitHub CI fails ┌─ HMAC-SHA256 (constant time)
│ │ invalid => 401
│ POST /webhooks/github │
└───────────────────────────────────►┤ persist raw payload
│ (webhook_deliveries, replayable)
│
│ respond 202 immediately
│ ┌───────────────────┐
│ │ tokio::spawn │
│ │ │
│ │ load enabled rules │
│ │ matching event │
│ │ │ │
│ │ ▼ │
│ │ matcher │
│ │ (dot-notation, │
│ │ AND, strict eq) │
│ │ │ │
│ │ ▼ │
│ │ templating │
│ │ ({{path.to.field}}) │
│ │ │ │
│ │ ▼ │
│ │ Reaction.execute() │
│ │ (via dyn trait) │
│ │ │ │
│ │ ▼ │
│ │ log to │
│ │ rule_executions │
│ │ + broadcast │
│ │ rule_triggered │
│ │ or rule_failed │
│ └───────────────────┘
└────

The webhook receiver responds `202 Accepted` before the engine runs, so a slow reaction (Discord API, blocked release) never delays GitHub. A failure in one rule never affects the others: each is executed in isolation and reported as `rule_failed` on the WS channel.

### Catalog vs registry

| | ActionCatalog | ReactionRegistry |
|---|---|---|
| What it holds | Metadata (service, event, description) | Types implementing `ReactionExecutor` |
| Where declared | `main.rs` builder | `main.rs` builder |
| Exposed via | `/about.json` (actions[]) | `/about.json` (reactions[]) |
| Runtime role | Filter incoming webhooks | Dispatch reactions by kind |

**Asymmetry rationale.** Reactions have runtime behavior (`execute()`), so a trait + `Arc<dyn ReactionExecutor>` earns its complexity. Actions are declarative. They describe events a service can send us; the matching and dispatching happens against fields inside the payload, not against the metadata.
Building a symmetric `ActionExecutor` trait would be complexity without a purpose. Both extension points still cost a single line in `main.rs` for a new entry.

### Filters

Filters are dot-notation paths matched against the incoming payload:

```json
{
  "workflow_run.conclusion": "failure",
  "repository.full_name": "hajatiana/vigil"
}
```

- All paths must match (implicit AND)
- Comparison is strict equality on JSON values (string, number, bool)
- Missing paths never match
- `{}` matches everything. This is the safe default in the rule form

### Templating

Reaction payloads may embed `{{path.to.field}}` placeholders resolved
against the webhook payload:

```json
{
  "title": "CI broken on {{repository.name}}",
  "body":  "Workflow {{workflow_run.name}} failed"
}
```

**Unresolved placeholders are left literal.** If the template says `{{workflow.name}}` but the payload only has `workflow_run.name`, the output contains the literal string `{{workflow.name}}` rather than an empty string.
This is deliberate: it surfaces the mistake at the first run instead of producing silently degraded messages.

### Registered reactions

| Kind | Service | Effect |
|---|---|---|
| `vigil_create_incident` | vigil | Creates an incident on the rule's team |
| `vigil_escalate_incident` | vigil | Transitions an existing incident to `escalated` |
| `vigil_block_release` | vigil | Links an incident to a release, triggering the auto-block |
| `vigil_validate_release_step` | vigil | Advances a release step |
| `discord_message` | discord | Posts a message to a Discord webhook URL |

Reactions triggered by a rule are attributed in the audit log to
`rule.created_by`, not to a system user. This keeps the actor chain
honest — every action in VIGIL is traceable to a real user.

### Registered actions

Currently: `github/workflow_run`, `github/push`, `github/pull_request`.
Only `workflow_run` is wired end-to-end in the demo scenario; `push` and
`pull_request` are registered in the catalog so rules can target them,
but they carry no VIGIL-specific processing beyond generic dispatch.

### Service connections

Third-party services are connected per-user via
`POST /me/services/{service}` with a token or webhook URL body. Tokens
are encrypted at rest with AES-256-GCM (see `MASTER_KEY_HEX`) and
decrypted just-in-time inside a reaction (`DiscordMessage` reads the
rule creator's Discord webhook URL). They are never logged, never
returned in a response.

Connectable services are the intersection of what the DB `CHECK`
constraint allows and what the front discovers via `/about.json`
(`server.services[].connectable`). VIGIL itself appears in the catalog
(it exposes reactions) but is marked `connectable: false` — it's the
application, not a third party.

**Known limitation.** GitHub tokens are stored encrypted but not yet
consumed at runtime: the webhook receiver authenticates GitHub payloads
via a global `WEBHOOK_SECRET` (HMAC), not via per-user tokens. Reading
the token would require a reaction that calls the GitHub API on the
user's behalf (e.g. `github_create_issue`). This is on the extended
scope backlog.

---

## `/about.json`

Public discovery endpoint. Serves the catalog of Actions and Reactions so that clients build their UI without hard-coding any service name, event, or reaction kind.

**GET /about.json** (no authentification)

Response:

```json
{
  "client": {
    "host": "10.0.0.1"
  },
  "server": {
    "current_time": 1718000000,
    "token": "3f2a9b...e4c1",
    "services": [
      {
        "name": "github",
        "connectable": true,
        "actions": [
          {
            "name": "workflow_run",
            "description": "A CI workflow run has completed (success or failure)"
          }
        ],
        "reactions": []
      },
      {
        "name": "vigil",
        "connectable": false,
        "actions": [],
        "reactions": [
          {
            "name": "vigil_create_incident",
            "description": "Create a VIGIL incident with configurable title, severity, and body",
            "payload_example": "{\n  \"title\": \"CI broken on {{repository.name}}\",\n  \"severity\": \"high\"\n}"
          }
        ]
      },
      {
        "name": "discord",
        "connectable": true,
        "actions": [],
        "reactions": [
          {
            "name": "discord_message",
            "description": "Post a message to a Discord channel via webhook",
            "payload_example": "{\n  \"content\": \"CI broken on {{repository.name}}\",\n  \"username\": \"VIGIL\"\n}"
          }
        ]
      }
    ]
  }
}
```

### Fields

- **`client.host`** : the requesting client's IP, read from the TCP layer via Axum's `ConnectInfo<SocketAddr>`.
- **`server.current_time`** : Unix seconds, computed on each request.
- **`server.token`** : SHA-256 of `STUDENT_FIRSTNAME + STUDENT_LOGIN + "VIGIL2026"`, computed once at startup. This is the kickoff token required by the subject.
- **`server.services[].connectable`** : whether a user can attach a personal token or webhook URL to this service. Derived from the enum  that mirrors the DB `CHECK` constraint on `service_connections.service`, so this field and the connection endpoints share a single source of truth.
- **`server.services[].actions[]`** : events the service can send us. Sourced from the `ActionCatalog` built at startup.
- **`server.services[].reactions[]`** : what we can do in response. Sourced from the `ReactionRegistry` by iterating registered executors and grouping by their `service_name()`.
- **`payload_example`** : present on reactions only. Ships a well-formed example of the JSON payload a reaction expects, used by the rule form to prefill the payload textarea. Adding a new reaction automatically enriches this endpoint. No manual JSON update.

### What this makes possible

The rule form in the web client is built entirely from this endpoint: service selects, event selects, reaction selects, prefilled payload textareas. The client contains no hard-coded service or reaction name.
Adding a new reaction on the backend adds it to the form on the next page load, with its description and example.

---

## WebSocket Events

Full specification - connection handshake, envelope format, delivery modes, reconnection strategy, and the complete event catalog - lives in [WEBSOCKET_SPEC.md](./WEBSOCKET_SPEC.md).

---

## Configuration

| Variable            | Required | Description                                              |
|---------------------|----------|----------------------------------------------------------|
| `DATABASE_URL`      | yes      | PostgreSQL connection string                             |
| `SERVER_HOST`       | no       | Default `0.0.0.0`                                        |
| `SERVER_PORT`       | no       | Default `8080`                                           |
| `WEBHOOK_SECRET`    | no       | HMAC secret for `POST /webhooks/*` (default: dev secret) |
| `MASTER_KEY_HEX`    | yes      | 64 hex chars (32 bytes) for AES-256-GCM token encryption |
| `STUDENT_FIRSTNAME` | yes      | Used to derive the `/about.json` kickoff token           |
| `STUDENT_LOGIN`     | yes      | Used to derive the `/about.json` kickoff token           |

---

## Design Decisions

A few cross-cutting decisions worth calling out explicitly, beyond the per-table rationale above:

- **Uniform error shape.** Every error response - across every endpoint - has the same `{ error: { code, message } }` JSON body. This is implemented once, in `AppError`'s `IntoResponse` impl, so handlers never hand-roll error formatting.
- **`lib.rs` / `main.rs` split.** The application logic lives in a library crate; `main.rs` is a thin binary entry point. This is what makes `tests/` able to spin up a real, fully-wired server instance per test without duplicating bootstrap code.
- **Session tokens are hashed (SHA-256); service tokens are encrypted (AES-256-GCM).** Different threat models: a session token only needs to be *verified* (hash comparison is enough, and irreversible by design), while a service token (GitHub, Discord) must be *reused* to call the external API, so it must be decryptable.
- **WebSocket broadcaster is transport-only.** It exposes `to_team(team_id, event)` and `to_user(user_id, event)`; only services call it, never handlers. Adding a new event type is a new enum variant in `WsEvent` plus a call site in a service - the broadcaster itself never changes.

- **Severity is orthogonal to state.** An incident's severity (`low`/`medium`/`high`/`critical`) and its lifecycle state (`open`/`acknowledged`/`escalated`/`resolved`) are independent axes. You can raise severity without changing state, and an escalation *may* raise severity in the same gesture but doesn't have to. This is a deliberate interpretation of the subject: the `escalated` state represents management involvement, not just a severity bump. The state machine enforces this separation — `PATCH .../status` and `PATCH .../severity` are separate endpoints with separate validation.

- **State machine transitions are strict and centralized.** All valid transitions live in a single pure function (`domain::incidents::can_transition`) with no database or HTTP dependency. The allowed matrix is: `open → acknowledged`, `acknowledged → escalated`, `acknowledged → resolved` (shortcut — not all incidents escalate), `escalated → resolved`. Everything else returns 422. This function is the single source of truth called by the service layer, the rule engine, and (later) any future caller. The shortcut `acknowledged → resolved` is a deliberate choice documented in the README because the subject's state diagram could be read either way.

- **Timeline entry length limit: 2000 characters.** Enforced server-side, consistent with the private message limit. Chosen as a reasonable ceiling for an operational note — long enough for a stack trace excerpt, short enough to prevent abuse. The limit is validated in the service layer before touching the database.

---

### UUID generation

All UUIDs are generated in Rust (`Uuid::new_v4()`) before being passed to the INSERT query.
The database columns have no `DEFAULT gen_random_uuid()`. This keeps ID generation independent
from the database engine and allows the application layer to know the entity ID before persistence,
which simplifies logging, event broadcasting, and tracing.

---

## Contract

**Sign up**
POST /auth/signup
Body: { "email": string, "password": string, "display_name": string }

201 Created  => { id, email, display_name, language, created_at }
422 Unprocessable => email too short / password < 8 / display_name null
409 Conflict      => email already exists

WS : none.


**Sign in + sessions**
POST /auth/signin
Body: { "email": string, "password": string }

200 OK    => { "token": "hex string", "user": { id, email, display_name, language, created_at } }
401       => "invalid credentials"

WS : none.


GET /me
Header: Authorization: Bearer <token_hex>

200 OK  => { id, email, display_name, language, created_at }
401     => token invalid, expired or absent


**Sign out**
POST /auth/signout
Header: Authorization: Bearer <token>

204 No Content  => session deleted, token invalid
401             => token absent or already invalid

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
