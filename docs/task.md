# Kanban Planner — Prioritized Execution Plan

## Completed Work

- [x] `CardId`, `BucketId` ULID newtypes — `src/domain/id.rs` (`PartialOrd`/`Ord` derived)
- [x] `Bucket` entity — `src/domain/bucket.rs`
- [x] `Card` entity (2 constructors, private fields, controlled mutators) — `src/domain/card.rs`
- [x] `DomainError` enum — `src/domain/error.rs`
- [x] Title validation at construction (`EmptyTitle`) — `Card::new_root`, `Card::new_child`
- [x] Strict bucket reorder (`DuplicateBucketId`, `BucketNotFound`) — `Card::reorder_buckets`
- [x] All `Result<_, String>` in `card.rs` replaced with `Result<_, DomainError>`
- [x] `CardRegistry` + `DeleteStrategy` — `src/domain/registry.rs`
- [x] Registry methods: `create_root_card`, `create_child_card`, `get_card`, `get_root_cards`, `get_children`, `board_projection`
- [x] Registry mutations: `rename_card`, `add_bucket`, `reorder_buckets`, `move_card_to_bucket`, `remove_bucket`
- [x] `detect_cycle`, `reparent_card`, `delete_card` with all three `DeleteStrategy` variants
- [x] Registry tests: lifecycle, cycle detection, delete strategies, bucket-not-empty guard
- [x] Dioxus shell and route scaffolding added — `src/interface/app.rs`, `src/interface/routes/`
- [ ] `cargo test --all`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt -- --check` all pass on the current interface branch (`cargo test` currently passes; clippy/rustfmt still fail)
- [x] Architecture documented — `docs/design_document.md`
- [x] Agent skills defined — `.agents/skills/`
- [x] GitHub Issues created and P0/P1 issues closed

---

## P0 — Stabilize Domain Invariants ✅ COMPLETE

---

## P1 — Build CardRegistry ✅ COMPLETE

---

## P1.5 — Harden Registry Correctness Gaps ✅ COMPLETE

- [x] **Same-parent reparent is a no-op:** Added early-return guard in `reparent_card`.
- [x] **Regression test for same-parent no-op:** `test_reparent_to_same_parent_is_noop` added.
- [x] **Fail loudly in `get_children`:** Replaced soft guards with hard `DomainError::CardNotFound`.
- [x] **Fail loudly in `board_projection`:** Replaced silent fallback with `DomainError::BucketNotFound`.
- [x] **Regression tests for read-path corruption:** `test_get_children_fails_on_missing_child` and `test_board_projection_fails_on_unknown_bucket` added.

---

## P2 — Application Commands ✅ COMPLETE

- [x] `src/application/mod.rs` — `Command` enum + `execute` dispatcher
  - Variants: `CreateRootCard`, `CreateChildCard`, `RenameCard`, `DeleteCard { strategy }`,
    `MoveCardToBucket`, `ReparentCard`, `AddBucket`, `RemoveBucket`, `RenameBucket`,
    `ReorderBuckets`, `ReorderChildren`
  - `execute(command, &mut registry) -> Result<(), DomainError>`
- [x] `BoardView { card: &Card, columns: Vec<ColumnView> }` struct
- [x] `ColumnView { bucket: &Bucket, cards: Vec<&Card> }` struct
- [x] `build_board_view(card_id, &registry) -> Result<BoardView, DomainError>` —
  Unassigned column omitted when empty

---

## P3 — Persistence / Infrastructure ✅ COMPLETE

- [x] `Serialize`/`Deserialize` derives added to domain types (gate: verify they compile for WASM)
- [x] `src/infrastructure/mod.rs`
- [x] `JsonRepository`: `serialize_registry` / `deserialize_registry` using `serde_json`
- [x] Roundtrip integration test: create → serialize → deserialize → verify full structural equality
- [x] `LocalStorageRepository`: `save_to_local_storage` / `load_from_local_storage` using `web-sys`
- [x] Janitor gate: verify `web-sys` compiles for `wasm32-unknown-unknown` before adding

---

## P4 — Dioxus Interface

- [ ] Fix the current interface lint/format regressions in `src/interface/app.rs` and related interface files
- [x] `tailwind.config.js`: Initialize Tailwind CSS, configure `darkMode: 'class'`, and setup Warm Orange brand colors.
- [x] Dioxus Router setup (`/` vs `/board/:card_id`) in `app.rs`.
- [ ] Implement `TopBar` component (Flexbox: Back Arrow/Previous Card Name, Current Card Name, +Create Bucket, Modifiers).
- [x] Implement `Board` and `Column` view layout (horizontal/vertical auto-scrolling).
- [ ] Implement `CardItem` component with click-to-nav behaviour and "Move" dropdown context menu.
- [ ] Implement blurred `Modal` system for "Create Card," "Rename Item," and "Create Bucket." Modal closes on "X" (auto-saves).
- [x] Hook UI components to application layer `Command` dispatching via shared `Signal<CardRegistry>`.
- [x] Run state diffs against `LocalStorageRepository` on each mutation to persist data.

---

## P5 — Release & Docs

- [ ] Reviewer pass: zero `.unwrap()` in non-test code
- [ ] Readability pass: all public items have `///` doc-comments with `# Examples`
- [ ] Optimizer pass: unnecessary clones, redundant allocations
- [ ] `dx serve` (WASM build) verified
- [ ] `dx serve --platform desktop` (native) verified
- [ ] `README.md` written
- [ ] Tag `v0.1.0` on GitHub
