# VIGIL

VIGIL is a collaborative operational control room that handles both realities of production operations in real time: **Releases** (planned deployments, validated step by step) and **Incidents** (detected problems, triaged and resolved). The two are connected -- a Release can automatically trigger an Incident, and an active Incident can block an ongoing Release.

> Hajatiana Rabemananjara

---

## Stack

| Component          | Technology              | Justification       |
|--------------------|-------------------------|----------------------|
| Application server | **Rust (Axum)**         | Imposed              |
| Web client         | **Next.js** (TypeScript)| Imposed              |
| Desktop client     | **Tauri v2**            | Chosen -- see below  |
| Persistence        | **PostgreSQL**          | Chosen -- see below  |
| Real-time          | **WebSockets**          | Imposed              |
| Containerization   | **Docker Compose**      | Imposed              |

### Why Tauri over Electron

Tauri was chosen over Electron for three concrete reasons:

- **Binary size and footprint.** The AppImage weighs ~100 MB versus 200-400 MB for Electron, which ships its own Chromium. Tauri uses the system webview (WebKitGTK on Linux), lighter for a tool expected to stay open in the background.
- **Security model by capabilities.** Every native API (notifications, tray, filesystem) must be explicitly declared in a capability file, scoped to specific windows and URLs. This forces a clean separation between the front and the OS.
- **Consistency with the Rust backend.** The desktop shell (`lib.rs`) reuses the same language and idioms as the server. The embedded HTTP server is 40 lines of Rust with `tiny_http`, versus a separate Node bundling step in Electron.

### Why PostgreSQL over SQLite

VIGIL's schema leans on three Postgres features that SQLite supports weakly or not at all:

- **Partial unique indexes** enforce invariants like exactly one active Manager per team, one active assignee per incident, one active link per (release, incident). SQLite supports partial indexes but Postgres's are more mature at this constraint density.
- **JSONB** with native indexing is used for rule filters, reaction payloads, and webhook deliveries. SQLite's JSON support is function-based over TEXT, not a real binary type.
- **Concurrent writers**: SQLite serializes writes at the database level. Postgres handles concurrent writes with row-level locking, which fits a tool where multiple operators act on the same resources simultaneously.

PostgreSQL also fits the deployment story naturally -- `db` is one of the four services in `docker-compose.yml`.

---

## Architecture

The project is a monorepo: `server/` (Rust/Axum) and `client/` (Next.js). The client folder hosts both the web and desktop targets -- the Next.js codebase under `client/src/` is the single source of truth for the UI. It is built with `output: 'export'` (static, CSR only). The Tauri desktop shell at `client/src-tauri/` embeds this static export, ensuring feature parity by construction.

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
**Core principle**: writes go up via REST, truth comes back down via WebSocket. A client never trusts its own HTTP response to update its UI -- it waits for the broadcast, exactly like every other connected client.

### Codebase navigation

| Layer | Location | Responsibility |
|-------|----------|----------------|
| Routes | `server/src/routes/` | HTTP route definitions, wiring only |
| Handlers | `server/src/handlers/` | Request extraction, response formatting. No SQL, no business rules |
| Services | `server/src/services/` | Business logic. Orchestrates repo + broadcaster + audit |
| Domain | `server/src/domain/` | Pure types and rules (e.g. `can_transition`). Zero I/O |
| Repo | `server/src/repo/` | The only layer that touches SQL (via `sqlx`) |
| WebSocket | `server/src/ws/` | Broadcaster (transport only). Services decide what and to whom |
| Hooks | `server/src/hooks/` | Rule engine: registry, matcher, templating, reactions |

The crate is split into `lib.rs` (declares modules, re-exports core types) and a thin `main.rs` (wires config, pool, router). This split lets integration tests import the application as a library and spin up real instances against disposable databases.

| Area | Location | Responsibility |
|------|----------|----------------|
| Pages | `client/src/app/` | App Router pages |
| Components | `client/src/components/` | Reusable UI (`StateBadge`, `SeverityBadge`, `ConfirmDialog`, `AppShell`) |
| shadcn | `client/src/components/ui/` | Generated shadcn/ui primitives (Radix-based) |
| Lib | `client/src/lib/` | `api.ts` (HTTP client), `platform.ts` (native abstraction), `i18n.ts`, `useRouteParams.ts` |
| Stores | `client/src/stores/` | `auth.tsx` (React Context), `socket.tsx` (WebSocket provider) |

### Frontend conventions

- Next.js **App Router**, TypeScript strict mode, Tailwind CSS, shadcn/ui for base components
- No hardcoded user-facing strings: every label goes through `t()` from `client/src/lib/i18n.ts`
- Native capabilities behind `client/src/lib/platform.ts`. Components never call Tauri APIs directly
- All UUIDs generated in Rust (`Uuid::new_v4()`) before INSERT, never via DB defaults
- Rust: snake_case, PascalCase types, enforced by `cargo fmt` + clippy. TypeScript: camelCase, PascalCase types, enforced by ESLint + Prettier

### Authentication

- Opaque session tokens (32 random bytes, hex-encoded)
- Server stores SHA-256 of the token as `BYTEA` (irreversible)
- Passwords hashed with Argon2 (PHC string, stored as TEXT)
- Token sent via `Authorization: Bearer <token>` on every authenticated request
- Client-side storage in `localStorage`. Trade-off: simpler than HttpOnly cookies, at the cost of XSS exposure. Acceptable for an academic project

### Client-side auth flow

The auth state lives in a React Context (`client/src/stores/auth.tsx`, exposed via `useAuth()`):

- On mount, reads the token from `localStorage` and validates it with `GET /me`. An `isLoading` flag gates the UI until this check resolves
- `RequireAuth` wrapper redirects unauthenticated users to `/signin`
- Signup auto-signs in (the API returns no token on signup, so the client calls signin immediately)
- Signout is local-first: clears state even if the server call fails
- Post-login routing via `postLoginDestination()`: 0 teams goes to `/onboarding`, 1+ teams goes to the last active team's incidents view

All HTTP calls go through `client/src/lib/api.ts`, which injects the Bearer header and throws typed `ApiError` with HTTP status and server error code.

---

## Installation and local setup

### Prerequisites

- Rust stable (version pinned via `rust-toolchain.toml`)
- Node.js 24+
- Docker and Docker Compose
- `sqlx-cli`: `cargo install sqlx-cli --no-default-features --features postgres`

### Quick start

```bash
# 1. Start the database
docker compose -f docker-compose.dev.yml up -d
docker compose -f docker-compose.dev.yml ps   # wait for "healthy"

# 2. Run the server (migrations apply automatically on startup)
cd server
cargo run

# 3. Run the web client
cd client
npm install
npm run dev
```

The server listens on `http://localhost:8080`. The web client on `http://localhost:3000`.

```bash
curl http://localhost:8080/health
# { "status": "ok", "version": "0.1.0" }
```

Adminer (database UI) is available at `http://localhost:8888`.

### Running tests

```bash
cd server
cargo test
```

Each test spins up its own disposable database (via `spawn_app()` in `tests/common/mod.rs`), runs all migrations, and tears it down afterward. Full isolation, safe to run in parallel.

### Linting and formatting

```bash
cargo fmt --check
cargo clippy
cd client && npx eslint . && npx prettier --check .
```

---

## Desktop application

The desktop client is a Tauri v2 application targeting **Linux/AppImage** (Ubuntu 24.04 tested). It exposes exactly the same features as the web client, plus tray icon and OS notifications.

### Standalone by construction

The AppImage is standalone: no Node runtime, no external server needed at launch. The static Next.js export (`out/`) is embedded in the binary and served internally by an embedded HTTP server (`tiny_http`) on `http://localhost:9527`.

Rationale: Next.js static export produces one HTML per dynamic route (`/teams/placeholder`, not `/teams/<uuid>`). A naive file server returns 404 on real UUIDs. The embedded server rewrites any UUID-like path segment to `placeholder` while preserving the real URL, so client-side routing reads the actual identifier via `useRouteParams()` (`client/src/lib/useRouteParams.ts`) rather than `useParams()` (which returns the frozen build-time value `placeholder`).

### Tray icon and background lifecycle

Closing the window does not terminate the app. The window is hidden, the WebSocket stays connected, and the app remains represented by a tray icon. The tray menu exposes `Open` (restore window) and `Quit` (real exit).

Requires the AppIndicator GNOME extension (preinstalled on Ubuntu 24.04).

### Native OS notifications

Three triggers, fired from a central hook (`client/src/lib/useNotifications.ts`):

- Incident assigned to the current user (`incident_assigned`)
- Incident escalated to critical severity (`incident_escalated`)
- Release blocked by a linked incident (`release_state_changed`)

Notifications use the browser `Notification` API from within the WebKitGTK webview, dispatched uniformly on web and desktop through `platform.ts`.

**Known limitation.** On GNOME 46+, notifications emitted from a webview may not display even though the API returns success. This is a documented environment bug (tauri-apps/tauri#14095, tauri-apps/plugins-workspace#2566), independent of application code. Verified by logs showing successful emission on every trigger.

### Building the AppImage

```bash
cd client
npx tauri build --bundles appimage
```

Output: `client/src-tauri/target/release/bundle/appimage/*.AppImage`.

### Installing and running

Requires `libfuse2t64` on the host:

```bash
sudo apt install libfuse2t64
chmod +x vigil-desktop_*.AppImage
./vigil-desktop_*.AppImage
```

---

## Running with Docker

The full stack runs via Docker Compose with four services:

| Service          | Role                                    | Port |
|------------------|-----------------------------------------|------|
| `db`             | PostgreSQL 16                           | --   |
| `server`         | Rust/Axum API + WebSocket               | 8080 |
| `client_web`     | Nginx serving the Next.js static export | 8081 |
| `client_desktop` | Builds the AppImage into a shared volume| --   |

`client_web` depends on `client_desktop` completing successfully. The built AppImage is exposed for download at `http://localhost:8081/client.AppImage`.

Two compose files are provided:

- `docker-compose.dev.yml`: db + adminer only, for local development with `cargo run` and `npm run dev` on the host
- `docker-compose.yml`: the full production-like stack

### Launch the full stack

Copy `.env.example` to `.env` and fill in the sensitive values. `MASTER_KEY_HEX` must be 64 hex characters:

```bash
openssl rand -hex 32
```

Then:

```bash
docker compose up --build
```

First build takes several minutes (Rust compilation + Next build + AppImage bundling). Subsequent builds use the Docker layer cache.

- Web client: `http://localhost:8081`
- API: `http://localhost:8080`
- Desktop binary: `http://localhost:8081/client.AppImage`

---

## REST API

All endpoints return errors in a uniform shape:

```json
{ "error": { "code": "VALIDATION_ERROR", "message": "..." } }
```

| HTTP Status | Code | Meaning |
|-------------|------|---------|
| 401 | `UNAUTHORIZED` | Missing or invalid session token |
| 403 | `FORBIDDEN` | Authenticated but not permitted |
| 404 | `NOT_FOUND` | Resource doesn't exist (or hidden) |
| 409 | `CONFLICT` | Unique constraint violation |
| 422 | `VALIDATION_ERROR` | Input failed validation |

### Health

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/health` | none | `{ "status": "ok", "version": "0.1.0" }` |

### Authentication

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/auth/signup` | none | Body: `{ email, password, display_name }`. Returns user (201). 409 if email taken |
| POST | `/auth/signin` | none | Returns `{ token, user }`. 401 if invalid |
| GET | `/me` | session | Current user info |
| POST | `/auth/signout` | session | Deletes the session. 204 |

### Teams

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/teams` | session | Create a team. Creator becomes Manager |
| GET | `/teams` | session | List teams the user belongs to |
| GET | `/teams/{team_id}` | member | Team detail. Non-member gets 404 |
| GET | `/teams/{team_id}/members` | member | Member list with roles |
| PATCH | `/teams/{team_id}/members/{user_id}/role` | Manager | Body: `{ role }`. Promote/demote Observer/Responder |
| POST | `/teams/{team_id}/transfer-manager` | Manager | Body: `{ target_user_id }`. Atomic swap |
| POST | `/teams/{team_id}/leave` | member | Manager without transfer gets 409 |
| POST | `/teams/{team_id}/invitations` | Manager | Returns `{ code }` |
| POST | `/teams/join` | session | Body: `{ code }`. Joins as Observer. Banned user gets 403 |

### Incidents

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/teams/{team_id}/incidents` | Manager | Body: `{ title, body, severity }`. Initial state `open` |
| GET | `/teams/{team_id}/incidents` | member | Filterable: `?status=`, `?severity=` |
| GET | `/teams/{team_id}/incidents/{id}` | member | Single incident with `assignee_id` |
| PATCH | `/teams/{team_id}/incidents/{id}/status` | Responder+ | Body: `{ status, severity? }`. State machine enforced |
| PATCH | `/teams/{team_id}/incidents/{id}/severity` | Responder+ | Body: `{ severity }` |
| POST | `/teams/{team_id}/incidents/{id}/assign` | Manager | Body: `{ user_id }`. Target must be Responder+ |
| POST | `/teams/{team_id}/incidents/{id}/timeline` | Responder+ | Body: `{ content }`. Max 2000 chars |
| GET | `/teams/{team_id}/incidents/{id}/timeline` | member | Chronological entries |

### Releases

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/teams/{team_id}/releases` | Manager | Body: `{ title, body?, steps: [...] }`. Steps are names; positions derived server-side. Max 20 steps, title max 200 chars |
| GET | `/teams/{team_id}/releases` | member | Filterable: `?status=`. Includes `progress { completed, total }` |
| GET | `/teams/{team_id}/releases/{id}` | member | Full detail: steps, linked_incidents, progress |
| POST | `/teams/{team_id}/releases/{id}/start` | Manager | `created` to `in_progress` |
| POST | `/teams/{team_id}/releases/{id}/cancel` | Manager | From `created`, `in_progress`, `blocked`. 422 from terminal states |
| POST | `/teams/{team_id}/releases/{id}/steps/{step_id}/validate` | Responder+ | Sequential order enforced. Auto-completes on last step. 409 if blocked |
| POST | `/teams/{team_id}/releases/{id}/link` | Manager | Body: `{ incident_id }`. Auto-blocks if release is `in_progress` and incident unresolved |
| POST | `/teams/{team_id}/releases/{id}/unlink` | Manager | Body: `{ incident_id }`. Auto-unblocks when no active unresolved links remain |

### Webhooks

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/webhooks/github` | HMAC | Validates `X-Hub-Signature-256` (HMAC-SHA256). Persists raw payload. Returns 202; rule evaluation runs async |

### Rules

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/teams/{team_id}/rules` | member | List rules |
| POST | `/teams/{team_id}/rules` | Manager | Create rule. Trigger and reaction validated against registries |
| GET | `/teams/{team_id}/rules/{id}` | member | Single rule |
| PATCH | `/teams/{team_id}/rules/{id}` | Manager | Partial update |
| DELETE | `/teams/{team_id}/rules/{id}` | Manager | 204 |

### Service connections

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/me/services` | session | List connected services (no secrets exposed) |
| POST | `/me/services/{service}` | session | Body: `{ token }`. Encrypted AES-256-GCM at rest. Upsert. Service must match DB constraint |
| DELETE | `/me/services/{service}` | session | 204. Deletes the encrypted token |

### Discovery

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/about.json` | none | Dynamic catalog of Actions, Reactions, kickoff token |

---

## Database schema

Full DDL lives in `server/migrations/`. Nineteen tables in seven blocks:

**Core identity**: `users`, `sessions`, `teams`, `team_members`. The 3-role system (`observer`/`responder`/`manager`) with a partial unique index guaranteeing exactly one active Manager per team.

**Membership lifecycle**: `invitations`, `team_bans`. `invitations.code` is globally unique; `team_bans` uses a partial unique index per `(team_id, user_id)`.

**Incidents**: `incidents`, `incident_assignments`, `timeline_entries`, `timeline_reactions`. `incident_assignments` enforces one active assignee via partial unique index.

**Releases**: `releases`, `release_steps`, `release_incident_links`. The N-N link table drives the automatic `blocked` state.

**Social**: `private_messages`. Bilateral only. No `team_id`; access checked via shared-team query at send time.

**Automation**: `service_connections`, `rules`, `rule_executions`, `webhook_deliveries`. `service_connections` stores AES-256-GCM-encrypted tokens. `rules` separates filterable columns from JSONB payloads.

**Audit**: `audit_log`. No foreign keys (survives deletions). Append-only.

### Conventions

- UUID v4 primary keys, generated server-side
- `TIMESTAMPTZ` for every timestamp
- Enums are `TEXT` + `CHECK`, never native Postgres enum types
- History preserved via `status` column, never via `DELETE`
- Foreign keys to `users` never cascade; foreign keys to `teams` cascade

---

## Rule engine

VIGIL turns external events into actions through a rule engine built on two extension seams: a catalog of Actions (what services can send us) and a registry of Reactions (what we can do in response).

### Pipeline

The webhook receiver validates HMAC, persists the raw payload to `webhook_deliveries`, and responds 202 immediately. Rule evaluation runs async via `tokio::spawn`: load matching enabled rules, apply dot-notation filters (implicit AND, strict equality), resolve `{{path.to.field}}` template placeholders against the payload, then call `Reaction.execute()` via the trait dispatch. Each rule runs in isolation; a failure in one never affects others.

### Catalog vs registry

| | ActionCatalog | ReactionRegistry |
|---|---|---|
| Holds | Metadata (service, event, description) | Types implementing `ReactionExecutor` |
| Exposed via | `/about.json` actions | `/about.json` reactions |
| Runtime role | Filter incoming webhooks | Dispatch reactions by kind |

Actions are declarative metadata. Reactions have runtime behavior (`execute()`), justifying the trait. Both cost one line in `main.rs` to add.

### Filters

Dot-notation paths matched against the payload. All must match (AND). `{}` matches everything.

```json
{ "workflow_run.conclusion": "failure", "repository.full_name": "hajatiana/vigil" }
```

### Templating

`{{path.to.field}}` placeholders resolved against the payload. Unresolved placeholders are left literal to surface mistakes at first run.

### Registered reactions

| Kind | Service | Effect |
|------|---------|--------|
| `vigil_create_incident` | vigil | Creates an incident on the rule's team |
| `vigil_escalate_incident` | vigil | Escalates an existing incident |
| `vigil_block_release` | vigil | Links an incident to a release, triggering auto-block |
| `vigil_validate_release_step` | vigil | Advances a release step |
| `discord_message` | discord | Posts a message to a Discord webhook URL |

### Service connections

Connected per-user via `POST /me/services/{service}`. Tokens encrypted AES-256-GCM at rest, decrypted just-in-time inside reactions. Never logged, never returned.

**Known limitation.** GitHub tokens are stored but not consumed at runtime: the webhook receiver authenticates via a global `WEBHOOK_SECRET` (HMAC), not per-user tokens.

---

## `/about.json`

Public discovery endpoint. Clients build their rule form UI entirely from this response.

```json
{
  "client": { "host": "10.0.0.1" },
  "server": {
    "current_time": 1718000000,
    "token": "3f2a9b...e4c1",
    "services": [
      {
        "name": "github",
        "connectable": true,
        "actions": [
          { "name": "workflow_run", "description": "A CI workflow run has completed" }
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
            "description": "Create a VIGIL incident",
            "payload_example": "{ \"title\": \"CI broken on {{repository.name}}\", \"severity\": \"high\" }"
          }
        ]
      }
    ]
  }
}
```

- `client.host`: requesting client's IP
- `server.token`: SHA-256 of `STUDENT_FIRSTNAME + STUDENT_LOGIN + "VIGIL2026"`, computed at startup
- `connectable`: whether a user can attach a token to this service
- `payload_example`: well-formed example, used to prefill the rule form textarea

---

## WebSocket events

Full specification lives in [WEBSOCKET_SPEC.md](./WEBSOCKET_SPEC.md).

---

## Configuration

| Variable            | Required | Description                                              |
|---------------------|----------|----------------------------------------------------------|
| `DATABASE_URL`      | yes      | PostgreSQL connection string                             |
| `SERVER_HOST`       | no       | Default `0.0.0.0`                                       |
| `SERVER_PORT`       | no       | Default `8080`                                           |
| `WEBHOOK_SECRET`    | no       | HMAC secret for webhook validation (default: dev secret) |
| `MASTER_KEY_HEX`    | yes      | 64 hex chars (32 bytes) for AES-256-GCM encryption      |
| `STUDENT_FIRSTNAME` | yes      | Derives the `/about.json` kickoff token                  |
| `STUDENT_LOGIN`     | yes      | Derives the `/about.json` kickoff token                  |

---

## Design decisions

- **Permission extractors, not middleware.** Access control is implemented as Axum extractors (`TeamMember`, `RequireResponder`, `RequireManager`) declared in handler signatures. A handler's required role is self-documenting. Adding a new protected route reuses an existing extractor (Open/Closed).

- **Team access returns 404, never 403.** Returning 403 would reveal that the team exists. Returning 404 makes a non-existent team indistinguishable from one the user isn't part of.

- **Role hierarchy is numeric.** Observer (0) < Responder (1) < Manager (2). `has_at_least(required)` compares levels. Adding a role changes one function.

- **Severity is orthogonal to state.** An incident's severity and lifecycle state are independent axes. You can raise severity without changing state. `PATCH .../status` and `PATCH .../severity` are separate endpoints.

- **State machine transitions are strict.** All valid transitions live in a single pure function (`domain::incidents::can_transition`). The shortcut `acknowledged -> resolved` is deliberate (not all incidents escalate).

- **Auto-blocking cascade.** Linking a non-resolved incident to an `in_progress` release blocks it. Resolution or unlinking unblocks when no active unresolved links remain. Multi-incident blocking handled correctly.

- **Sequential step validation.** Steps must be validated in position order. The last step triggers auto-completion.

- **Session tokens are hashed (SHA-256); service tokens are encrypted (AES-256-GCM).** Sessions only need verification (irreversible by design). Service tokens must be reused to call external APIs (reversible by necessity).

- **WebSocket broadcaster is transport-only.** It exposes `to_team()` and `to_user()`; only services call it. Adding a new event is a new enum variant plus a call site.

- **Uniform error shape.** Every error response has the same `{ error: { code, message } }` body, implemented once in `AppError::IntoResponse`.

- **CORS allows multiple dev origins.** `http://localhost:3000` (dev), `http://localhost:8081` (Docker web), `http://localhost:9527` (Tauri webview). A production deployment should read origins from an environment variable.

- **Expired invitation codes return 410.** 404 means "check your spelling", 410 means "ask the Manager for a new code".

- **Blocked release returns 409.** Not 422, because the release itself is valid -- an external condition prevents progress.

---

## Documented limits

- Timeline entry: 2000 characters
- Private message: 2000 characters
- Release title: 200 characters
- Release steps: 1 to 20
- Step name: 100 characters
- Available reaction emojis: `+1`, `-1`, `eyes`, `warning`, `check`, `fire`
- Invitation code: 8 characters, human-safe alphabet (no 0/O, 1/I/L)

---

## Known limitations

- **`find_membership` takes `&PgPool`, not `&mut PgConnection`.** Prevents use inside a transaction. The transfer service calls it before `pool.begin()`, introducing a theoretical TOCTOU race. Guarded by the `promoted == false` check.

- **Test helpers are duplicated across test files.** `register_and_login`, `create_team_and_invite` are copy-pasted. Extracting into `tests/common/` deferred.

- **Start does not check existing links.** Starting a release with already-linked unresolved incidents transitions to `in_progress` without auto-blocking. The next incident resolution triggers the check.

---

## License

Author: Hajatiana Rabemananjara.