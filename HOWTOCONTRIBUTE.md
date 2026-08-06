# How to contribute

How to extend VIGIL. Each section is a recipe: follow the steps, run the tests, ship.

The engine is built on Open/Closed: adding a new Action, Reaction, or WebSocket event never modifies the engine core or the broadcaster. You add files at the edge.

---

## Adding a new Action

An Action is pure metadata (no runtime behavior). It declares that a service can send us an event. Registered actions show up in `/about.json` automatically.

**Example: add GitLab's `pipeline` event.**

1. In `server/src/main.rs`, in the `ActionCatalog::builder()` block:

```rust
.register("gitlab", "pipeline", "A GitLab CI/CD pipeline has finished")
```

2. Same line in `server/tests/common/mod.rs` (test harness).

3. Verify: `cargo test --test about_e2e`

**Files modified: 2.**

---

## Adding a new Reaction

A Reaction implements the `ReactionExecutor` trait. The engine calls `execute()` when a rule fires.

**Example: add `slack_message`.**

1. Create `server/src/hooks/reactions/slack_message.rs` implementing `ReactionExecutor` (see `discord_message.rs` as a template -- same pattern: deserialize payload, fetch encrypted token, POST to webhook URL).

2. Export from `server/src/hooks/reactions/mod.rs`:

```rust
pub mod slack_message;
pub use slack_message::SlackMessage;
```

3. Register in `main.rs`:

```rust
.register(Arc::new(SlackMessage::new()))
```

4. Same in `tests/common/mod.rs`.

5. If the service is new, add a `ServiceName` variant in `domain/service_connections.rs` and a migration updating the CHECK constraint.

6. Write tests following `tests/discord_reaction_e2e.rs` (happy path, no connection, 500 response).

**Files modified: 4 + 1 migration if new service.**

---

## Adding a WebSocket event

1. Add a variant to `WsEvent` in `server/src/ws/events.rs`:

```rust
DeploymentStarted {
    team_id: Uuid,
    release_id: Uuid,
    environment: String,
},
```

2. Emit from a service (never a handler):

```rust
broadcaster.to_team(team_id, WsEvent::DeploymentStarted { ... }).await;
```

3. Handle in the client via `useVigilSocket()`:

```tsx
useEffect(() => {
    if (lastEvent?.type !== "deployment_started") return;
    setState((prev) => [...prev, { ... }]);
}, [lastEvent]);
```

Rules: never fetch inside a WS handler (use `setState` with event data). Exception: new entity events where the full shape isn't in the payload.

4. Document in `WEBSOCKET_SPEC.md`.

**Files modified: 3 + 1 doc update.**

---

## Adding an audit-logged action

Call `audit::record()` in the service, after the action succeeds:

```rust
audit::record(pool, team_id, actor_id, "action_name", "entity_type", target_id,
    json!({ "target_name": name })).await;
```

Store names in metadata (not just UUIDs) since targets may be deleted later. The call is fire-and-forget: errors are logged but never block the action.

All entries are read via `GET /teams/{id}/audit`. No new endpoint needed.

**Files modified: 1.**

---

## Live webhook demo

1. Install ngrok: https://ngrok.com/download
2. `ngrok http 8080` (or with a static domain: `ngrok http --url=your-domain.ngrok-free.dev 8080`)
3. Configure webhook on GitHub repo: Settings > Webhooks, payload URL = ngrok URL + `/webhooks/github`, secret = your `WEBHOOK_SECRET`
4. Create a rule in VIGIL matching `github` / `workflow_run` with filter `{"workflow_run.conclusion": "failure"}`
5. Push a failing CI (`exit 1` in the workflow)

**Discord:** connect a webhook URL in `/settings/services`, create a `discord_message` rule. One CI failure triggers both an incident and a Discord message (two rules, isolated failure domains).

**Fallback (no network):**

```bash
SECRET=$(grep WEBHOOK_SECRET .env | cut -d= -f2)
PAYLOAD='{"action":"completed","workflow_run":{"name":"Build","conclusion":"failure"},"repository":{"name":"vigil"}}'
SIGNATURE=$(echo -n "$PAYLOAD" | openssl dgst -sha256 -hmac "$SECRET" | awk '"'"'{print "sha256="$2}'"'"')
curl -X POST http://localhost:8080/webhooks/github \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature-256: $SIGNATURE" \
  -H "X-GitHub-Event: workflow_run" \
  -d "$PAYLOAD"
```

---

## Conventions

**i18n:** never hardcode strings. Add keys to `client/src/locales/en.json` and `fr.json`, use `t("key")`. Convention: `scope.subscope.element`.

**Pages:** create a folder under `client/src/app/`. Wrap in `<RequireAuth>` for auth, `<AppShell>` for the layout.

**Commits:** one per ticket, format `feat(scope): description (VGL-XXX)`.