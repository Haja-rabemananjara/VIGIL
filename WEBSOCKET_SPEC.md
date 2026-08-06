# WebSocket Specification

## Connection

```
GET /ws?token=<session_token>
```

One connection per client. Token validated before upgrade (SHA-256 hash lookup). Invalid token: 401, no upgrade.

Once connected, the user receives events for all their teams automatically. No subscribe/unsubscribe. Multi-tab: each connection registered independently, events delivered to all.

## Message format

```json
{ "type": "<event_type>", "team_id": "<uuid>", ...payload }
```

Timestamps are Unix seconds. User IDs are UUIDs resolved client-side.

## Delivery modes

| Mode | Who receives | Example |
|------|-------------|---------|
| team | All active team members | `incident_state_changed` |
| targeted | Team + explicit push to one user | `incident_assigned` |
| bilateral | Sender + recipient only | `private_message_received` |

Broadcaster API: `to_team(team_id, event)` and `to_user(user_id, event)`. Called by services, never handlers.

## Reconnection

Exponential backoff: 1s, 2s, 4s, 8s, 16s, cap 30s. On reconnect, client re-fetches state via REST. Server does not replay missed events.

## Client to server

Only presence tracking:

```json
{ "type": "watch", "resource_type": "incident", "resource_id": "uuid", "team_id": "uuid" }
{ "type": "unwatch", "resource_type": "incident", "resource_id": "uuid", "team_id": "uuid" }
```

Hard disconnect auto-unwatches. Multi-tab aware (counter per user+resource).

---

## Events

### Incidents

**incident_state_changed** (team)
```json
{ "type": "incident_state_changed", "team_id": "uuid", "incident_id": "uuid", "new_state": "acknowledged", "by": "uuid" }
```
Values: `open`, `acknowledged`, `escalated`, `resolved`. Escalation may emit both this and `incident_escalated`.

**incident_escalated** (team)
```json
{ "type": "incident_escalated", "team_id": "uuid", "incident_id": "uuid", "new_severity": "critical", "by": "uuid" }
```
Only when severity changes during escalation. Desktop notification on `critical`.

**incident_assigned** (targeted)
```json
{ "type": "incident_assigned", "team_id": "uuid", "incident_id": "uuid", "assigned_to": "uuid", "by": "uuid" }
```
Desktop notification when assigned to current user.

### Timeline

**timeline_entry_added** (team)
```json
{ "type": "timeline_entry_added", "team_id": "uuid", "incident_id": "uuid", "entry_id": "uuid", "author_id": "uuid", "content": "...", "at": 1718000000 }
```

**timeline_entry_edited** (team)
```json
{ "type": "timeline_entry_edited", "team_id": "uuid", "incident_id": "uuid", "entry_id": "uuid", "new_content": "...", "edited_at": 1718000000 }
```

**user_typing** (team)
```json
{ "type": "user_typing", "team_id": "uuid", "incident_id": "uuid", "user_id": "uuid" }
```
Throttled 2.5s client-side. Display timeout 3s. No "stopped typing" event.

### Reactions

**reaction_added** / **reaction_removed** (team)
```json
{ "type": "reaction_added", "team_id": "uuid", "incident_id": "uuid", "entry_id": "uuid", "emoji": "+1", "user_id": "uuid" }
```

### Releases

**release_state_changed** (team)
```json
{ "type": "release_state_changed", "team_id": "uuid", "release_id": "uuid", "new_state": "blocked" }
```
Values: `created`, `in_progress`, `completed`, `cancelled`, `blocked`. Desktop notification on `blocked`.

**release_step_validated** (team)
```json
{ "type": "release_step_validated", "team_id": "uuid", "release_id": "uuid", "step_id": "uuid", "step_name": "staging", "by": "uuid" }
```
Last step triggers `release_state_changed` with `completed`.

**release_incident_linked** / **release_incident_unlinked** (team)
```json
{ "type": "release_incident_linked", "team_id": "uuid", "release_id": "uuid", "incident_id": "uuid" }
```

### Members

**member_role_changed** (team)
```json
{ "type": "member_role_changed", "team_id": "uuid", "user_id": "uuid", "new_role": "responder", "by": "uuid" }
```
Manager transfer emits two events.

**member_joined** (team)
```json
{ "type": "member_joined", "team_id": "uuid", "user_id": "uuid", "display_name": "Bob", "role": "observer" }
```

**member_kicked** (team + targeted to kicked user)
```json
{ "type": "member_kicked", "team_id": "uuid", "user_id": "uuid", "by": "uuid" }
```
Targeted via `to_user` since membership is deactivated before broadcast.

**member_banned** (team + targeted to banned user)
```json
{ "type": "member_banned", "team_id": "uuid", "user_id": "uuid", "expires_at": 1718000000, "by": "uuid" }
```
`expires_at`: Unix seconds or `null` (permanent).

### Rules

**rule_triggered** / **rule_failed** (team)
```json
{ "type": "rule_triggered", "team_id": "uuid", "rule_id": "uuid", "rule_name": "CI failure => Incident", "reaction_type": "vigil_create_incident" }
{ "type": "rule_failed", "team_id": "uuid", "rule_id": "uuid", "rule_name": "...", "reaction_type": "discord_message", "error": "Discord returned HTTP 401" }
```

**rule_created** / **rule_updated** / **rule_deleted** (team)
```json
{ "type": "rule_created", "team_id": "uuid", "rule_id": "uuid" }
```

### Presence

**presence_update** (team)
```json
{ "type": "presence_update", "team_id": "uuid", "resource_type": "incident", "resource_id": "uuid", "watchers": ["uuid-1", "uuid-2"] }
```
Complete list, not a delta.

### Messages

**private_message_received** (bilateral)
```json
{ "type": "private_message_received", "from": "uuid", "to": "uuid", "message_id": "uuid", "content": "...", "at": 1718000000 }
```
No `team_id`. Delivered to sender + recipient only.

### System

**connected** (single connection)
```json
{ "type": "connected", "user_id": "uuid" }
```

---

## Delivery matrix

| Event | Mode |
|-------|------|
| `connected` | single |
| `incident_state_changed` | team |
| `incident_escalated` | team |
| `incident_assigned` | targeted |
| `timeline_entry_added` | team |
| `timeline_entry_edited` | team |
| `user_typing` | team |
| `reaction_added` | team |
| `reaction_removed` | team |
| `presence_update` | team |
| `release_state_changed` | team |
| `release_step_validated` | team |
| `release_incident_linked` | team |
| `release_incident_unlinked` | team |
| `member_role_changed` | team |
| `member_joined` | team |
| `member_kicked` | team + targeted |
| `member_banned` | team + targeted |
| `rule_triggered` | team |
| `rule_failed` | team |
| `rule_created` | team |
| `rule_updated` | team |
| `rule_deleted` | team |
| `private_message_received` | bilateral |

## Desktop notifications

| Trigger | Event | Condition |
|---------|-------|-----------|
| Assigned to incident | `incident_assigned` | `assigned_to` = current user |
| Critical severity | `incident_escalated` | `new_severity` = `critical` |
| Release state change | `release_state_changed` | all except `created` |
| DM received | `private_message_received` | `from` != current user |
| Promoted to Manager | `member_role_changed` | `new_role` = `manager` |
| Rule executed | `rule_triggered` | always |
| Rule failed | `rule_failed` | always |

Desktop-only via `notify-send`.