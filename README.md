# VIGIL


It is a collaborative operational control room that handles both realities in real time. Teams coordinate their Releases (planned deployments, validated st by st) and their Incidents (detected problems, triaged and resolved). The two are connected: a Release can automatically trigger an Incident, and an active Incident can block an ongoing Release.

---

## Stack

| Component | Technology | Justification |
|-----------|-----------|---------------|
| Application server | **Rust (Axum)** | Imposed |
| Web client | **Next.js** (TypeScript) | Imposed |
| Desktop client | **Tauri** | _TODO_ |
| Persistence | **PostgreSQL** | _TODO_ |
| Real-time | **WebSockets** | Imposed |
| Containerization | **Docker Compose** | Imposed |

### Why Rust (Axum) over NodeJS

_TODO: justify_

#### Other
rust-toolchain.toml: To fixe Rust version
.editorconfig : shared format convetions

### Why Tauri over Electron

_TODO: justify_

### Why PostgreSQL over SQLite

_TODO: justify_

---

## Architecture

```
TODO: architecture diagram
```

### Codebase navigation

| Layer | Location | Responsibility |
|-------|----------|----------------|
| Routes | `server/src/routes/` | HTTP route definitions |
| Handlers | `server/src/handlers/` | Request extraction, response formatting |
| Services | `server/src/services/` | Business logic |
| Domain | `server/src/domain/` | Types, enums, validation |
| Repo | `server/src/repo/` | SQL queries (sqlx) |
| WebSocket | `server/src/ws/` | Broadcaster, event dispatch |

---

## Installation & Local Setup

### Prerequisites

- Rust (stable)
- Docker & Docker Compose
- Node.js 20+

### Quick start

```bash
# 1. Installation:
cargo init server
cargo install sqlx-cli --no-default-features --features postgres

# 2. Start database
cp .env.example .env
docker compose -f docker-compose.dev.yml up -d

# 3. Migration
cd server
sqlx migrate add initial_schema

# 4. Run migrations
cd server && cargo sqlx migrate run

# 4. Start server
cargo run
```

---

## REST API

_TODO: document all endpoints_

---

## Database Schema

_TODO: commented schema (see docs/)_

---

## WebSocket Events

See [WEBSOCKET_SPEC.md](./WEBSOCKET_SPEC.md)

---

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | — |
| `SERVER_HOST` | Bind address | `0.0.0.0` |
| `SERVER_PORT` | Bind port | `8080` |

---

## Design Decisions

_TODO: document key architectural decisions_

---

## Target OS

**Linux** — desktop binary delivered as `.AppImage`.

---

## Private Messages

- Maximum message length: **2000 characters** (enforced server-side)

## Reactions

- Available emojis: `+1`, `-1`, `eyes`, `warning`, `check`, `fire`

---

## License

Hajatiana Rabemananjara

