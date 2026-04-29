# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.0.1] - 2026-04-29

### Added
- Plan item CRUD: create, toggle, delete with SQLite persistence
- Frontend store actions for plan item management (createPlanItem, togglePlanItem, deletePlanItem)
- Rust integration tests for db layer (15 tests covering happy path and error paths)
- get_notes coverage with multiple notes and empty state tests

### Fixed
- Fixed SQL typo in init_schema (plan_items case_id → plan_items(case_id))
- Added comment clarifying dual-lock pattern safety in create_plan_item

