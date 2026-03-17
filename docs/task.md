# Kanban Planner — Prioritized Execution Plan

## Completed Work
- [x] `CardId`, `BucketId` ULID newtypes — `src/domain/id.rs`
- [x] `Bucket` entity — `src/domain/bucket.rs`
- [x] `Card` entity (2 constructors, private fields, controlled mutators) — `src/domain/card.rs`
- [x] Dioxus hello-world shell — `src/app.rs`, `src/main.rs`
- [x] Architecture documented — `docs/design_document.md`
- [x] Agent skills defined — `.agents/skills/`
- [x] Micro-step workflow — `.agents/workflows/micro-step-flow.md`
- [x] GitHub Issues created for all phases

---

## P0 — Stabilize Domain Invariants
*Fix the gaps in existing code before building on top of it.*

- [x] `Card::new_root` and `Card::new_child` must reject blank/empty titles at construction time
- [x] `Card::reorder_buckets` must reject duplicate IDs in the input list
- [x] `Card::reorder_buckets` must reject unknown IDs in the input list
- [x] Create `src/domain/error.rs` — `DomainError` enum using `thiserror`
- [x] Migrate all `Result<_, String>` in `card.rs` to `Result<_, DomainError>`
- [x] `cargo clippy --all-targets -- -D warnings` passes with zero warnings
- [x] `cargo fmt -- --check` passes with zero diffs

---

## P1 — Build CardRegistry
- [x] `src/domain/registry.rs` — `CardRegistry { HashMap<CardId, Card> }`
- [x] `create_root_card`, `create_child_card`
- [x] `get_card`, `get_card_mut`, `get_root_cards`, `get_children`, `board_projection`
- [x] `rename_card`, `add_bucket`, `reorder_buckets` — delegation wrappers
- [x] `move_card_to_bucket` — validates bucket belongs to parent
- [x] `remove_bucket` — rejects if any child references that bucket
- [x] `detect_cycle` + `reparent_card`
- [x] `delete_card(id, DeleteStrategy)` — Reject / CascadeDelete / ReparentToGrandparent
- [x] Full test suite covering every error variant
- [x] Integration test: full lifecycle roundtrip

---

## P2 — Application Commands
- [ ] `src/application/mod.rs` — `Command` enum + `execute` dispatcher
- [ ] `BoardView` + `ColumnView` structs
- [ ] `build_board_view` — Unassigned column omitted when empty

---

## P3 — Persistence / Infrastructure
- [ ] `Serialize`/`Deserialize` derives added to domain types
- [ ] `JsonRepository`: `serialize_registry` / `deserialize_registry`
- [ ] Roundtrip integration test
- [ ] `LocalStorageRepository` using `web-sys`

---

## P4 — Dioxus Interface
- [ ] Routes: `/` and `/board/:card_id`
- [ ] `Signal<CardRegistry>` at root via Dioxus context
- [ ] `RootList`, `Board`, `BucketColumn`, `CardItem`, `Breadcrumb` components
- [ ] Modals: create card, rename, delete confirmation
- [ ] Persistence: load on startup, auto-save, export/import

---

## P5 — Release & Docs
- [ ] Reviewer, Readability, Optimizer passes
- [ ] Dioxus CLI install and `dx serve` verification (browser + desktop)
- [ ] `README.md`
- [ ] Tag `v0.1.0`
