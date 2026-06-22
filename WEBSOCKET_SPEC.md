# WEBSOCKET_SPEC.md - VIGIL WebSocket Specification

---

## Connection

### Endpoint

```
GET /ws?token=<session_token>
```

The client opens a single WebSocket connection per session. The session token (the same opaque token used in `Authorization: Bearer` for REST) is passed as a query parameter during the handshake.

### Authentication

The server hashes the received token (SHA-256), looks it up in the `sessions` table, and verifies it is not expired. On failure, the server rejects the upgrade with a **4001** close code and reason `"invalid_token"`. On success, the connection is registered in the broadcaster under the user's ID.

### Implicit Subscription

Once connected, the user automatically receives events for **all teams they belong to** (status `active` in `team_members`). There is no explicit subscribe/unsubscribe mechanism. If a user is kicked or banned, their connection stops receiving events for that team immediately.

### Multi-Tab Support

A user may open multiple tabs or clients simultaneously. Each connection is registered independently in the broadcaster registry (`DashMap<UserId, Vec<Sender>>`). An event targeted at a user is delivered to **all** their active connections.

---

## Message Envelope

All messages (server => client) follow a common JSON envelope:

```json
{
  "type": "<event_type>",
  "team_id": "<uuid>",
  ...payload fields
}
```

- `type` : string identifying the event (see catalog below)
- `team_id` : present on all team-scoped events; absent on bilateral events (private messages)
- Remaining fields are event-specific (documented per event below)

Timestamps are **Unix seconds** (integer) for compactness over the wire. The client converts to local time.

---

## Delivery Modes

Events are delivered through one of three modes, depending on their nature:

| Mode | Description | Example |
|------|-------------|---------|
| **team** | All active members of the team | `incident_state_changed` |
| **targeted** | Team broadcast **+ explicit push** to a specific user | `incident_assigned` |
| **bilateral** | Only the sender and the recipient | `private_message_received` |

The broadcaster exposes two methods, called by services (never by handlers):
- `to_team(team_id, event)` : delivers to all connected members of the team
- `to_user(user_id, event)` : delivers to all connections of a specific user

A **targeted** event calls `to_team` + `to_user` (the assignee may not be viewing the team at that moment).

---

## Reconnection Strategy

Connection drops are expected (network change, laptop sleep, server restart). The client implements:

1. **Exponential backoff** : reconnection attempts at 1s, 2s, 4s, 8s, 16s, capped at 30s
2. **State re-fetch on reconnect** : after a successful reconnection, the client re-fetches the current state of all visible resources via REST (incident list, active release, presence). This avoids stale UI caused by events missed during the disconnection window. The WS connection only carries **deltas**, not full state.
3. **Jitter** : a random factor (±25%) is added to each backoff interval to prevent thundering herd when the server restarts and all clients reconnect simultaneously

The server does **not** replay missed events. The client is responsible for reconciling its state via REST after reconnecting.

---

## Event Catalog

### Phase 1 : Core

#### `incident_state_changed`

An incident transitions to a new lifecycle state.

```json
{
  "type": "incident_state_changed",
  "team_id": "uuid",
  "incident_id": "uuid",
  "new_state": "acknowledged",
  "by": "alice"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `incident_id` | UUID | The affected incident |
| `new_state` | string | One of: `open`, `acknowledged`, `escalated`, `resolved` |
| `by` | string | Display name of the actor |

**Trigger:** Any lifecycle transition (acknowledge, escalate, resolve).
**Delivery:** team
**Note:** An escalation emits **both** `incident_state_changed` (new_state: `escalated`) **and** `incident_escalated` if the severity also changes. The two are independent signals.

---

#### `incident_escalated`

The severity of an incident is raised.

```json
{
  "type": "incident_escalated",
  "team_id": "uuid",
  "incident_id": "uuid",
  "new_severity": "critical",
  "by": "bob"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `incident_id` | UUID | The affected incident |
| `new_severity` | string | One of: `low`, `medium`, `high`, `critical` |
| `by` | string | Display name of the actor |

**Trigger:** Severity change (can accompany an `escalated` state transition or happen independently).
**Delivery:** team
**Note:** Reaching `critical` triggers a native desktop notification (Phase 3).

---

#### `incident_assigned`

A responder is assigned to an incident.

```json
{
  "type": "incident_assigned",
  "team_id": "uuid",
  "incident_id": "uuid",
  "assigned_to": "alice",
  "by": "bob"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `incident_id` | UUID | The affected incident |
| `assigned_to` | string | Display name of the assigned responder |
| `by` | string | Display name of the manager who assigned |

**Trigger:** Manager assigns (or re-assigns) a responder.
**Delivery:** targeted (team broadcast + explicit push to the assignee)
**Note:** Assignment triggers a native desktop notification (Phase 3).

---

#### `timeline_entry_added`

A new entry is added to an incident's timeline.

```json
{
  "type": "timeline_entry_added",
  "team_id": "uuid",
  "incident_id": "uuid",
  "entry": {
    "id": "uuid",
    "content": "Restarted the service, monitoring.",
    "author": "alice",
    "kind": "message",
    "at": 1718000000
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `incident_id` | UUID | The parent incident |
| `entry.id` | UUID | The entry ID (for reactions, editing) |
| `entry.content` | string | The message text |
| `entry.author` | string | Display name of the author |
| `entry.kind` | string | `message` (human) or `system` (auto-generated) |
| `entry.at` | integer | Unix timestamp of creation |

**Trigger:** A responder or manager adds a message, or a system event generates an automatic entry.
**Delivery:** team

---

#### `presence_update`

The list of users currently watching a resource has changed.

```json
{
  "type": "presence_update",
  "team_id": "uuid",
  "resource_type": "incident",
  "resource_id": "uuid",
  "watchers": ["alice", "bob"]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `resource_type` | string | `incident` (core) or `release` (extended) |
| `resource_id` | UUID | The watched resource |
| `watchers` | string[] | Display names of all current watchers |

**Trigger:** A user opens or closes an incident/release detail view. Multi-tab aware: a watcher is only removed when **all** their connections on that resource are closed.
**Delivery:** team

---

### Phase 1 : Extended

#### `release_step_validated`

A step in a release is validated.

```json
{
  "type": "release_step_validated",
  "team_id": "uuid",
  "release_id": "uuid",
  "step": "staging",
  "by": "alice"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `release_id` | UUID | The parent release |
| `step` | string | Name of the validated step |
| `by` | string | Display name of the validator |

**Trigger:** A responder or manager validates a release step.
**Delivery:** team

---

#### `release_state_changed`

A release transitions to a new lifecycle state.

```json
{
  "type": "release_state_changed",
  "team_id": "uuid",
  "release_id": "uuid",
  "new_state": "blocked"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `release_id` | UUID | The affected release |
| `new_state` | string | One of: `created`, `in_progress`, `completed`, `cancelled`, `blocked` |

**Trigger:** Manual transition (start, cancel) or automatic (blocked/unblocked by incident link).
**Delivery:** team
**Note:** `blocked` triggers a native desktop notification (Phase 3).

---

#### `member_kicked`

A member is removed from the team.

```json
{
  "type": "member_kicked",
  "team_id": "uuid",
  "member": "alice",
  "by": "bob"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `member` | string | Display name of the kicked member |
| `by` | string | Display name of the manager |

**Trigger:** Manager kicks a member.
**Delivery:** team (including the kicked member : so their client can react by leaving the team view)

---

#### `member_banned`

A member is banned from the team.

```json
{
  "type": "member_banned",
  "team_id": "uuid",
  "member": "alice",
  "until": 1718000000,
  "by": "bob"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `member` | string | Display name of the banned member |
| `until` | integer or null | Unix timestamp of ban expiry; `null` for permanent bans |
| `by` | string | Display name of the manager |

**Trigger:** Manager bans a member (temporary or permanent).
**Delivery:** team

---

#### `timeline_entry_edited`

The content of a timeline entry is modified by its author.

```json
{
  "type": "timeline_entry_edited",
  "team_id": "uuid",
  "incident_id": "uuid",
  "entry_id": "uuid",
  "new_content": "Updated: restarted both services.",
  "edited_at": 1718000000
}
```

| Field | Type | Description |
|-------|------|-------------|
| `incident_id` | UUID | The parent incident |
| `entry_id` | UUID | The edited entry |
| `new_content` | string | The new content after edit |
| `edited_at` | integer | Unix timestamp of the edit |

**Trigger:** Author edits their own timeline entry.
**Delivery:** team

---

#### `private_message_received`

A direct message is sent between two members.

```json
{
  "type": "private_message_received",
  "from": "alice",
  "to": "bob",
  "message_id": "uuid",
  "content": "Can you check the staging logs?",
  "at": 1718000000
}
```

| Field | Type | Description |
|-------|------|-------------|
| `from` | string | Display name of the sender |
| `to` | string | Display name of the recipient |
| `message_id` | UUID | The message ID |
| `content` | string | Message body |
| `at` | integer | Unix timestamp |

**Trigger:** A user sends a private message.
**Delivery:** bilateral (sender + recipient only : **never** broadcast to the team)

---

#### `reaction_added`

A user reacts to a timeline entry.

```json
{
  "type": "reaction_added",
  "team_id": "uuid",
  "incident_id": "uuid",
  "entry_id": "uuid",
  "emoji": "+1",
  "by": "alice"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `incident_id` | UUID | The parent incident |
| `entry_id` | UUID | The timeline entry reacted to |
| `emoji` | string | One of the server-defined set: `+1`, `-1`, `eyes`, `warning`, `check`, `fire` |
| `by` | string | Display name of the reactor |

**Trigger:** A user adds a reaction.
**Delivery:** team

---

#### `reaction_removed`

A user removes their reaction from a timeline entry.

```json
{
  "type": "reaction_removed",
  "team_id": "uuid",
  "incident_id": "uuid",
  "entry_id": "uuid",
  "emoji": "+1",
  "by": "alice"
}
```

Same structure as `reaction_added`.

**Trigger:** A user removes their own reaction.
**Delivery:** team

---

### Phase 2

#### `rule_triggered`

A rule successfully executed its reaction.

```json
{
  "type": "rule_triggered",
  "team_id": "uuid",
  "rule_name": "CI failure => Incident",
  "result": "incident_created",
  "incident_id": "uuid"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `rule_name` | string | Human-readable name of the rule |
| `result` | string | What the reaction produced (e.g. `incident_created`, `release_blocked`) |
| `incident_id` | UUID or null | If the reaction created an incident, its ID; null otherwise |

**Trigger:** The hook engine successfully executes a rule's reaction.
**Delivery:** team

---

#### `rule_failed`

A rule's reaction failed to execute.

```json
{
  "type": "rule_failed",
  "team_id": "uuid",
  "rule_name": "CI failure => Incident",
  "error": "service_unavailable"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `rule_name` | string | Human-readable name of the rule |
| `error` | string | Error description |

**Trigger:** The hook engine fails to execute a rule's reaction.
**Delivery:** team

---

## Delivery Matrix (Summary)

| Event | Mode | Recipients |
|-------|------|------------|
| `incident_state_changed` | team | All team members |
| `incident_escalated` | team | All team members |
| `incident_assigned` | targeted | All team members + explicit push to assignee |
| `timeline_entry_added` | team | All team members |
| `presence_update` | team | All team members |
| `release_step_validated` | team | All team members |
| `release_state_changed` | team | All team members |
| `member_kicked` | team | All team members (including kicked member) |
| `member_banned` | team | All team members |
| `timeline_entry_edited` | team | All team members |
| `private_message_received` | bilateral | Sender + recipient only |
| `reaction_added` | team | All team members |
| `reaction_removed` | team | All team members |
| `rule_triggered` | team | All team members |
| `rule_failed` | team | All team members |

---

## Desktop Notification Triggers (Phase 3)

Three events trigger native OS notifications when the desktop window is closed:

| Trigger | Source Event |
|---------|-------------|
| User assigned to an incident | `incident_assigned` (if `assigned_to` = current user) |
| Incident reaches critical severity | `incident_escalated` (if `new_severity` = `critical`) |
| Release blocked by an incident | `release_state_changed` (if `new_state` = `blocked`) |

---

## Client => Server Messages

The WebSocket is primarily server => client. The client communicates actions via REST, and the resulting state change is broadcast back via WS. The only client => server WS messages are:

### `presence_join`

```json
{
  "type": "presence_join",
  "resource_type": "incident",
  "resource_id": "uuid"
}
```

Sent when the user opens a resource detail view.

### `presence_leave`

```json
{
  "type": "presence_leave",
  "resource_type": "incident",
  "resource_id": "uuid"
}
```

Sent when the user closes a resource detail view.

### `ping`

```json
{ "type": "ping" }
```

Client-side keepalive. Server responds with `{ "type": "pong" }`.