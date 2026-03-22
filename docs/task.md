# Kanban Planner - Prioritized Execution Plan

## Completed Work

- [x] `CardId` and other ULID-backed newtypes in `src/domain/id.rs`
- [x] `Card` entity with constructor and mutation invariants in `src/domain/card.rs`
- [x] `DomainError` typed error model in `src/domain/error.rs`
- [x] Title validation at construction and rename time
- [x] Ordered child-card validation and reordering
- [x] `CardRegistry` with workspace, child creation, projection, and delete flows
- [x] Cross-card registry mutations and projections
- [x] Application command dispatcher in `src/application/mod.rs`
- [x] UI projections via `BoardView` and `CardPreviewView`
- [x] JSON persistence and browser `localStorage` persistence
- [x] `AppPersistence` facade with explicit non-web unsupported behavior
- [x] Runtime logging foundation in `src/infrastructure/logging.rs`
- [x] Build logging wrapper scripts under `scripts/`
- [x] Dioxus shell, routes, board view, home view, and modal flows
- [x] Export, import, and clear-cache web utilities
- [x] Ordered child-card drag-and-drop
- [x] Inline child previews on cards
- [x] Notebook-style notes with titled plain-text pages
- [x] Date-only due dates with overdue state
- [x] Workspace top-level card migration
- [x] Workspace now renders through the same board chrome as other cards
- [x] Modal system split into feature-focused modules
- [x] `src/interface/tailwind.css` is the editable stylesheet source
- [x] Browser smoke test covers app boot, workspace creation, card creation, child preview visibility, and theme toggle
- [x] `cargo test --all` passes
- [x] `cargo clippy --all-targets -- -D warnings` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo check --target wasm32-unknown-unknown` passes
- [x] `cargo check --no-default-features --features desktop` passes
- [x] Design document reconciled with the current MVP implementation
- [x] `README.md` reconciled with actual feature status
- [x] Frontend tooling added for stylesheet generation and smoke tests

---

## P0 - Stabilize Domain Invariants COMPLETE

- [x] Replace `Result<_, String>` with `DomainError`
- [x] Reject blank titles in constructors
- [x] Enforce registry-owned cross-card validation
- [x] Keep corrupt registry reads fail-loud

---

## P1 - Build CardRegistry COMPLETE

- [x] Workspace and child creation
- [x] Card lookup and workspace lookup
- [x] Child lookup and board projection
- [x] Rename, reparent, delete, and child-order flows

---

## P1.5 - Harden Registry Correctness COMPLETE

- [x] Same-parent reparent returns `Ok(())` without mutation
- [x] Regression test for same-parent no-op
- [x] `get_children` fails loudly on missing child ids
- [x] `board_projection` fails loudly on invalid tree structure
- [x] Regression tests for corrupt read paths

---

## P2 - Application Commands COMPLETE

- [x] `Command` enum for all supported registry mutations
- [x] `execute(command, &mut registry) -> Result<(), DomainError>`
- [x] `execute` owns command lifecycle logging
- [x] `BoardView` and `CardPreviewView` projections
- [x] `build_board_view(card_id, &registry) -> Result<BoardView, DomainError>`

---

## P3 - Persistence and Infrastructure COMPLETE

- [x] `Serialize` and `Deserialize` derives across domain types
- [x] `JsonRepository`
- [x] `LocalStorageRepository`
- [x] `AppPersistence` facade
- [x] Explicit non-web session-only fallback with visible warning
- [x] Runtime logging foundation with `tracing`
- [x] Build logging scripts with timestamped output
- [x] Legacy bucket-shaped snapshots reset to a clean workspace

---

## P4 - Dioxus Interface COMPLETE

- [x] Dioxus router setup in `src/interface/app.rs`
- [x] `TopBar` component
- [x] Wrapping board layout for workspace and child cards
- [x] `CardItem` component with open, edit, delete, and notes actions
- [x] Blurred modal system for create-card, rename-card, and notes
- [x] UI hooked to `Command` dispatch through shared `Signal<CardRegistry>`
- [x] Persistence runs through `AppPersistence`
- [x] Workspace back button is disabled because there is no higher level
- [x] ASCII-safe user-facing labels replace mojibake glyphs

---

## P5 - Release and Docs

- [x] Reviewer pass: zero `.unwrap()` in non-test code
- [x] Readability pass: public API entry points have `///` docs with `# Examples`
- [x] Documentation reconciled with current card-only behavior
- [x] Web-target cargo validation is green
- [x] Desktop-target cargo compilation validated
- [ ] `dx serve --platform desktop` verified end-to-end
- [ ] Tag `v0.1.0` on GitHub
- [ ] Keep the browser smoke test aligned with future UI changes

---

## Proposed Next Stages

### P6 - UI Polish COMPLETE

- [x] Tighten drag-and-drop affordances now that cards carry more metadata
- [x] Rebalance card density, spacing, and readability
- [x] Improve modal ergonomics for note editing and board controls

### P7 - Search and Navigation

- [ ] Add search and filtering for larger workspaces
- [ ] Consider quick-open or command-palette behavior for deep trees

### P8 - Native Persistence

- [ ] Add a desktop/mobile persistence backend behind `AppPersistence`
- [ ] Verify export/import/clear-cache behavior across native and web targets
- [ ] Run and verify `dx serve --platform desktop` end-to-end
