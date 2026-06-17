# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`worklog` is a local-first, AI-native work log. The fact source is a `WorkEntry` (action + result + value) stored in local SQLite. AI skills/agents/CLI and humans write entries; humans review them in a Tauri desktop app; confirmed entries are exported as source data for weekly/monthly reports. It is **not** a generic calendar app — calendars are just a grouping/projection of work entries.

## Commands

Rust (run from repo root; `make` wraps the canonical invocations):

- `make fmt` → `cargo fmt --check`
- `make test` → `cargo test --workspace`
- `make build` → `cargo build --workspace`
- `make smoke` → builds + runs a CLI round-trip (add draft → list → confirm → export) against `target/worklog-smoke.db`
- Single test: `cargo test -p worklog-core <name_substring>` (e.g. `cargo test -p worklog-core delete_work_calendar`); a whole integration test: `cargo test -p worklog-core --test scaffold_contract`

CLI (`worklog` binary):

- `cargo run -p worklog-cli -- <subcommand> ...` — subcommands: `entry add|edit|confirm|archive|rm|list`, `calendar add|list|default`, `export report-source`
- DB target precedence: global `--db <path>` flag → `WORKLOG_DB` env → OS app-data default (`~/Library/Application Support/worklog/worklog.db` on macOS). **CLI and the desktop app default to the same DB file**, so CLI writes appear in the desktop UI.
- JSON output (`--format json`) is the stable contract for AI clients — keep field names stable.

Desktop (run from `desktop/`):

- `npm install` then `npm run dev` — Vite preview in a **plain browser**. This uses the in-memory dev shim (see below): no Tauri, no SQLite, mock data. Good for fast UI iteration only.
- `npm run build` → `tsc --noEmit && vite build` (typecheck + bundle frontend)
- `npm run tauri dev` — the real app against real SQLite (debug)
- `npm run tauri build --debug` — bundles a runnable `.app` + `.dmg` (faster than release)

## Architecture

Cargo workspace with three Rust crates + one Preact frontend. **`worklog-core` is the single source of truth for all domain logic; the CLI and desktop are thin adapters that both call into it.** Neither the CLI nor the frontend touches SQL directly.

### `crates/worklog-core` — domain, storage, queries

- `model.rs` — `WorkEntry` (fact source), `WorkCalendar`, input/patch DTOs. Serde shape matters: structs serialize snake_case, enums lowercase, `Source` is a transparent string. The frontend's TS types mirror this exactly.
- `db.rs` — SQLite path resolution (env/flag/OS dir), migrations embedded via `include_str!` and run idempotently on every open, foreign keys on.
- `repo/` — `calendar.rs`, `entry.rs`, `codec.rs`. All repository operations live here.
- `report.rs` — `report_source` export (confirmed entries → JSON / raw numbered Markdown).
- `sync.rs` — Phase-2 `SyncAdapter` trait + projection types only. **No external sync is implemented** (no Google/macOS/ICS/CalDAV code anywhere — by design).

### Core invariants (these span multiple files; respect them)

- A `WorkEntry` belongs to exactly one `WorkCalendar` via `calendar_id` (NOT NULL FK). **Calendar groups are the only grouping dimension in the desktop UI.**
- `status` ∈ {draft, confirmed, archived}. **AI-created entries default to `draft`** (`actor=ai` or an AI-client `source` with no explicit `--status`); trusted/HITL-confirmed flows must pass `--status confirmed`. **Reports/exports only ever read `confirmed`.** This draft-by-default + confirmed-only-in-reports rule is the backbone of the review workflow.
- The `project` string field still exists in the model/CLI/report, but the **desktop UI deliberately no longer writes it** (legacy; grouping moved to calendar groups). Desktop-created entries therefore have `project = null` in report output.
- At most one default `WorkCalendar` (enforced by a partial unique index). The default calendar cannot be deleted; deleting any other calendar **cascades** to its entries.

### `crates/worklog-cli` — the AI-integration surface (`worklog` bin)

Hand-rolled arg parser (`args.rs`, no clap). `lib.rs::run_with_args` extracts the global `--db` first, then dispatches to `commands/{entry,calendar,export}.rs`. Datetimes accept RFC3339 or `YYYY-MM-DD HH:MM` (parsed as UTC). Stable command names are the contract for skills/agents.

### `desktop/` — Tauri v2 desktop app (no sidecar, no HTTP server)

- `src-tauri/src/lib.rs` — the Tauri builder: registers commands + the autostart plugin, builds the tray icon, and intercepts window close (CloseRequested → hide to tray, not quit; tray menu Show/Quit; macOS Reopen restores the window).
- `src-tauri/src/commands.rs` — each Tauri command is a thin `*_for_state` wrapper over `worklog_core::repo`. `state.rs` holds `AppState` = a `Mutex<Connection>` + `DesktopSettings` (db_path, current filters).
- `desktop/src/` — Preact + `@preact/signals`.
  - **`api.ts` is the ONLY backend entry**: typed wrappers over Tauri `invoke`, plus a **browser dev shim** — when `__TAURI_INTERNALS__` is absent (i.e. `npm run dev` in a browser), it falls back to an in-memory store with seed data so the UI is clickable without Tauri/SQLite. The real app path is unaffected.
  - `store.ts` holds signals + actions (view, entries, filters, calendars, CRUD). `selection.ts` is shared selection state. Views live in `views/`.

### `skills/` — worklog CLI skills

`log-work` (write via `worklog entry add`) and `weekly-report` (read via `worklog export report-source`). These are the AI-facing automation, migrated off earlier macOS-EventKit/`oa ` calendar prototypes onto the stable CLI.

## Gotchas

- `npm run dev` (Vite/browser) is **not** the real app — it runs the dev shim with mock data and no persistence. Verify real behavior with `npm run tauri dev` or a built binary.
- `docs/` is gitignored — design/plans/contract docs live locally only and are not in the repo. Keep architectural context in this file, not in `docs/` pointers.
- `package.json` declares `packageManager: pnpm`, but the committed lockfile is `package-lock.json` (npm). They are inconsistent — pick one before depending on it.
- CI: `.github/workflows/release.yml`.
