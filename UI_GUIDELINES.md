# UI Guidelines

VIGIL is an operational control room. The interface must let an operator read a situation, stay consistent across every screen, and be usable by keyboard and by color-blind users. This document is the contract the jury uses to evaluate those requirements.

---

## Targeted accessibility level

VIGIL targets **WCAG 2.1 AA** as a guiding reference.

- **Keyboard navigation** on all primary actions.
- **Explicit labels** on every form field (never placeholder-only).
- **Color is never the only signal**: every state and severity is conveyed by
  **color + icon + text** simultaneously, so meaning survives for color-blind
  users and in grayscale.

Verification method: keyboard-only walkthrough of each primary flow, and visual inspection of badges in grayscale to confirm icon + text remain sufficient.

---

## Color palette

Colors are defined as design tokens (CSS variables in `client/src/app/globals.css`) and consumed through Tailwind utility classes.
No component hardcodes a hex value, changing a token updates the whole app.

| Role | Token | Usage |
|------|-------|-------|
| **Primary** | `--primary` | Primary actions (submit, confirm), in-progress / acknowledged states |
| **Success** | `--success` | Resolved incidents, completed steps, positive outcomes |
| **Warning** | `--warning` | Escalated incidents, high severity, attention-needed states |
| **Danger** | `--destructive` | Destructive actions (kick, ban, cancel) and `critical` severity only |
| **Neutral** | `--muted` / `--muted-foreground` | Initial / low-emphasis states (open incident, low severity), secondary text |

**Usage rule for red (`--destructive`):** reserved for the single most severe case in each axis, `critical` severity and irreversible destructive actions. Overusing red destroys its signaling power, so it never marks ordinary states.

---

## Incident state mapping

Each incident state has a distinct color, icon, and text label. Rendered by the
`StateBadge` component (`client/src/components/StateBadge.tsx`).

| State | Color | Icon | Label |
|-------|-------|------|-------|
| `open` | Neutral (gray) | AlertCircle | Open |
| `acknowledged` | Primary | Clock | Acknowledged |
| `escalated` | Warning (amber) | AlertTriangle | Escalated |
| `resolved` | Success (green) | CheckCircle2 | Resolved |

Rationale: `open` is neutral because it carries no alarm yet; `acknowledged` adopts the primary color to signal active handling; `escalated` uses warning amber to draw attention; `resolved` uses success green as the positive terminal state.

---

## Severity mapping

Severity is an axis orthogonal to state. Rendered by the `SeverityBadge`
component (`client/src/components/SeverityBadge.tsx`).

| Severity | Color | Icon | Label |
|----------|-------|------|-------|
| `low` | Neutral (gray) | ChevronDown | Low |
| `medium` | Primary | Equal | Medium |
| `high` | Warning (amber) | ChevronUp | High |
| `critical` | Danger (red) | Flame | Critical |

Rationale: the chevron direction encodes magnitude (down = low, up = high) for an at-a-glance read; `critical` is the only severity allowed to use red.

---

## Release state mapping

Each release state has a distinct color, icon, and text label. Rendered by the
`ReleaseStateBadge` component (`client/src/components/ReleaseStateBadge.tsx`).

| State | Color | Icon | Label |
|-------|-------|------|-------|
| `created` | Neutral (gray) | Circle | Created |
| `in_progress` | Primary (blue) | Play | In Progress |
| `completed` | Success (green) | CheckCircle2 | Completed |
| `cancelled` | Neutral (gray) | XCircle | Cancelled |
| `blocked` | Danger (red) | ShieldAlert | Blocked |

`blocked` is the only release state using red, consistent with the palette rule that red signals the most severe condition.

---

## Reusable components (v1)

| Component | File | Purpose |
|-----------|------|---------|
| `StateBadge` | `components/StateBadge.tsx` | Incident state, color + icon + text |
| `SeverityBadge` | `components/SeverityBadge.tsx` | Severity level, color + icon + text |
| `ReleaseStateBadge` | `components/ReleaseStateBadge.tsx` | Release state, color + icon + text |
| `ConfirmDialog` | `components/ConfirmDialog.tsx` | Confirmation for destructive actions |
| `AppShell` | `components/AppShell.tsx` | Post-login layout: header + sidebar + content |
| `UserMenu` | `components/UserMenu.tsx` | Header dropdown with identity, language, and sign out |
| `UserAvatar` | `components/UserAvatar.tsx` | DiceBear avatar with initials fallback |
| `LanguageToggle` | signin/signup pages | EN/FR toggle button |
| `ConnectionIndicator` | `components/ConnectionIndicator.tsx` | WebSocket status, color + icon + text |

These build on shadcn/ui primitives (Radix-based) under `components/ui/`.
The full inventory with all variants is documented in VGL-114.

---

## Information hierarchy

- **Title** : page or section heading (`text-2xl` / `text-lg font-semibold`).
- **Subtitle** : section grouping (`text-sm font-medium text-muted-foreground`).
- **Body** : default content text.

Critical actions are visually distinct from secondary ones: primary buttons use
the primary color, destructive buttons use red, secondary actions are neutral.

---

## Dark patterns, identified and avoided

The interface must not manipulate the user. Measures in place:

- **All destructive actions** (kick, ban, transfer Manager, cancel Release) go through `ConfirmDialog`, which **names the affected resource** in its message (e.g. "Kick Alice from the team?"). No destructive action fires on a single unconfirmed click.
- **No confirmation inversion** : confirm means confirm; the destructive button is clearly labeled and colored red, the cancel button is the safe default.
- **No hidden critical options** : sign out and other important actions live in predictable, visible locations (the header user menu).
- **Audit log is read-only** : Managers can view the history of governance actions but cannot modify or delete entries. Moderation does not rewrite the past.
- **Language change is immediate** : switching language does not require a page reload or re-login. The interface updates in place.

---

## Internationalization

The interface is available in **French and English**. Translation dictionaries live in `client/src/locales/en.json` (source of truth) and `client/src/locales/fr.json`. The `t()` function is typed with `TranslationKey`, catching typos at compile time.

Language can be changed from:
- The **profile page** (persisted server-side via `PATCH /me`)
- The **user menu** in the header (quick toggle)
- The **signin/signup pages** (before authentication)

API values (states, severities) stay in English internally; only their displayed labels are translated. Dates use the active language locale (`toLocaleDateString(getLanguage(), ...)`).

## Annotated screenshots

Screenshots are located in `docs/screenshots/`. At minimum two are required:

1. **Incident detail view** -- showing StateBadge, SeverityBadge, timeline, presence watchers, reaction picker, and role-contextual actions
2. **Release detail view** -- showing ReleaseStateBadge, sequential stepper, blocked banner with incident link, and confirmation dialog on cancel

These screenshots demonstrate the three-signal rule (color + icon + text), the confirmation dialog pattern, and the keyboard-accessible action buttons.