# HOWTOCONTRIBUTE.md

How to extend VIGIL's rule engine and real-time infrastructure. Each
section is a self-contained recipe: follow the steps, run the tests,
ship.

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
integration tests can create rules targeting this event:

```rust
let action_catalog = server::hooks::ActionCatalog::builder()
    // ... existing entries ...
    .register(
        "gitlab",
        "pipeline",
        "A GitLab CI/CD pipeline has finished",
    )
    .build();
```

### 3. Verify

```bash
cargo test --test about_e2e
```

The test `about_lists_registered_github_actions` won't check GitLab
(it's scoped to GitHub), but `every_action_exposes_a_valid_json_filters_example`
will exercise the new entry if you added a filters example. You can add
a targeted assertion if needed.

`GET /about.json` now lists `gitlab` with one action. The rule form in
the web client picks it up on the next page load.

### What you did NOT touch

- The engine (`hooks/engine.rs`)
- The matcher or templating
- The broadcaster
- Any existing Action or Reaction
- Any frontend code

**Files modified: 2.** `main.rs` and `tests/common/mod.rs`.

---

## Adding a new REAction (outgoing behavior)

A Reaction is a type that implements the `ReactionExecutor` trait. The
engine calls `execute()` when a matching rule fires. Each Reaction lives
in its own file under `server/src/hooks/reactions/`.

**Example: add a `slack_message` reaction.**

### 1. Create `server/src/hooks/reactions/slack_message.rs`

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
    fn kind(&self) -> &'static str {
        "slack_message"
    }

    fn service_name(&self) -> &'static str {
        "slack"
    }

    fn description(&self) -> &'static str {
        "Post a message to a Slack channel via webhook"
    }

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

        // Fetch the rule creator's Slack connection (encrypted).
        let connection = repo::service_connections::find_with_token(
            ctx.pool,
            ctx.rule_created_by,
            ServiceName::Slack, // requires adding Slack to the enum
        )
        .await?
        .ok_or_else(|| {
            AppError::Validation(
                "The rule creator has no Slack connection configured".into(),
            )
        })?;

        let webhook_bytes =
            crypto::decrypt(ctx.master_key, &connection.encrypted_token)?;
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

Same line in the test harness builder:

```rust
.register(std::sync::Arc::new(
    server::hooks::reactions::SlackMessage::new(),
))
```

### 5. If the service is new: add to `ServiceName`

If `slack` is not yet in the `ServiceName` enum
(`server/src/domain/service_connections.rs`), add the variant and update
the `CHECK` constraint in a new migration:

```rust
pub enum ServiceName {
    Github,
    Gitlab,
    Discord,
    Slack,   // new
}

impl ServiceName {
    pub const ALL: [ServiceName; 4] = [
        ServiceName::Github,
        ServiceName::Gitlab,
        ServiceName::Discord,
        ServiceName::Slack,
    ];
    // ...
    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "github" => Some(Self::Github),
            "gitlab" => Some(Self::Gitlab),
            "discord" => Some(Self::Discord),
            "slack" => Some(Self::Slack),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Github => "github",
            Self::Gitlab => "gitlab",
            Self::Discord => "discord",
            Self::Slack => "slack",
        }
    }
}
```

Migration:

```sql
ALTER TABLE service_connections
  DROP CONSTRAINT service_connections_service_check,
  ADD CONSTRAINT service_connections_service_check
    CHECK (service IN ('github', 'gitlab', 'discord', 'slack'));
```

### 6. Write a test

Create `server/tests/slack_reaction_e2e.rs` using `wiremock` to mock
the Slack endpoint, following the pattern in
`tests/discord_reaction_e2e.rs`. The three tests to write:

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
its `payload_example`. The rule form in the web client shows it on the
next page load.

### What you did NOT touch

- The engine (`hooks/engine.rs`)
- The matcher, templating, or `EngineContext`
- The broadcaster
- Any existing Reaction
- Any frontend code

**Files modified: 4** (new reaction file, `mod.rs`, `main.rs`,
`tests/common/mod.rs`) **+ 1 migration** if the service is new.

---

## Adding a new WebSocket event

Events are delivered by the broadcaster, which is transport-only. The
broadcaster never decides what to send or to whom. Services decide.

### 1. Add a variant to `WsEvent`

In `server/src/ws/events.rs`, add the variant to the `WsEvent` enum:

```rust
#[serde(rename = "deployment_started")]
DeploymentStarted {
    team_id: Uuid,
    release_id: Uuid,
    environment: String,
},
```

The `#[serde(rename = "...")]` controls the `type` field in the JSON
envelope. Rust naming convention on the variant, wire naming convention
on the rename.

### 2. Emit it from a service

In the relevant service function (not a handler), call the broadcaster:

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

The broadcaster serializes and delivers. You never touch
`ws/broadcaster.rs`.

### 3. Handle it in the client

In the component that cares about this event, consume `lastEvent` from
`useVigilSocket()`:

```tsx
useEffect(() => {
    if (!lastEvent) return;
    if (lastEvent.type !== "deployment_started") return;
    if (lastEvent.team_id !== teamId) return;

    // Update local state inline, never fetch inside a WS handler.
    setDeployments((prev) => [...prev, {
        releaseId: lastEvent.release_id as string,
        environment: lastEvent.environment as string,
        at: Date.now(),
    }]);
}, [lastEvent, teamId]);
```

Rules:

- **Never call a fetch function inside a WS handler.** Use
  `setState((prev) => ...)` with the data already in the event.
- The only exception: when a WS event signals the creation of a new
  entity whose full shape isn't in the event payload (e.g.
  `newState === "created"` on a resource). In that case, one targeted
  fetch is acceptable.

### 4. Document it

Add the event to `WEBSOCKET_SPEC.md` with its payload, trigger, and
delivery mode. Add it to the delivery matrix table at the bottom.

### What you did NOT touch

- The broadcaster (`ws/broadcaster.rs`)
- Any existing event variant
- The socket provider (`stores/socket.tsx`)
- The reconnection logic

**Files modified: 3** (`events.rs`, the emitting service, the consuming
component) **+ 1 doc update**.

---

## Adding a complete new service (end-to-end)

Combining the recipes above. Say you want to add **Timer** (an internal
cron service that triggers rules at fixed intervals).

### Checklist

1. **Action** (what Timer can send us):
   Register `("timer", "cron", "Fires at a configured interval")` in the
   `ActionCatalog` builder in `main.rs` and `tests/common/mod.rs`.

2. **Webhook receiver or internal scheduler**:
   Timer has no external webhook. You would add an internal scheduler
   (e.g. `tokio::time::interval`) that calls
   `hooks::engine::evaluate(ctx, "timer", "cron", &payload, delivery_id)`
   directly. The engine does not care where the event comes from. This
   is the only step that differs from an external service like GitHub.

3. **Reaction** (optional, if Timer also does something):
   Timer as described is trigger-only. No new `ReactionExecutor` needed
   unless Timer also acts (e.g. "schedule a reminder"). If needed,
   follow the Reaction recipe above.

4. **`ServiceName` variant** (only if connectable):
   Timer is internal, nothing to connect. Skip.

5. **Tests**:
   An integration test that starts the scheduler, waits for a tick, and
   verifies a rule fired. Use `wait_until(|| ...)` with a 5s deadline.

6. **Documentation**:
   - `HOWTOCONTRIBUTE.md`: already covered (it's this file)
   - `WEBSOCKET_SPEC.md`: no new events (Timer uses existing
     `rule_triggered` / `rule_failed`)
   - `README.md`: add Timer to the "Registered actions" table

### Cost estimate

For a service that only triggers (like Timer): **2 files modified**
(`main.rs` + test harness) + **1 new file** (the scheduler or receiver).

For a service that also reacts (like Slack): add the Reaction recipe on
top: **+2 files** (reaction + `mod.rs`).

The engine, the matcher, the templating, the broadcaster, the frontend:
untouched. That is the Open/Closed guarantee.

---

## Architecture summary for contributors

server/src/hooks/
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
ws/
events.rs WsEvent enum (serde-tagged)
broadcaster.rs to_team() / to_user() (transport only)

The engine never knows the concrete reaction types. It calls
`registry.get(kind)` and gets back an `Arc<dyn ReactionExecutor>`. The
broadcaster never knows the concrete event types. It serializes whatever
`WsEvent` variant it receives. Both extension points grow at the edge.