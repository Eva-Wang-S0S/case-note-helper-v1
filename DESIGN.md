# Design System — CaseHelper

## Product Context

- **What this is:** Toolbelt app for Salvation Army Australia social workers — LLM-assisted case notes, case plans, Todoist sync, archive search. v1 ships case notes only.
- **Who it's for:** Social workers managing 5-10 long-running cases simultaneously, switching between cases multiple times daily.
- **Space/industry:** Social services / NGO case management
- **Project type:** App UI — workspace-driven, task-focused, data-dense. Not a marketing or landing page.

## Aesthetic Direction

- **Direction:** Utilitarian-Calm
- **Decoration level:** Minimal — typography and whitespace do the work. No decorative elements. One subtle texture on sidebar background (5% opacity noise) to break up flat color without adding visual noise.
- **Mood:** Professional, trustworthy, efficient. Bloomberg Terminal's confidence. Not warm-and-fuzzy, not startup-y. Serious tool for serious work.
- **Reference:** Linear, Notion, Bloomberg Terminal (visual reference, not direct inspiration)

## Typography

- **Body/UI:** Geist (Google Fonts) — professional, has character without being flashy, excellent legibility at 14px, tabular-nums for data
- **Monospace (case IDs, timestamps, code):** Geist Mono — pairs naturally with Geist
- **Font loading:** Google Fonts CDN with `display=swap`. Preconnect to `fonts.googleapis.com` and `fonts.gstatic.com`.
- **Scale:**
  - 12px — xs (muted labels, timestamps)
  - 14px — body/default
  - 16px — large body
  - 20px — section headings
  - 24px — page titles
- **Weights used:**
  - 400 regular — body text, descriptions
  - 500 medium — labels, buttons, tab names
  - 600 semibold — headings, active states, case names

## Color

- **Approach:** Restrained — slate blue is rare and meaningful. Color is used to signal state, not decorate.

### Palette

| Token | Hex | Usage |
|-------|-----|-------|
| `--color-bg` | `#f8f8f6` | App background (warm off-white — NOT pure white) |
| `--color-surface` | `#ffffff` | Cards, panels, modals |
| `--color-sidebar` | `#f0f0ee` | Sidebar background (slightly darker warm gray) |
| `--color-text-primary` | `#1a1a1a` | Primary text, headings |
| `--color-text-secondary` | `#6b7280` | Secondary text, descriptions |
| `--color-text-muted` | `#9ca3af` | Timestamps, metadata, placeholder |
| `--color-accent` | `#4a5568` | Primary actions, active states, links |
| `--color-accent-hover` | `#2d3748` | Hover state for accent |
| `--color-border` | `#e5e5e3` | Dividers, input borders |
| `--color-selected` | `#e8e8f0` | Selected row/item background (blue-gray tint) |
| `--color-success` | `#16a34a` | Success states, completed tasks |
| `--color-error` | `#dc2626` | Error states |
| `--color-warning` | `#d97706` | Warning states |

### Dark Mode

Not in v1 scope. If added later: reduce saturation 10-20%, don't redesign surfaces.

## Spacing

- **Base unit:** 4px
- **Density:** Comfortable — caseworkers read dense information all day; editing areas get more breathing room
- **Scale:** 2(2px) / 4(4px) / 8(8px) / 12(12px) / 16(16px) / 24(24px) / 32(32px) / 48(48px) / 64(64px)

## Layout

- **Approach:** Grid-disciplined — strict column discipline, no grid-breaking
- **Sidebar:** 240px fixed width, always visible on desktop
- **Main workspace:** flex-grow, min-width 0
- **Todoist panel:** 280px, collapsible (toggle button on panel edge)
- **Max content width:** No artificial cap — app uses full viewport
- **Breakpoint:** Sidebar collapses to hamburger at <1024px viewport width

## Border Radius

- **Scale:** 4px (inputs, small buttons) / 6px (cards, panels) / 8px (modals, larger containers)
- **No full-round** unless checkbox or avatar

## Motion

- **Approach:** Minimal-functional — only transitions that aid comprehension
- **Easing:** `ease-out` for enters, `ease-in` for exits, `ease-in-out` for moves
- **Duration:** 50-100ms (micro: hover states) / 150-250ms (short: state changes) / 250-400ms (medium: panel slides)
- **No entrance animations** — workers switch contexts constantly; don't make them wait for animations

## Component Inventory

### Case List Item
- **Default:** white bg, case name (14px, semibold), stage tag (pill, muted), days-since-update (12px, muted)
- **Hover:** `--color-sidebar` bg
- **Selected:** `--color-selected` bg, left border 3px `--color-accent`

### Tab Bar (Notes / Plan)
- **Default:** text secondary, no bg
- **Hover:** text primary
- **Active:** text primary, semibold, bottom border 2px `--color-accent`

### Button (Primary)
- **Default:** `--color-accent` bg, white text, 500 weight, 14px, 6px radius, 8px 16px padding
- **Hover:** `--color-accent-hover` bg
- **Disabled:** 50% opacity, no pointer events
- **Loading:** spinner replaces text, same dimensions

### Button (Secondary/Ghost)
- **Default:** transparent bg, `--color-accent` text, 1px `--color-border` border
- **Hover:** `--color-sidebar` bg
- **Active:** `--color-selected` bg

### Input (Textarea)
- **Default:** white bg, 1px `--color-border`, 6px radius, 12px padding, 14px Geist
- **Focus:** 2px `--color-accent` outline, no default ring
- **Placeholder:** `--color-text-muted`

### Toggle (Anonymize)
- **Off:** 1px `--color-border`, track 40px × 20px, knob 16px circle white
- **On:** `--color-accent` bg, white knob
- **Transition:** 150ms ease-out

### Save Indicator
- **Transient "Saved" text:** 12px, `--color-text-muted`, positioned bottom-right of note area
- **Animation:** opacity 0 → 1 (200ms) → hold 1.5s → opacity 1 → 0 (300ms). Total visible ~2s.

### Drafting State
- **Draft pane:** "Drafting..." text (14px, `--color-text-muted`, italic) centered in pane
- **Raw input:** opacity 0.5 while drafting

### Empty State (No Cases)
- **Layout:** centered in main workspace area
- **Content:** "Welcome to CaseHelper" (24px, semibold) + "Add your first case to get started" (14px, text-secondary) + "Add first case" button (primary)
- **No illustration** — text + button is sufficient

### Error State
- **Container:** light red bg (`#fef2f2`), red left border (3px `#dc2626`), 6px radius
- **Content:** error icon (optional) + error message (14px) + retry button (ghost style)

### Todoist Panel
- **Header:** "Todoist" (14px, semibold, text-secondary) + collapse chevron button
- **Task item:** checkbox + task name (14px) + case tag (12px, muted). Completed: checkbox checked, text strikethrough + muted.
- **Empty:** "No tasks for this case" (12px, muted, centered)

## Navigation Structure

- **Left sidebar (240px):** Case list. Always visible on desktop. Collapses on <1024px.
- **Top bar:** Archive search bar (full-width within top bar). Settings gear icon (top-right).
- **Main workspace:** Case detail with tabs (Notes / Plan). Instant switching.
- **Right panel (280px):** Todoist summary. Collapsible.

## Accessibility

- **Contrast:** All text meets WCAG AA (4.5:1 for body, 3:1 for large text)
- **Touch targets:** 44px minimum for all interactive elements
- **Keyboard nav:** Full keyboard navigation. Tab order: sidebar → top bar → workspace → panels. Escape closes panels/modals.
- **ARIA:** Landmarks (`nav`, `main`, `aside`), labels on all form inputs, `aria-live` for save/drafting states
- **Focus:** Visible focus ring (2px `--color-accent`), not removed

## Decisions Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-04-29 | Initial design system created | Created by /design-consultation. Utilitarian-Calm aesthetic. Geist + slate blue. Warm off-white bg. |
