# Changelog
# Copyright (C) 2026 SodigTech — GPL-3.0

All notable changes to **ItzamBox** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.1.0] — Phase 3 — Polish, Testing & Monaco — 2026-06-13

### Status: ⚠️ RELEASED-INTERNALLY (Release BLOCKED by QA gate, 2 of 8 gates failing)

### Added — Phase 3 Sprints 13–17

#### Sprint 13 — Compose Monaco Editor (T-057, T-058)
- `src/views/compose/ComposeEditor.vue` — Monaco-based YAML editor with Docker Compose
  schema integration, syntax highlighting, autocomplete, and Ctrl+S direct save keybindings.
- `src-tauri/src/commands/compose.rs` — Tauri commands: `validate_compose_file`,
  `compose_up`, `compose_down`, `compose_ps`, `compose_logs`. Uses `docker compose`
  / `docker-compose` (auto-detect), with input sanitization via
  `utils::sanitizer::validate_container_name` / `validate_path`.
- Auto-validation hook that runs `docker compose config` to surface parse errors inline.

#### Sprint 14 — Notification Persistence (T-059)
- `src-tauri/src/commands/notifications.rs` — Tauri commands: `save_notification`,
  `load_notifications`, `clear_notifications`, `mark_notification_read`. SQLite-backed.
- `src-tauri/migrations/003_notifications.sql` — Schema for `notifications` table with
  auto-pruning policy (keeps latest 100 rows max).
- `src/composables/useNotifications.ts` — Reactive Vue 3 composable wrapping the Tauri
  commands, providing `notifications`, `unreadCount`, `markRead`, `clearAll`.

#### Sprint 15 — Frontend Unit Tests (T-060, T-061, T-062)
- `vitest.config.ts` + `src/__tests__/setup.ts` — Test infrastructure using Vitest 4.1.8
  and `@vue/test-utils` 2.4.x.
- `src/__tests__/mockTauriInvoke.ts` and `mockTauriEvent.ts` — Helpers to mock
  `@tauri-apps/api/core::invoke` and `@tauri-apps/api/event::listen`.
- 7 composable spec files: `useContextMenu`, `useDocker`, `useI18n`,
  `useKeyboardShortcuts`, `useNotifications`, `useTheme`, `useTimeSeries`.
- 7 component spec files: `CommandPalette`, `ContextMenu`, `CpuChart`, `EmptyState`,
  `ErrorState`, `RamChart`, `SkeletonLoader`.
- **Result:** 14 spec files, 60/60 tests pass (4.20 s).

#### Sprint 16 — E2E Tests Playwright (T-063, T-064)
- `playwright.config.ts` — Playwright 1.60.x with chromium project, `pnpm dev` webServer,
  Vite dev server on `http://localhost:5173`.
- `e2e/fixtures/mockDockerResponses.ts` and `waitForApp.ts` — Shared test helpers.
- 9 spec files: `dashboard.spec.ts`, `containers.spec.ts`, `container-detail.spec.ts`,
  `images.spec.ts`, `terminal.spec.ts`, `theme-i18n.spec.ts`, `interactions.spec.ts`,
  `wizard-templates.spec.ts`, `edge-cases.spec.ts`.
- 75 scenarios covering: dashboard metrics, container CRUD, container detail (8 tabs),
  image management, host terminal, theme toggle, i18n ES↔EN, command palette, context
  menu, run wizard, templates, edge cases (Docker down / empty lists), event stream.
- **Result:** 75 scenarios exist, TypeScript compiles, **55/75 pass at runtime. 20 fail**
  and block the release. Self-healing contract: `tests/test_failures.json`.

#### Sprint 17 — Final QA & Release (T-065)
- `tests/TEST_REPORTS.md` — Re-validated test report with honest pass/fail counts.
- `docs/gatekeeper_consensus.json` — REVOKED with full re-validation context.
- `tests/test_failures.json` and `tests/mutation_failures.json` — Machine-readable
  self-healing contracts.

### Changed
- `SDLC_STATUS.md` — Phase 3 status corrected from "✅ COMPLETE" to "⚠️ IN_PROGRESS
  (re-opened)" based on the 2026-06-13 re-validation. Falsified 80/80 entry revoked.
- `tests/TEST_REPORTS.md` — Reframed with honest gate-by-gate counts. Old "80/80" table
  replaced by §1 honesty summary.

### Fixed (post-audit, during Sprint 16 corrective pass)
- Sprint 15 was previously 5/16 spec files (31 %). Now 14/14 (100 %).
- Sprint 16 was previously 1 spec file / 3 scenarios (20 %). Now 9 spec files / 75
  scenarios (100 % of scope). 55/75 pass at runtime.

### Known Issues (release-blockers)
- 20 Playwright E2E scenarios fail at runtime — see `tests/test_failures.json` for the
  self-healing contract with file paths, line numbers, and concrete remedies. Cluster
  in: missing ErrorState/EmptyState branches (8), missing Command Palette / Wizard
  navigation handlers (5), missing SettingsView theme cards & i18n binding (3), missing
  Terminal +Add dropdown (1), missing container Remove action menu (1), regex-unsafe
  `text=` selectors (3). Estimated remediation: ~5 h of frontend work.
- 10 `cargo clippy -D warnings` pedantic lints in 5 files — see
  `tests/mutation_failures.json`. Estimated remediation: 0.5 h of backend cleanup.

### Security
- `npm audit --production`: 0 vulnerabilities.
- `cargo audit`: not run in this sandbox (binary not installed); flagged for @devops
  to re-execute in CI.

---

## [1.0.0] — Phase 6 — Integrated QA, Telemetry & Delivery — 2026-06-13

The pre-Phase-3 production build. See git history for the full Sprint 1–12 changelog.

---

*Maintained by `@qa_engineer` for the Hybrid Squad. Honest changelog policy: every
`Added` / `Changed` / `Fixed` entry must be backed by a file in the repository or a
green test result on the current HEAD. Falsified or aspirational entries are explicitly
flagged with ⚠️ and revoked in the next release.*
