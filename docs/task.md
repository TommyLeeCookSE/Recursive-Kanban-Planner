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
- [x] Export, import, and clear-cache web utilities
- [x] Drag-and-drop for cards, buckets, and root boards
- [x] Inline child previews on cards grouped by bucket name
- [x] Notebook-style notes with titled plain-text pages
- [x] Date-only due dates with overdue state
- [x] Reusable visual labels
- [x] Popup-only rule presets for note and bucket events
- [x] Bucket rename is exposed directly in the board UI
- [x] Modal system split into feature-focused modules
- [x] `src/interface/tailwind.css` is the editable stylesheet source
- [x] Browser smoke test covers app boot, board creation, bucket creation, bucket rename, and theme toggle
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
- [ ] Tag `v0.1.0` on GitHub
- [ ] Keep the browser smoke test aligned with future UI changes

---

## Proposed Next Stages

### P6 - UI Polish

- [ ] Tighten drag-and-drop affordances now that boards carry more metadata
- [ ] Rebalance card density, spacing, and readability
- [ ] Improve modal ergonomics for note, label, and rule editing

### P7 - Rule Expansion

- [ ] Add more rule actions beyond popup notifications
- [ ] Explore enable/disable or filter controls for larger rule sets
- [ ] Decide whether recent rule activity needs a visible history panel

### P8 - Native Persistence

- [ ] Add a desktop/mobile persistence backend behind `AppPersistence`
- [ ] Verify export/import/clear-cache behavior across native and web targets
- [ ] Run and verify `dx serve --platform desktop` end-to-end
