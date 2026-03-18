# Kanban Planner - Prioritized Execution Plan

## Completed Work

- [x] `CardId` and `BucketId` ULID newtypes in `src/domain/id.rs`
- [x] `Bucket` entity in `src/domain/bucket.rs`
- [x] `Card` entity with constructor and mutation invariants in `src/domain/card.rs`
- [x] `DomainError` typed error model in `src/domain/error.rs`
- [x] Title validation at construction and rename time
- [x] Strict bucket reorder validation
- [x] `CardRegistry` and `DeleteStrategy`
- [x] Cross-card registry mutations and projections
- [x] Application command dispatcher in `src/application/mod.rs`
- [x] UI projections via `BoardView` and `ColumnView`
- [x] JSON persistence and browser `localStorage` persistence
- [x] `AppPersistence` facade with explicit non-web unsupported behavior
- [x] Runtime logging foundation in `src/infrastructure/logging.rs`
- [x] Build logging wrapper scripts under `scripts/`
- [x] Dioxus shell, routes, board view, home view, and modal flows
- [x] `cargo test --all` passes
- [x] `cargo clippy --all-targets -- -D warnings` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo check --target wasm32-unknown-unknown` passes
- [x] `cargo check --no-default-features --features desktop` passes
- [x] Design document reconciled with the current MVP implementation
- [x] `README.md` reconciled with actual feature status

---

## P0 - Stabilize Domain Invariants COMPLETE

- [x] Replace `Result<_, String>` with `DomainError`
- [x] Reject blank titles in constructors
- [x] Reject invalid bucket reorder input
- [x] Enforce registry-owned cross-card validation

---

## P1 - Build CardRegistry COMPLETE

- [x] Root and child creation
- [x] Card lookup and root lookup
- [x] Child lookup and board projection
- [x] Rename, move, reparent, delete, and bucket mutation flows

---

## P1.5 - Harden Registry Correctness COMPLETE

- [x] Same-parent reparent returns `Ok(())` without mutation
- [x] Regression test for same-parent no-op
- [x] `get_children` fails loudly on missing child ids
- [x] `board_projection` fails loudly on invalid bucket references
- [x] Regression tests for corrupt read paths

---

## P2 - Application Commands COMPLETE

- [x] `Command` enum for all supported registry mutations
- [x] `execute(command, &mut registry) -> Result<(), DomainError>`
- [x] `execute` owns command lifecycle logging
- [x] `BoardView` and `ColumnView` projections
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

---

## P4 - Dioxus Interface COMPLETE

- [x] Dioxus router setup in `src/interface/app.rs`
- [x] `TopBar` component
- [x] Board and column layout
- [x] `CardItem` component with move and rename actions
- [x] Blurred modal system for create-card, rename-card, and create-bucket
- [x] UI hooked to `Command` dispatch through shared `Signal<CardRegistry>`
- [x] Persistence runs through `AppPersistence`
- [x] Child-card creation rejects missing bucket context instead of synthesizing ids
- [x] Board route preserves `DomainError` for logs and renders a stable fallback panel
- [x] ASCII-safe user-facing labels replace mojibake glyphs

---

## P5 - Release and Docs

- [x] Reviewer pass: zero `.unwrap()` in non-test code
- [x] Readability pass: public API entry points have `///` docs with `# Examples`
- [x] Documentation reconciled with current MVP behavior
- [x] Web-target cargo validation is green
- [x] Desktop-target cargo compilation validated
- [ ] `dx serve --platform desktop` verified end-to-end
- [ ] Export/import flow implemented
- [ ] Tag `v0.1.0` on GitHub

---

## Proposed Next Stages

### P6 - Direct Manipulation UX

- [ ] Replace the move dropdown with drag-and-drop for cards within a board
- [ ] Add drag-and-drop bucket ordering in the board view
- [ ] Decide whether root boards on `Home` should support drag-and-drop ordering too

### P7 - Card Notebook

- [ ] Add notes to cards
- [ ] Open notes in a notebook-style popup
- [ ] Support titled note pages rather than a single untitled text blob

### P8 - Due Dates

- [ ] Add an optional due date field to cards
- [ ] Surface overdue and due-soon state in board cards
- [ ] Decide whether completion is inferred from bucket membership or stored explicitly

### P9 - Tags and Trigger Rules

- [ ] Add configurable tags to cards
- [ ] Separate presentational tags from behavior-driving tags if both are needed
- [ ] Define trigger points such as note-open, note-close, and bucket-entry
- [ ] Route trigger effects through an explicit rule engine instead of ad hoc UI conditionals
