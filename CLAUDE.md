# CaseHelper

Social worker toolbelt for Salvation Army Australia. LLM-assisted case notes, case plans, Todoist sync, archive search.

## Tech Stack
- Tauri 2 (Rust shell + web frontend)
- React + TypeScript + Zustand
- tauri-plugin-sql (SQLite WAL + FTS5)
- tauri-plugin-keychain (API key storage)
- casehelper-llm (Rust crate, reqwest, OpenAI-compatible)
- casehelper-todoist (Rust crate, 60s polling)

## Design System
Always read `DESIGN.md` in the project root before making any visual or UI decisions.
All font choices, colors, spacing, and aesthetic direction are defined there.
Do not deviate without explicit user approval.

## Key Design Decisions (from /plan-design-review + /design-consultation)
- Nav: Sidebar + top bar (left case list, main workspace, right Todoist panel)
- Case note layout: Draft primary, raw input secondary
- Typography: Geist (Google Fonts)
- Accent: slate blue (#4a5568) on warm off-white (#f8f8f6)
- Responsive: hamburger on mobile, full sidebar on desktop (≥1024px)

## Project Artifacts
- `ceo-plans/` — CEO plan
- `designs/` — design mockups and approved directions
- `DESIGN.md` — design system tokens and component specs
