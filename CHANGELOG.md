# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.0.3] - 2026-04-30

### Added
- Configurable LLM note prompt: user can customize the system prompt used when drafting case notes
- Redaction list with usage examples: redaction list now shows explanation and common examples to add
- CaseNoteSettings page: quick-access settings page at /settings/casenotes for redaction and prompt config
- Case-specific draft textarea: raw observations and draft persist per case, not shared across cases
- Raw observations persist on LLM error: if draft generation fails, raw input is saved to prevent data loss
- + Add Case button in sidebar: create new cases from any page via the sidebar footer
- Context menu on selected case: stage change and delete options via dropdown on active case
- Stage color dots: 5 stages now display colored dots (blue/amber/purple/cyan/gray) instead of text tags
- Delete case with confirmation: permanent deletion requires confirmation dialog

### Changed
- Case sort order: cases now sorted by creation date (newest first) instead of last-updated
- Redaction list defaults to empty: removed non-functional placeholder entries
- Draft textarea has Save button: users can manually save draft content

### Fixed
- New case starts empty: newly created cases show blank notes fields, not leftover from previous case

## [0.1.0.2] - 2026-04-30

### Added
- Todoist integration (Phase 1 + 2.5 + 3 partial): Rust `TodoistClient` with REST API (get_tasks, get_tasks_by_ids, create_task, update_task), httpmock test coverage
- Poll worker: 60s sync loop with `last_write_at` guard (120s), rate-limit cooldown, orphaned task reference cleanup
- `sync_plan_item_toggle` command: toggles local DB + PUT to Todoist with sync status tracking
- `TodoistConnectionStatus` struct and `get_todoist_connection_status` command for frontend status dot
- `PlanItem` gains `todoist_task_id`, `sync_status`, `last_synced_at` columns via additive migration
- New DB functions: `set_plan_item_sync_status`, `get_plan_items_with_todoist_ids`, `clear_todoist_task_id`
- `TodoistPanel.tsx` component: status dot (green/yellow/red), "Synced X min ago", task list grouped by due date
- `appStore.ts` extended: `loadTodoistTasks`, `getTodoistConnectionStatus`, `syncPlanItem` actions; `TodoistTask` and `TodoistConnectionStatus` interfaces
- `DESIGN.md` updated: Todoist Panel spec, Plan Item component spec
- `CasePlan.tsx` improved: checkbox toggle, delete button on hover, empty state

### Fixed
- Fixed `chrono::Utc::now()` call in poll worker (was using wrong module)

## [0.1.0.1] - 2026-04-29

### Added
- Plan item CRUD: create, toggle, delete with SQLite persistence
- Frontend store actions for plan item management (createPlanItem, togglePlanItem, deletePlanItem)
- Rust integration tests for db layer (15 tests covering happy path and error paths)
- get_notes coverage with multiple notes and empty state tests

### Fixed
- Fixed SQL typo in init_schema (plan_items case_id → plan_items(case_id))
- Added comment clarifying dual-lock pattern safety in create_plan_item

