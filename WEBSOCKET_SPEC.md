# WebSocket specification

---

## Connection

### Endpoint

```
GET /ws?token=<session_token>
```
One WebSocket connection per session. The session token (same as the `Authorization: Bearer` token for REST) is passed as a query parameter.

### Authentication

Authentication happens before the WebSocket upgrade. The server decodes the hex token, hashes it with SHA-256, and looks up the matching session. On failure: HTTP 401, no upgrade. On success: the connection is registered in the broadcaster under the user's ID.

### Implicit subscription

Once connected, the user receives events for all teams they belong to (status `active` in `team_members`). No explicit subscribe/unsubscribe. If a user is kicked or banned, their connection stops receiving events for that team immediately.

### Multi-tab support

Each connection is registered independently in the broadcaster (`DashMap<UserId, Vec<Sender>>`). An event targeted at a user is delivered to all their active connections.

---

## Message envelope

All messages (server to client) follow a common JSON shape:

```json
{
  "type": "<event_type>",
  "team_id": "<uuid>",
  ...payload fields
}
```

- `type`: string identifying the event
- `team_id`: present on all team-scoped events, absent on bilateral events
- Timestamps are Unix seconds (integer)
- User identifiers (`by`, `assigned_to`, `author_id`, `watchers`) are UUIDs. The client resolves them to display names locally

---

## Delivery modes

| Mode | Description | Example |
|------|-------------|---------|
| **team** | All active members of the team | `incident_state_changed` |
| **targeted** | Team broadcast + explicit push to a specific user | `incident_assigned` |
| **bilateral** | Only the sender and the recipient | `private_message_received` |

The broadcaster exposes two methods, called by services (never handlers):
- `to_team(team_id, event)`: delivers to all connected members
- `to_user(user_id, event)`: delivers to all connections of a specific user

A targeted event calls both.

---

## Reconnection strategy

Connection drops are expected (network change, sleep, restart). The client implements:

1. **Exponential backoff**: 1s, 2s, 4s, 8s, 16s, capped at 30s
2. **State re-fetch on reconnect**: a `reconnectCount` state variable increments on successful reconnection. Pages observing it re-fetch their data via REST to catch up on missed events

The server does not replay missed events. The WS carries deltas; REST carries full state.

---

## Client to server messages

The WebSocket is primarily server to client. Actions go via REST; state changes are broadcast back via WS. The only client-to-server messages are for presence tracking.

### `watch`

```json
{
  "type": "watch",
  "resource_type": "incident",
  "resource_id": "uuid",
  "team_id": "uuid"
}
```

Sent when the user opens a resource detail page. `team_id` is included so the server can broadcast the `presence_update` without an extra lookup.

### `unwatch`

```json
{
  "type": "unwatch",
  "resource_type": "incident",
  "resource_id": "uuid",
  "team_id": "uuid"
}
```

Sent on component unmount. On hard disconnect (tab crash, network loss), the server automatically unwatches all resources for that connection.

The server tracks a count per `(user, resource)`. A user watching from 3 tabs appears once in `watchers` but is only removed when all 3 connections close.

---

## Implemented events

### `connected`

Sent once after a successful handshake. Not broadcast.

```json
{ "type": "connected", "user_id": "uuid" }
```

**Trigger:** Successful token validation during HTTP upgrade.
**Delivery:** single connection only.

---

### `incident_state_changed`

```json
{
  "type": "incident_state_changed",
  "team_id": "uuid",
  "incident_id": "uuid",
  "new_state": "acknowledged",
  "by": "uuid"
}
```

| Field | Type | Values |
|-------|------|--------|
| `new_state` | string | `open`, `acknowledged`, `escalated`, `resolved` |
| `by` | UUID | User who triggered the transition |

**Trigger:** Any lifecycle transition, incident creation, or rule engine reaction.
**Delivery:** team.
**Note:** An escalation emits both `incident_state_changed` and `incident_escalated` if severity also changes.

---

### `incident_escalated`

```json
{
  "type": "incident_escalated",
  "team_id": "uuid",
  "incident_id": "uuid",
  "new_severity": "critical",
  "by": "uuid"
}
```

| Field | Type | Values |
|-------|------|--------|
| `new_severity` | string | `low`, `medium`, `high`, `critical` |

**Trigger:** `PATCH .../status` with `{ "status": "escalated", "severity": "..." }`. Only emitted when severity is provided during escalation. A standalone `PATCH .../severity` does not emit this event.
**Delivery:** team.
**Desktop notification:** triggers when `new_severity` is `critical`.

---

### `incident_assigned`

```json
{
  "type": "incident_assigned",
  "team_id": "uuid",
  "incident_id": "uuid",
  "assigned_to": "uuid",
  "by": "uuid"
}
```

**Trigger:** Manager assigns or re-assigns a responder.
**Delivery:** targeted (team + explicit push to assignee).
**Desktop notification:** triggers when `assigned_to` matches the current user.

---

### `timeline_entry_added`

```json
{
  "type": "timeline_entry_added",
  "team_id": "uuid",
  "incident_id": "uuid",
  "entry_id": "uuid",
  "author_id": "uuid",
  "content": "Restarted the service, monitoring.",
  "at": 1718000000
}
```

**Trigger:** Responder or manager adds a message via `POST .../timeline`.
**Delivery:** team.
**Note:** System-generated entries (from state transitions) do not emit this event. They are fetched via a timeline reload triggered by `incident_state_changed`.

---

### `presence_update`

```json
{
  "type": "presence_update",
  "team_id": "uuid",
  "resource_type": "incident",
  "resource_id": "uuid",
  "watchers": ["uuid-1", "uuid-2"]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `resource_type` | string | `incident` or `release` |
| `watchers` | UUID[] | Complete list, not a delta |

**Trigger:** User opens or closes a detail view. Multi-tab aware.
**Delivery:** team.

---

### `member_role_changed`

```json
{
  "type": "member_role_changed",
  "team_id": "uuid",
  "user_id": "uuid",
  "new_role": "responder",
  "by": "uuid"
}
```

| Field | Type | Values |
|-------|------|--------|
| `new_role` | string | `observer`, `responder`, `manager` |

**Trigger:** Role change (`PATCH .../role`) or manager transfer (emits two events: former manager becomes `responder`, new manager becomes `manager`).
**Delivery:** team.

---

### `release_state_changed`

```json
{
  "type": "release_state_changed",
  "team_id": "uuid",
  "release_id": "uuid",
  "new_state": "blocked"
}
```

| Field | Type | Values |
|-------|------|--------|
| `new_state` | string | `created`, `in_progress`, `completed`, `cancelled`, `blocked` |

**Trigger:** Manual (creation, start, cancel) or automatic (blocked by incident link, unblocked when last incident resolved/unlinked, auto-completed on last step validated).
**Delivery:** team.
**Desktop notification:** triggers when `new_state` is `blocked`.

---

### `release_step_validated`

```json
{
  "type": "release_step_validated",
  "team_id": "uuid",
  "release_id": "uuid",
  "step_id": "uuid",
  "step_name": "staging",
  "by": "uuid"
}
```

**Trigger:** `POST .../steps/{step_id}/validate`.
**Delivery:** team.
**Note:** When the last step is validated, a `release_state_changed` (new_state: `completed`) follows immediately.

---

### `release_incident_linked`

```json
{
  "type": "release_incident_linked",
  "team_id": "uuid",
  "release_id": "uuid",
  "incident_id": "uuid"
}
```

**Trigger:** `POST .../releases/{id}/link`. Emitted regardless of release state (the auto-block `release_state_changed` is emitted separately if applicable).
**Delivery:** team.

---

### `release_incident_unlinked`

```json
{
  "type": "release_incident_unlinked",
  "team_id": "uuid",
  "release_id": "uuid",
  "incident_id": "uuid"
}
```

**Trigger:** `POST .../releases/{id}/unlink`. The auto-unblock `release_state_changed` is emitted separately if applicable.
**Delivery:** team.

---

### `rule_triggered`

```json
{
  "type": "rule_triggered",
  "team_id": "uuid",
  "rule_id": "uuid",
  "rule_name": "CI failure => Incident",
  "reaction_type": "vigil_create_incident"
}
```

**Trigger:** The engine successfully executes a rule's reaction.
**Delivery:** team.

---

### `rule_failed`

```json
{
  "type": "rule_failed",
  "team_id": "uuid",
  "rule_id": "uuid",
  "rule_name": "CI failure => Incident",
  "reaction_type": "discord_message",
  "error": "Discord returned HTTP 401"
}
```

**Trigger:** The engine fails to execute a reaction. Each rule runs in isolation; a failure in one never affects others.
**Delivery:** team.

### `rule_created`

```json
{ "type": "rule_created", "team_id": "uuid", "rule_id": "uuid" }
```

**Trigger:** Manager creates a rule.
**Delivery:** team.

### `rule_updated`

```json
{ "type": "rule_updated", "team_id": "uuid", "rule_id": "uuid" }
```

**Trigger:** Manager updates a rule.
**Delivery:** team.

### `rule_deleted`

```json
{ "type": "rule_deleted", "team_id": "uuid", "rule_id": "uuid" }
```

**Trigger:** Manager deletes a rule.
**Delivery:** team.

### `member_kicked`

```json
{
  "type": "member_kicked",
  "team_id": "uuid",
  "user_id": "uuid",
  "by": "uuid"
}
```

**Trigger:** Manager kicks a member via `POST /teams/{team_id}/members/{user_id}/kick`.
**Delivery:** team + targeted to the kicked user (via `to_user`, since their membership is already deactivated when the broadcast fires).

### `member_banned`

```json
{
  "type": "member_banned",
  "team_id": "uuid",
  "user_id": "uuid",
  "expires_at": 1718000000,
  "by": "uuid"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `expires_at` | int or null | Unix seconds. `null` for permanent bans |

**Trigger:** Manager bans a member via `POST /teams/{team_id}/members/{user_id}/ban`.
**Delivery:** team + targeted to the banned user (via `to_user`).

### `member_joined`

```json
{ 
  "type": "member_joined",
  "team_id": "uuid",
  "user_id": "uuid",
  "display_name": "Bob",
  "role": "observer"
}
```
- **Trigger**: User joins a team via `POST /teams/join`
- **Recipients**: All team members

### `timeline_entry_edited`

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

**Trigger:** Author edits their own timeline entry via `PATCH /timeline/{entry_id}`.
**Delivery:** team.

---

### `reaction_added`

```json
{
  "type": "reaction_added",
  "team_id": "uuid",
  "incident_id": "uuid",
  "entry_id": "uuid",
  "emoji": "+1",
  "user_id": "uuid"
}
```

**Trigger:** User adds a reaction via `POST /timeline/{entry_id}/reactions`.
**Delivery:** team.

---

### `reaction_removed`

```json
{
  "type": "reaction_removed",
  "team_id": "uuid",
  "incident_id": "uuid",
  "entry_id": "uuid",
  "emoji": "+1",
  "user_id": "uuid"
}
```

**Trigger:** User removes their reaction via `DELETE /timeline/{entry_id}/reactions/{emoji}`.
**Delivery:** team.

---

### `private_message_received`

```json
{
  "type": "private_message_received",
  "from": "uuid",
  "to": "uuid",
  "message_id": "uuid",
  "content": "Can you check the staging logs?",
  "at": 1718000000
}
```

**Trigger:** User sends a DM via `POST /messages/{user_id}`.
**Delivery:** bilateral (sender + recipient only, via two `to_user` calls). No `team_id` field.

---

## Delivery matrix

| Event | Mode | Implemented |
|-------|------|-------------|
| `connected` | single | yes |
| `incident_state_changed` | team | yes |
| `incident_escalated` | team | yes |
| `incident_assigned` | targeted | yes |
| `timeline_entry_added` | team | yes |
| `presence_update` | team | yes |
| `member_role_changed` | team | yes |
| `release_state_changed` | team | yes |
| `release_step_validated` | team | yes |
| `release_incident_linked` | team | yes |
| `release_incident_unlinked` | team | yes |
| `rule_triggered` | team | yes |
| `rule_failed` | team | yes |
| `member_kicked` | team + targeted | yes |
| `member_banned` | team + targeted | yes |
| `timeline_entry_edited` | team | yes |
| `private_message_received` | bilateral | yes |
| `reaction_added` | team | yes |
| `reaction_removed` | team | yes |
| `rule_created` | team | yes |
| `rule_updated` | team | yes |
| `rule_deleted` | team | yes |

---

## Desktop notification triggers

| Trigger | Source event | Condition |
|---------|-------------|-----------|
| User assigned to an incident | `incident_assigned` | `assigned_to` = current user |
| Incident reaches critical | `incident_escalated` | `new_severity` = `critical` |
| Release state changed | `release_state_changed` | all states except `created` |
| Private message received | `private_message_received` | `from` ≠ current user |
| Promoted to Manager | `member_role_changed` | `user_id` = current user, `new_role` = `manager` |
| Rule executed | `rule_triggered` | always |
| Rule failed | `rule_failed` | always |

Notifications are desktop-only, dispatched via `notify-send` through the embedded HTTP server. The web client does not emit notifications.