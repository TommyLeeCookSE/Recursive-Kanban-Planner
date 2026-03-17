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
- [x] Dioxus shell compiles — `src/app.rs`
- [x] `cargo test --all`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt -- --check` all pass
- [x] Architecture documented — `docs/design_document.md`
- [x] Agent skills defined — `.agents/skills/`
- [x] GitHub Issues created and P0/P1 issues closed

---

## P0 — Stabilize Domain Invariants ✅ COMPLETE

---

## P1 — Build CardRegistry ✅ COMPLETE

---

## P1.5 — Harden Registry Correctness Gaps
*These gaps exist in the current P1 implementation and must be fixed before P2 begins.*

- [ ] **Same-parent reparent is a no-op:** `reparent_card(id, current_parent_id)` currently runs
  the full reparent path, double-appending the child ID and resetting the bucket to Unassigned.
  Add an early-return guard: if `card.parent_id() == Some(new_parent_id)`, return `Ok(())` immediately.
- [ ] **Regression test for same-parent no-op:** `test_reparent_to_same_parent_is_noop` —
  verify child count is unchanged, child ordering is unchanged, and bucket assignment is unchanged.
- [ ] **Fail loudly in `get_children`:** Remove the `if let Some(child)` silent-skip.
  Replace with a hard `self.get_card(*child_id)?` that returns `DomainError::CardNotFound`.
- [ ] **Fail loudly in `board_projection`:** Remove the Unassigned fallback for unknown bucket IDs.
  Return `DomainError::BucketNotFound` if a child's `bucket_id` is absent from the parent's buckets.
- [ ] **Regression tests for read-path corruption:** Add tests that manually construct a broken
  state (via `pub(crate)` helpers) and assert that `get_children` / `board_projection` return
  the expected errors instead of silently degrading.

---

## P2 — Application Commands
- [ ] `src/application/mod.rs` — `Command` enum + `execute` dispatcher
  - Variants: `CreateRootCard`, `CreateChildCard`, `RenameCard`, `DeleteCard { strategy }`,
    `MoveCardToBucket`, `ReparentCard`, `AddBucket`, `RemoveBucket`, `RenameBucket`,
    `ReorderBuckets`, `ReorderChildren`
  - `execute(command, &mut registry) -> Result<(), DomainError>`
- [ ] `BoardView { card: &Card, columns: Vec<ColumnView> }` struct
- [ ] `ColumnView { bucket: &Bucket, cards: Vec<&Card> }` struct
- [ ] `build_board_view(card_id, &registry) -> Result<BoardView, DomainError>` —
  Unassigned column omitted when empty

---

## P3 — Persistence / Infrastructure
- [ ] `Serialize`/`Deserialize` derives added to domain types (gate: verify they compile for WASM)
- [ ] `src/infrastructure/mod.rs`
- [ ] `JsonRepository`: `serialize_registry` / `deserialize_registry` using `serde_json`
- [ ] Roundtrip integration test: create → serialize → deserialize → verify full structural equality
- [ ] `LocalStorageRepository`: `save_to_local_storage` / `load_from_local_storage` using `web-sys`
- [ ] Janitor gate: verify `web-sys` compiles for `wasm32-unknown-unknown` before adding

---

## P4 — Dioxus Interface
- [ ] Routes: `/` and `/board/:card_id` via `dioxus_router`
- [ ] `Signal<CardRegistry>` at root; provided via Dioxus context to all child components
- [ ] `RootList` component — lists root cards; empty state CTA
- [ ] `Board` component — reads route param, calls `build_board_view`
- [ ] `BucketColumn` component — Unassigned hides when empty; named columns shrink to name width
- [ ] `CardItem` component — click navigates into card; context menu for rename/delete/move
- [ ] `Breadcrumb` component — ancestor chain, each node clickable
- [ ] Modals: create card, rename, delete confirmation (with `DeleteStrategy` choice)
- [ ] Persistence integration: load on startup from localStorage, auto-save after every mutation,
  JSON export/import buttons

---

## P5 — Release & Docs
- [ ] Reviewer pass: zero `.unwrap()` in non-test code
- [ ] Readability pass: all public items have `///` doc-comments with `# Examples`
- [ ] Optimizer pass: unnecessary clones, redundant allocations
- [ ] `dx serve` (WASM build) verified
- [ ] `dx serve --platform desktop` (native) verified
- [ ] `README.md` written
- [ ] Tag `v0.1.0` on GitHub
