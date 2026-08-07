# UI Guidelines

VIGIL is an operational control room. The interface must be readable at a glance, consistent across screens, and usable by keyboard and color-blind users.

---

## Accessibility

VIGIL targets **WCAG 2.1 AA** as a reference.

- Keyboard navigation on all primary actions
- Explicit labels on every form field (never placeholder-only)
- Color is never the only signal: every state and severity uses **color + icon + text**

Verified by keyboard-only walkthrough and grayscale inspection of badges.

---

## Color palette

Defined as CSS variables in `globals.css`, consumed via Tailwind. No hardcoded hex values.

| Role | Token | Usage |
|------|-------|-------|
| Primary | `--primary` | Submit, confirm, acknowledged/in-progress states |
| Success | `--success` | Resolved, completed |
| Warning | `--warning` | Escalated, high severity |
| Danger | `--destructive` | Critical severity, destructive actions only |
| Neutral | `--muted` | Open, low severity, secondary text |

Red is reserved for the most severe case in each axis. Overusing it destroys its signal.

---

## State mappings

### Incident states (`StateBadge`)

| State | Color | Icon | Label |
|-------|-------|------|-------|
| `open` | Gray | AlertCircle | Open |
| `acknowledged` | Primary | Clock | Acknowledged |
| `escalated` | Amber | AlertTriangle | Escalated |
| `resolved` | Green | CheckCircle2 | Resolved |

### Severity levels (`SeverityBadge`)

| Severity | Color | Icon | Label |
|----------|-------|------|-------|
| `low` | Gray | ChevronDown | Low |
| `medium` | Primary | Equal | Medium |
| `high` | Amber | ChevronUp | High |
| `critical` | Red | Flame | Critical |

### Release states (`ReleaseStateBadge`)

| State | Color | Icon | Label |
|-------|-------|------|-------|
| `created` | Gray | Circle | Created |
| `in_progress` | Primary | Play | In Progress |
| `completed` | Green | CheckCircle2 | Completed |
| `cancelled` | Gray | XCircle | Cancelled |
| `blocked` | Red | ShieldAlert | Blocked |

---

## Components

| Component | Purpose |
|-----------|---------|
| `StateBadge` | Incident state (color + icon + text) |
| `SeverityBadge` | Severity level (color + icon + text) |
| `ReleaseStateBadge` | Release state (color + icon + text) |
| `ConfirmDialog` | Confirmation for destructive actions |
| `AppShell` | Layout: header + sidebar + content |
| `UserMenu` | Identity, language toggle, sign out |
| `UserAvatar` | DiceBear avatar with initials fallback |
| `ConnectionIndicator` | WebSocket status (color + icon + text) |

Built on shadcn/ui (Radix-based) primitives.

---

## Dark patterns avoided

- All destructive actions go through `ConfirmDialog` naming the affected resource
- Confirm = confirm, cancel = safe default. No inversion
- Sign out and critical options in predictable locations (header menu)
- Audit log is read-only. Moderation does not rewrite the past

---

## i18n

French and English. Dictionaries in `client/src/locales/{en,fr}.json`. `t()` typed with `TranslationKey` (compile-time safety). Language changeable from profile page, header menu, or signin page. Dates localized via `getLanguage()`.

---

## Screenshots

Located in `docs/screenshots/`:

1. **Incident detail** -- StateBadge, SeverityBadge, timeline, watchers, reactions, role-contextual actions
2. **Release detail** -- ReleaseStateBadge, stepper, blocked banner, confirmation dialog

Both demonstrate the three-signal rule (color + icon + text) and the confirmation dialog pattern.