# How to contribute

How to extend VIGIL's rule engine and real-time infrastructure. Each
section is a self-contained recipe: follow the steps, run the tests,
ship.

The engine is built on the Open/Closed principle: adding a new Action,
Reaction, or WebSocket event never modifies the engine core, the
broadcaster, or any existing extension. You add files at the edge.

---

## Adding a new Action (incoming event type)

An Action declares that a service can send us a particular event. It is
pure metadata with no runtime behavior. The engine matches incoming
webhooks against registered Actions; the `ActionCatalog` also feeds
`/about.json` so clients discover available triggers without hard-coding
anything.

**Example: register GitLab's `pipeline` event.**

### 1. Register in `server/src/main.rs`

Find the `ActionCatalog::builder()` block and add one line:

```rust
let action_catalog = ActionCatalog::builder()
    .register(
        "github",
        "workflow_run",
        "A CI workflow run has completed (success or failure)",
    )
    // ... existing entries ...
    .register(
        "gitlab",
        "pipeline",
        "A GitLab CI/CD pipeline has finished",
    )
    .build();
```

### 2. Register in `server/tests/common/mod.rs`

The test harness builds its own catalog. Add the same line there so
integration tests can create rules targeting this event.

### 3. Verify

```bash
cargo test --test about_e2e
```

`GET /about.json` now lists `gitlab` with one action. The rule form in
the web client picks it up on the next page load.

**Files modified: 2** (`main.rs`, `tests/common/mod.rs`).

---

## Adding a new REAction (outgoing behavior)

A Reaction is a type that implements the `ReactionExecutor` trait. The
engine calls `execute()` when a matching rule fires. Each Reaction lives
in its own file under `server/src/hooks/reactions/`.

**Example: add a `slack_message` reaction.**

### 1. Create `server/src/hooks/reactions/slack_message.rs`

Implement the `ReactionExecutor` trait. The four metadata methods
(`kind`, `service_name`, `description`, `payload_example`) feed
`/about.json` automatically. The `execute` method contains the runtime
behavior.

```rust
use async_trait::async_trait;
use serde::Deserialize;

use crate::crypto;
use crate::domain::service_connections::ServiceName;
use crate::error::AppError;
use crate::hooks::{ReactionContext, ReactionExecutor};
use crate::repo;

pub struct SlackMessage;

impl SlackMessage {
    pub fn new() -> Self { Self }
}

impl Default for SlackMessage {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Deserialize)]
struct SlackPayload {
    text: String,
}

#[async_trait]
impl ReactionExecutor for SlackMessage {
    fn kind(&self) -> &'static str { "slack_message" }
    fn service_name(&self) -> &'static str { "slack" }
    fn description(&self) -> &'static str { "Post a message to a Slack channel via webhook" }
    fn payload_example(&self) -> &'static str {
        r#"{
  "text": "CI broken on {{repository.name}}"
}"#
    }

    async fn execute(&self, ctx: &ReactionContext<'_>) -> Result<(), AppError> {
        let payload: SlackPayload =
            serde_json::from_value(ctx.payload.clone()).map_err(|e| {
                AppError::Validation(format!("Invalid slack_message payload: {e}"))
            })?;

        let connection = repo::service_connections::find_with_token(
            ctx.pool,
            ctx.rule_created_by,
            ServiceName::Slack,
        )
        .await?
        .ok_or_else(|| {
            AppError::Validation("The rule creator has no Slack connection configured".into())
        })?;

        let webhook_bytes = crypto::decrypt(ctx.master_key, &connection.encrypted_token)?;
        let webhook_url = std::str::from_utf8(&webhook_bytes)
            .map_err(|_| AppError::Internal("Invalid UTF-8 in Slack URL".into()))?;

        let body = serde_json::json!({ "text": payload.text });
        let response = ctx.http_client.post(webhook_url).json(&body).send().await
            .map_err(|e| AppError::Internal(format!("Slack request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(AppError::Internal(
                format!("Slack returned HTTP {}", response.status()),
            ));
        }

        Ok(())
    }
}
```

### 2. Export from `server/src/hooks/reactions/mod.rs`

```rust
pub mod slack_message;
pub use slack_message::SlackMessage;
```

### 3. Register in `server/src/main.rs`

In the `ReactionRegistry::builder()` block:

```rust
.register(Arc::new(SlackMessage::new()))
```

### 4. Register in `server/tests/common/mod.rs`

Same line in the test harness builder.

### 5. If the service is new: add to `ServiceName`

If `slack` is not yet in the `ServiceName` enum
(`server/src/domain/service_connections.rs`), add the variant, update
`from_db`, `as_str`, and the `ALL` array. Then add a migration:

```sql
ALTER TABLE service_connections
  DROP CONSTRAINT service_connections_service_check,
  ADD CONSTRAINT service_connections_service_check
    CHECK (service IN ('github', 'gitlab', 'discord', 'slack'));
```

### 6. Write a test

Create `server/tests/slack_reaction_e2e.rs` using `wiremock` to mock
the Slack endpoint, following the pattern in
`tests/discord_reaction_e2e.rs`. Three tests to write:

- Happy path: rule matches, Slack receives the POST with the right body
- No connection: rule creator has no Slack connection, `rule_failed`
- Slack returns 500: server stays healthy, `rule_failed`

### 7. Verify

```bash
cargo test --lib hooks
cargo test --test slack_reaction_e2e
cargo test --test about_e2e
```

`/about.json` now lists `slack` with one reaction, its description, and
its `payload_example`. The rule form picks it up on the next page load.

**Files modified: 4** (new reaction file, `mod.rs`, `main.rs`,
`tests/common/mod.rs`) **+ 1 migration** if the service is new.

---

## Adding a new WebSocket event

Events are delivered by the broadcaster, which is transport-only. The
broadcaster never decides what to send or to whom -- services decide.

### 1. Add a variant to `WsEvent`

In `server/src/ws/events.rs`:

```rust
DeploymentStarted {
    team_id: Uuid,
    release_id: Uuid,
    environment: String,
},
```

Serde's `rename_all = "snake_case"` on the enum handles the wire name.

### 2. Emit it from a service

In the relevant service function (not a handler):

```rust
broadcaster
    .to_team(
        team_id,
        WsEvent::DeploymentStarted {
            team_id,
            release_id,
            environment: environment.to_string(),
        },
    )
    .await;
```

You never touch `ws/broadcaster.rs`.

### 3. Handle it in the client

In the component that cares, consume `lastEvent` from `useVigilSocket()`:

```tsx
useEffect(() => {
    if (!lastEvent) return;
    if (lastEvent.type !== "deployment_started") return;
    if (lastEvent.team_id !== teamId) return;

    setDeployments((prev) => [...prev, {
        releaseId: lastEvent.release_id as string,
        environment: lastEvent.environment as string,
        at: Date.now(),
    }]);
}, [lastEvent, teamId]);
```

Two rules:

- Never call a fetch function inside a WS handler. Use
  `setState((prev) => ...)` with the data already in the event.
- Exception: when a WS event signals a new entity whose full shape
  isn't in the payload (e.g. `newState === "created"`). One targeted
  fetch is acceptable.

### 4. Document it

Add the event to `WEBSOCKET_SPEC.md` with its payload, trigger, and
delivery mode.

**Files modified: 3** (`events.rs`, the emitting service, the consuming
component) **+ 1 doc update**.

---

## Adding a complete new service (end-to-end)

Combining the recipes above. Example: adding **Timer** (an internal
cron service that triggers rules at fixed intervals).

### Checklist

1. **Action**: register `("timer", "cron", "Fires at a configured interval")`
   in the `ActionCatalog` builder in `main.rs` and `tests/common/mod.rs`.

2. **Webhook receiver or internal scheduler**: Timer has no external
   webhook. Add an internal scheduler (e.g. `tokio::time::interval`)
   that calls `hooks::engine::evaluate()` directly. The engine does not
   care where the event comes from.

3. **Reaction** (optional): Timer as described is trigger-only. No new
   `ReactionExecutor` needed unless Timer also acts.

4. **`ServiceName` variant**: only if connectable. Timer is internal, skip.

5. **Tests**: an integration test that starts the scheduler, waits for a
   tick, and verifies a rule fired.

6. **Documentation**: update `README.md` (registered actions table) and
   `WEBSOCKET_SPEC.md` if new events are added.

### Cost estimate

Trigger-only service: **2 files modified** (`main.rs` + test harness) +
**1 new file** (scheduler or receiver).

Service that also reacts: add the Reaction recipe on top (**+2 files**).

The engine, the matcher, the templating, the broadcaster, the frontend:
untouched.

---

## Rule engine structure

For reference, the engine lives under `server/src/hooks/`:
hooks/
actions.rs ActionCatalog (metadata, no trait)
registry.rs ReactionRegistry + ReactionExecutor trait
engine.rs EngineContext, evaluate(), evaluate_one()
matcher.rs Dot-notation filter matching
templating.rs {{path.to.field}} rendering
context.rs ReactionContext (per-reaction parameter object)
reactions/
mod.rs Exports all reaction types
vigil_create_incident.rs
vigil_escalate_incident.rs
vigil_block_release.rs
vigil_validate_release_step.rs
discord_message.rs

The engine calls `registry.get(kind)` and gets back an
`Arc<dyn ReactionExecutor>`. The broadcaster serializes whatever
`WsEvent` variant it receives. Both extension points grow at the edge.

Full codebase navigation is in the README.

---

## General conventions

### Adding a user-facing string

Never hardcode text in a component. Add a key to the `en` dictionary in
`client/src/lib/i18n.ts` (convention: `scope.subscope.element`, e.g.
`auth.signin.title`) and read it with `t("your.key")`. A missing key
renders as the key itself, making the omission visible. The FR dictionary
plugs in as a second dictionary with no component changes.

### Adding a protected page

Create a folder under `client/src/app/` (the folder name is the route).
Wrap the page content in `<RequireAuth>` to enforce authentication, and
in `<AppShell>` if it should render inside the app layout. Read auth
state with `useAuth()`; never read the token from `localStorage`
directly.