# VIGIL

Collaborative operational control room for managing **Releases** (planned deployments) and **Incidents** (detected problems) in real time. The two are connected: an active Incident can automatically block an ongoing Release.

> Hajatiana Rabemananjara

---

## Stack

| Component          | Technology              |
|--------------------|-------------------------|
| Application server | **Rust (Axum)**         |
| Web client         | **Next.js** (TypeScript)|
| Desktop client     | **Tauri v2**            |
| Persistence        | **PostgreSQL**          |
| Real-time          | **WebSockets**          |
| Containerization   | **Docker Compose**      |

### Why Tauri over Electron

Tauri produces a lighter binary, enforces a capability-based security model for native APIs, and shares the Rust language with the backend. The embedded HTTP server (`tiny_http`) is 40 lines of Rust.

### Why PostgreSQL over SQLite

VIGIL relies on partial unique indexes (one Manager per team, one active assignee per incident), JSONB for rule filters and payloads, and concurrent writers with row-level locking. PostgreSQL handles all three natively.

---

## Architecture

Layered Monolith with Event-Driven Broadcasting.


Monorepo: `server/` (Rust/Axum) and `client/` (Next.js). The client is built with `output: 'export'` (static CSR). Tauri embeds this export, so web and desktop share the same codebase.

External services (GitHub, Discord, webhooks...)
                          │
                          │  POST /webhooks/{service}
                          v
┌──────────────────────────────────────────────────────────┐
│           Application Server  (Rust / Axum)              │
│                                                          │
│  ┌─────────────────┐   ┌──────────────────────┐          │
│  │ Webhook Receiver│   │    Hook Engine       │          │
│  │ HMAC validation ├──>│  (rule evaluation)   │          │
│  └─────────────────┘   └──────────┬───────────┘          │
│                                   │                      │
│  ┌────────────────────────────────v───────────────────┐  │
│  │                 WS Broadcaster                     │  │
│  │  - Release / Incident state updates                │  │
│  │  - Collaborative timeline                          │  │
│  │  - Presence (who is watching what)                 │  │
│  │  - Live feed of triggered rules                    │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  routes/ → handlers/ → services/ → domain/ → repo/       │
│                                                 │        │
│                                          PostgreSQL      │
│                                            (sqlx)        │
└────────────────────────┬─────────────────────────────────┘
                         │  WebSocket + REST
            ┌────────────┴────────────┐
            v                         v
  ┌────────────────────┐   ┌───────────────────────┐
  │     Web Client     │   │    Desktop Client     │
  │   Next.js 16       │   │    Tauri v2           │
  │   App Router       │   │    standalone         │
  │   Tailwind v4      │   │                       │
  │   shadcn/ui Nova   │   │    + Tray icon        │
  │                    │   │    + Notifications OS │
  │   All features     │   │    All features       │
  └────────────────────┘   └───────────────────────┘

**Core principle**: writes go up via REST, truth comes back down via WebSocket.


Every layer has a Single Responsibility and can only call the layer below.

### Codebase navigation

| Layer | Location | What it does |
|-------|----------|--------------|
| Routes | `server/src/routes/` | HTTP wiring only |
| Handlers | `server/src/handlers/` | Parse requests, format responses |
| Services | `server/src/services/` | Business logic, orchestrates repo + broadcaster |
| Domain | `server/src/domain/` | Pure types and rules (e.g. `can_transition`) |
| Repo | `server/src/repo/` | SQL queries via `sqlx` |
| WebSocket | `server/src/ws/` | Broadcaster (transport only) |
| Hooks | `server/src/hooks/` | Rule engine: registry, matcher, templating, reactions |

Frontend:

| Area | Location |
|------|----------|
| Pages | `client/src/app/` |
| Components | `client/src/components/` |
| Lib | `client/src/lib/` (api, i18n, platform, socket) |
| Stores | `client/src/stores/` (auth, socket) |

### Authentication

Opaque session tokens (32 random bytes). Server stores SHA-256 hash. Passwords hashed with Argon2. Token via `Authorization: Bearer`. OAuth2 GitHub also supported.

### i18n

French and English. Dictionaries in `client/src/locales/{en,fr}.json`. The `t()` function is typed with `TranslationKey` -- typos caught at compile time. Language changeable from profile, header menu, or signin page. Persisted server-side via `PATCH /me`.

### Profile

`PATCH /me` accepts `display_name`, `password`, `language`, `avatar_seed` (all optional). Profile page accessible from the user menu.

### Avatars

42 preset avatars generated by DiceBear (avataaars style). Client-side SVG generation, no network. Seed persisted in DB. Fallback: initials.

```bash
cd client && npm install @dicebear/core @dicebear/styles
```

### Audit log

Append-only log of governance actions (kick, ban, role change, transfer, rule changes, release start/cancel). Read-only, Manager-only. `GET /teams/{team_id}/audit`.

### OAuth2 GitHub

Sign in with GitHub via Authorization Code flow. Email matching links existing accounts, otherwise creates a new one.

Setup: create an OAuth App at https://github.com/settings/developers, set callback to `http://localhost:3000/auth/github/callback`, add `GITHUB_CLIENT_ID` and `GITHUB_CLIENT_SECRET` to `.env`.

---

## Quick start

```bash
# Database
docker compose -f docker-compose.dev.yml up -d

# Server (migrations run automatically)
cd server && cargo run

# Web client
cd client && npm install && npm run dev
```

Server: `http://localhost:8080` | Client: `http://localhost:3000` | Adminer: `http://localhost:8888`

### Running tests

```bash
# Backend (107 tests)
cd server && cargo test

# Frontend (339 tests)
cd client && npm test -- --run
```

### Coverage

```bash
# Server: HTML report in docs/coverage/
cd server && cargo llvm-cov --html --output-dir ../docs/coverage --ignore-run-fail -- --test-threads=2

# Client
cd client && npm test -- --run --coverage
```

### Open report

```bash
# Server: HTML report in docs/coverage/
xdg-open docs/coverage/html/index.html

# Client
xdg-open docs/coverage/html/index.html
```

| Component | Coverage | Threshold |
|-----------|----------|-----------|
| Server | **+81%** | 70% |
| Client | **+70%** | 70% |

### Linting

```bash
cargo fmt --check && cargo clippy
cd client && npx eslint . && npx prettier --check .
```

---

## Desktop application

Tauri v2 targeting **Linux/AppImage**. All VIGIL features + tray icon + OS notifications.

The AppImage is standalone: no Node runtime needed. Static Next.js export embedded in the binary, served by `tiny_http` on `localhost:9527`.

Notifications: assignment, critical severity, blocked release, DMs, promotions, rule events. Dispatched via `notify-send` (GNOME 46+ workaround for tauri-apps/tauri#14095).

```bash
cd client && npx tauri build --bundles appimage
sudo apt install libfuse2t64
chmod +x vigil-desktop_*.AppImage && ./vigil-desktop_*.AppImage
```

---

## Docker

Four services: `db`, `server` (:8080), `client_web` (:8081), `client_desktop` (build only).

```bash
cp .env.example .env   # fill MASTER_KEY_HEX (openssl rand -hex 32)
docker compose up --build
```

Desktop binary: `http://localhost:8081/client.AppImage`

---

## REST API

Errors: `{ "error": { "code": "...", "message": "..." } }`

Status codes: 401 (unauthorized), 403 (forbidden), 404 (not found), 409 (conflict), 422 (validation).

### Health

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/health` | none | Status + version |

### Authentication

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/auth/signup` | none | `{ email, password, display_name }` -> 201 |
| POST | `/auth/signin` | none | -> `{ token, user }` |
| GET | `/me` | session | Current user |
| PATCH | `/me` | session | `{ display_name?, password?, language?, avatar_seed? }` |
| POST | `/auth/signout` | session | 204 |
| GET | `/auth/oauth/github` | none | Redirect to GitHub |
| GET | `/auth/oauth/github/callback` | none | Exchange code -> `{ token, user }` |

### Teams

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/teams` | session | Create team (creator = Manager) |
| GET | `/teams` | session | List my teams |
| GET | `/teams/{id}` | member | Team detail |
| GET | `/teams/{id}/members` | member | Member list |
| PATCH | `/teams/{id}/members/{uid}/role` | Manager | `{ role }` |
| POST | `/teams/{id}/transfer-manager` | Manager | `{ target_user_id }` |
| POST | `/teams/{id}/leave` | member | Manager must transfer first |
| POST | `/teams/{id}/invitations` | Manager | -> `{ code }` |
| POST | `/teams/join` | session | `{ code }` |
| POST | `/teams/{id}/members/{uid}/kick` | Manager | 204 |
| POST | `/teams/{id}/members/{uid}/ban` | Manager | `{ expires_at?, reason? }` |
| DELETE | `/teams/{id}/bans/{uid}` | Manager | Lift ban |

### Incidents

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/teams/{id}/incidents` | Manager | `{ title, body, severity }` |
| GET | `/teams/{id}/incidents` | member | `?status=&severity=` |
| GET | `/teams/{id}/incidents/{iid}` | member | Detail |
| PATCH | `/teams/{id}/incidents/{iid}/status` | Responder+ | `{ status, severity? }` |
| PATCH | `/teams/{id}/incidents/{iid}/severity` | Responder+ | `{ severity }` |
| POST | `/teams/{id}/incidents/{iid}/assign` | Manager | `{ user_id }` |
| POST | `/teams/{id}/incidents/{iid}/timeline` | Responder+ | `{ content }` max 2000 chars |
| GET | `/teams/{id}/incidents/{iid}/timeline` | member | Chronological |
| PATCH | `/timeline/{eid}` | author | `{ content }` |
| GET | `/teams/{id}/incidents/{iid}/reactions` | member | Grouped by entry + emoji |

### Reactions

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/reactions/available` | session | `+1, -1, eyes, warning, check, fire` |
| POST | `/timeline/{eid}/reactions` | session | `{ emoji }` |
| DELETE | `/timeline/{eid}/reactions/{emoji}` | session | Remove own |

### Private messages

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/messages/{uid}` | session | `{ content }` max 2000 chars, must share a team |
| GET | `/messages/{uid}` | session | `?before=&limit=` |

### Releases

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/teams/{id}/releases` | Manager | `{ title, body?, steps }` max 20 steps |
| GET | `/teams/{id}/releases` | member | `?status=` |
| GET | `/teams/{id}/releases/{rid}` | member | Detail with steps + linked incidents |
| POST | `/teams/{id}/releases/{rid}/start` | Manager | created -> in_progress |
| POST | `/teams/{id}/releases/{rid}/cancel` | Manager | 422 from terminal states |
| POST | `/teams/{id}/releases/{rid}/steps/{sid}/validate` | Responder+ | Sequential. 409 if blocked |
| POST | `/teams/{id}/releases/{rid}/link` | Manager | `{ incident_id }` auto-blocks |
| POST | `/teams/{id}/releases/{rid}/unlink` | Manager | `{ incident_id }` auto-unblocks |

### Webhooks, Rules, Services, Audit, Discovery

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/webhooks/github` | HMAC | 202, async processing |
| GET/POST/PATCH/DELETE | `/teams/{id}/rules[/{rid}]` | Manager (write) / member (read) | Rule CRUD |
| GET | `/teams/{id}/rules/executions` | member | 20 most recent |
| GET/POST/DELETE | `/me/services[/{service}]` | session | Token encrypted AES-256-GCM |
| GET | `/about.json` | none | Dynamic catalog + kickoff token |
| GET | `/teams/{id}/audit` | Manager | `?limit=&offset=` |

---

## Database schema

19 tables in `server/migrations/`. Key blocks:

- **Identity**: `users`, `sessions`, `teams`, `team_members` (3 roles, partial unique index for single Manager)
- **Membership**: `invitations`, `team_bans`
- **Incidents**: `incidents`, `incident_assignments`, `timeline_entries`, `timeline_reactions`
- **Releases**: `releases`, `release_steps`, `release_incident_links` (N-N, drives auto-blocking)
- **Social**: `private_messages` (bilateral, no team_id)
- **Automation**: `service_connections` (AES-256-GCM), `rules`, `rule_executions`, `webhook_deliveries`
- **Audit**: `audit_log` (append-only)

Conventions: UUID v4 PKs (generated server-side), `TIMESTAMPTZ`, enums as `TEXT + CHECK`, history via `status` column (never DELETE).

---

## Rule engine

External events trigger actions inside VIGIL. Built on two extension points: `ActionCatalog` (metadata) and `ReactionRegistry` (trait-based dispatch). Adding a service = 1 new file + 1 line in `main.rs`.

Pipeline: HMAC validation -> persist payload -> 202 -> async evaluation -> filter matching -> template resolution -> reaction execution. Each rule runs in isolation.

### Registered reactions

| Kind | Effect |
|------|--------|
| `vigil_create_incident` | Creates an incident |
| `vigil_escalate_incident` | Escalates an incident |
| `vigil_block_release` | Links incident to release, triggers auto-block |
| `vigil_validate_release_step` | Validates a release step |
| `discord_message` | Posts to a Discord webhook |

### Testing locally

**Simulated (no network):**

```bash
SECRET=$(grep WEBHOOK_SECRET .env | cut -d= -f2)
PAYLOAD='{"action":"completed","workflow_run":{"name":"Build","conclusion":"failure","html_url":"https://github.com/test/run/1"},"repository":{"name":"vigil","full_name":"haja/vigil"}}'
SIGNATURE=$(echo -n "$PAYLOAD" | openssl dgst -sha256 -hmac "$SECRET" | awk '"'"'{print "sha256="$2}'"'"')
curl -X POST http://localhost:8080/webhooks/github \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature-256: $SIGNATURE" \
  -H "X-GitHub-Event: workflow_run" \
  -d "$PAYLOAD"
```

**Live with ngrok:**

```bash
# Install: https://ngrok.com/download
ngrok http 8080
```

Configure webhook on GitHub repo (Settings > Webhooks), create a matching rule in VIGIL, push a failing CI. See [HOWTOCONTRIBUTE.md](./HOWTOCONTRIBUTE.md) for the full walkthrough.

**Discord:** connect a Discord webhook URL in `/settings/services`, create a `discord_message` rule. One CI failure triggers both an incident AND a Discord message (two separate rules, isolated failure domains).

---

## Configuration

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | yes | PostgreSQL connection string |
| `SERVER_HOST` | no | Default `0.0.0.0` |
| `SERVER_PORT` | no | Default `8080` |
| `WEBHOOK_SECRET` | no | HMAC secret for webhooks |
| `MASTER_KEY_HEX` | yes | 64 hex chars for AES-256-GCM |
| `STUDENT_FIRSTNAME` | yes | For `/about.json` token |
| `STUDENT_LOGIN` | yes | For `/about.json` token |
| `GITHUB_CLIENT_ID` | no | OAuth2 (optional) |
| `GITHUB_CLIENT_SECRET` | no | OAuth2 (optional) |

---

## Key design decisions

- **One rule = one reaction.** Multiple effects from one event = multiple rules. Failure isolation: Discord down never blocks incident creation.
- **Team access returns 404, never 403.** Non-members can't tell if a team exists.
- **State machines are strict.** Transitions in a single pure function. Shortcut `acknowledged -> resolved` is intentional.
- **Auto-blocking cascade.** Linking an unresolved incident to an in_progress release blocks it. Multi-incident handled.
- **Kick/ban broadcast via `to_user`.** The target is already deactivated when `to_team` fires, so they need a direct push.
- **Ban expiry checked at join time.** No background job, no scheduler. Expired bans are invisible to the join check.
- **Audit is fire-and-forget.** A failed audit write never blocks the action itself.
- **i18n without a framework.** Custom `t()` with compile-time type safety. Fallback: current lang -> EN -> raw key.
- **Avatars are client-side.** DiceBear SVGs generated in-memory from a seed. No network dependency.

---

## WebSocket events

Full spec: [WEBSOCKET_SPEC.md](./WEBSOCKET_SPEC.md)

---

## Documented limits

- Timeline / DM: 2000 chars | Release title: 200 chars | Steps: 1-20, name max 100 chars
- Emojis: `+1` `-1` `eyes` `warning` `check` `fire`
- Invitation code: 8 chars, human-safe alphabet

---

## License

Author: Hajatiana Rabemananjara.