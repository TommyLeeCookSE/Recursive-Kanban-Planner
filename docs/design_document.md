# Recursive Kanban Planner - Design Document

## Core Philosophy

This project prioritizes clean, maintainable code over speed of feature delivery.

We optimize for:

- Maximum type safety
- Explicit data flow and mutations
- Minimal duplication without blurring boundaries
- Human readability first
- Strict layer separation: Domain -> Application -> Infrastructure -> Interface

---

## Project Overview

Recursive Kanban Planner is a local-first planning tool where everything is a `Card`.
There are no separate task, project, or board entities. A card can contain child cards,
and opening a card means viewing that card's board.

Stack:

- Rust
- Dioxus
- WASM, desktop, and mobile targets from one codebase
- No server
- No auth

---

## Current Implementation Status

Last reconciled: 2026-03-18

Current validation state:

- `cargo test` passes
- `cargo clippy --all-targets -- -D warnings` passes
- `cargo fmt --check` passes
- `cargo check --target wasm32-unknown-unknown` passes
- `cargo check --no-default-features --features desktop` passes

Implemented in the current worktree:

| Item | File | Notes |
| :--- | :--- | :--- |
| `CardId`, `BucketId`, and feature id newtypes | `src/domain/id.rs` | ULID-backed ids for cards, buckets, notes, labels, and rules |
| `Bucket` entity | `src/domain/bucket.rs` | Private fields and controlled mutators |
| `Card` entity | `src/domain/card.rs` | Local invariants enforced inside the entity, including notes, due dates, labels, and rules |
| `NotePage`, `DueDate`, `LabelDefinition`, `RuleDefinition` | `src/domain/` | Feature-specific value objects and definitions |
| `DomainError` | `src/domain/error.rs` | Typed domain errors throughout the domain/application layers |
| `CardRegistry` and `DeleteStrategy` | `src/domain/registry.rs` | Cross-card invariant boundary plus workspace-level labels/rules |
| `Command` dispatcher | `src/application/mod.rs` | `execute` owns command lifecycle logging |
| `BoardView`, `ColumnView`, and rule evaluation | `src/application/mod.rs` | Read-only UI projections plus popup trigger evaluation |
| JSON persistence | `src/infrastructure/repository.rs` | Serialize/deserialize full registry state |
| Browser persistence | `src/infrastructure/repository.rs` | `localStorage` on `wasm32` |
| `AppPersistence` facade | `src/infrastructure/repository.rs` | Explicit browser-first persistence boundary |
| Runtime logging | `src/infrastructure/logging.rs` | `tracing`, diagnostics buffer, native/web setup |
| Dioxus shell and routing | `src/interface/` | App shell, routes, modal system, board/home views |
| `TopBar`, `CardItem`, modal flows | `src/interface/components/` | Create/edit card, notes, labels, rules, and bucket UI |
| Deterministic child-card creation | `src/interface/components/modal.rs` | Child creation requires a real bucket id |
| Fail-loud board fallback | `src/interface/routes/board.rs` | Board load preserves the real `DomainError` for logs |
| Public API docs with examples | `src/application/`, `src/infrastructure/`, `src/interface/` | Public entry points now have `# Examples` blocks |

Not yet implemented or not yet fully verified:

- Desktop-target `dx serve --platform desktop` runtime verification
- Native desktop/mobile persistence backend beyond browser storage
- Manual browser sanity pass across the newer notes, due-date, labels, rules, and drag/drop flows
- Release tagging workflow

Recent binding decisions already implemented:

1. Same-parent reparenting is a no-op.
2. Registry read-path corruption fails loudly.
3. Child-card creation in the UI must never synthesize a fallback `BucketId`.
4. The board route must keep the underlying `DomainError` for logs and show a user-safe fallback.
5. Command lifecycle logging belongs to `application::execute`.

---

## Core Concept: Cards And Buckets

The current shipped model is intentionally recursive, but it is not cards-only.

- `Card` is the primary planning item and board owner.
- `Bucket` is a column owned by a card.
- Opening a card means viewing that card's buckets and child cards.

Cards still form a strict tree:

- Root cards have `parent_id: None`.
- Non-root cards have both `parent_id: Some(...)` and `bucket_id: Some(...)`.
- Depth is unlimited.
- Navigation means entering a card and viewing that card's board.

---

## Authoritative Domain Contract

This section is the source-of-truth contract for the implemented domain layer.

### `DomainError`

```rust
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DomainError {
    CardNotFound(CardId),
    BucketNotFound(BucketId),
    DuplicateBucketId(BucketId),
    EmptyTitle,
    DuplicateBucketName(String),
    BucketNotEmpty,
    CardHasChildren,
    CycleDetected,
    InvalidOperation(String),
}
```

### `DeleteStrategy`

```rust
pub enum DeleteStrategy {
    Reject,
    CascadeDelete,
    ReparentToGrandparent,
}
```

### `CardRegistry`

```rust
pub struct CardRegistry { /* store is private */ }

impl CardRegistry {
    pub fn new() -> Self;

    pub fn create_root_card(&mut self, title: String) -> Result<CardId, DomainError>;
    pub fn create_child_card(
        &mut self,
        title: String,
        parent_id: CardId,
        bucket_id: BucketId,
    ) -> Result<CardId, DomainError>;

    pub fn get_card(&self, id: CardId) -> Result<&Card, DomainError>;
    pub fn get_root_cards(&self) -> Vec<&Card>;
    pub fn get_children(&self, parent_id: CardId) -> Result<Vec<&Card>, DomainError>;
    pub fn board_projection(
        &self,
        card_id: CardId,
    ) -> Result<HashMap<BucketId, Vec<&Card>>, DomainError>;

    pub fn rename_card(&mut self, id: CardId, title: String) -> Result<(), DomainError>;
    pub fn add_bucket(&mut self, card_id: CardId, name: String) -> Result<BucketId, DomainError>;
    pub fn reorder_buckets(
        &mut self,
        card_id: CardId,
        ordered_ids: Vec<BucketId>,
    ) -> Result<(), DomainError>;
    pub fn move_card_to_bucket(
        &mut self,
        card_id: CardId,
        bucket_id: BucketId,
    ) -> Result<(), DomainError>;
    pub fn remove_bucket(&mut self, card_id: CardId, bucket_id: BucketId)
        -> Result<(), DomainError>;
    pub fn reparent_card(
        &mut self,
        card_id: CardId,
        new_parent_id: CardId,
    ) -> Result<(), DomainError>;
    pub fn delete_card(
        &mut self,
        card_id: CardId,
        strategy: DeleteStrategy,
    ) -> Result<(), DomainError>;
}
```

---

## Invariant Ownership

### `Card` owns local invariants

These do not require cross-card lookup:

- Title must be non-empty in constructors and `rename`
- Bucket names must be unique per card
- Bucket reorder must reject duplicates, omissions, and unknown ids
- The "Unassigned" bucket cannot be removed

### `CardRegistry` owns cross-card invariants

These require looking across cards:

- Parent card must exist before creating a child
- Child `bucket_id` must belong to the parent card
- Reparenting must reject cycles
- Removing a bucket must fail while children still reference it
- Delete strategies must be enforced at the registry level
- Same-parent reparenting must return `Ok(())` without mutating ordering or bucket assignment
- Corrupt read paths must fail loudly rather than skipping data or silently falling back

---

## Resolved Behavioral Decisions

### Same-parent reparenting

`reparent_card(id, current_parent)` is a no-op that returns `Ok(())`.
It must preserve:

- child ordering
- `parent_id`
- `bucket_id`

Regression test:

- `test_reparent_to_same_parent_is_noop`

### Read-path corruption policy

The registry fails loudly if:

- a parent references a missing child card
- a child points to a bucket that does not belong to the parent board

Error shapes:

- `DomainError::CardNotFound`
- `DomainError::BucketNotFound`

Regression tests:

- `test_get_children_fails_on_missing_child`
- `test_board_projection_fails_on_unknown_bucket`

### Interface determinism

The interface layer must not invent domain identifiers.

Current enforced behavior:

- child-card creation without a selected bucket is rejected before any command is built
- the modal stays open and shows inline validation feedback
- the application layer remains the single owner of command logging
- note-open, note-close, and moved-to-bucket events are routed through a shared popup-rule dispatcher

### Board route failure handling

Board loading is a typed `Result` path, not an `Option` path.

Current enforced behavior:

- `build_board_view(...)` errors remain typed
- the route logs the full `DomainError`
- the user sees a stable fallback panel rather than a misleading "not found" screen

### Persistence validation

The persistence boundary validates deserialized registry state before it is accepted.

Current enforced behavior:

- malformed or tampered snapshots are rejected during JSON load/import
- legacy snapshots without newer optional fields still load through serde defaults
- workspace-level label and rule definitions must stay consistent with per-card assignments

---

## Architecture Overview

```text
src/
|- domain/          <- Pure domain logic, no I/O
|  |- id.rs
|  |- bucket.rs
|  |- card.rs
|  |- error.rs
|  |- registry.rs
|  `- mod.rs
|- application/     <- Commands and read-model projections
|  `- mod.rs
|- infrastructure/  <- Persistence and logging adapters
|  `- mod.rs
|- interface/       <- Dioxus UI layer
|  |- app.rs
|  |- components/
|  |- routes/
|  |- error_templates.rs
|  `- mod.rs
|- lib.rs
`- main.rs
```

Layer responsibilities:

- Domain: invariants and core state transitions
- Application: command dispatch and UI-friendly projections
- Infrastructure: persistence, logging, platform adapters
- Interface: Dioxus components, routing, user-facing flows

---

## Persistence Strategy

Primary policy:

- Browser `localStorage` on `wasm32`
- `AppPersistence` is the only persistence API used by the interface layer

Current non-web behavior:

- returns `DomainError::InvalidOperation("Persistence is not yet supported on this platform")`
- app starts with empty in-memory state
- app shows a visible warning banner that the session is not persisted
- non-web export/import/clear-cache controls stay unavailable

Future:

- add native desktop/mobile persistence backend
- keep the same `AppPersistence` boundary

---

## Logging Strategy

The repo uses two separate logging paths:

- Build logging through PowerShell wrapper scripts under `scripts/`
- Runtime logging through `tracing` in `src/infrastructure/logging.rs`

Ownership rules:

- `application::execute` logs command start, success, and failure
- UI code may log UI-only concerns such as board load failures or invalid modal context
- UI code should not duplicate command failure logs already emitted by `execute`

---

## Bucket Validation Rules

1. `add_bucket(name)` rejects duplicate names on the same card.
2. `remove_bucket(id)` rejects removal of the Unassigned bucket.
3. `remove_bucket(id)` also rejects deletion while any child card still references that bucket.
4. `reorder_buckets(ids)` rejects duplicate ids, unknown ids, and omissions.

---

## Ordering Strategy

- Bucket order is the order of `Vec<Bucket>` on the parent card.
- Child order is the order of `Vec<CardId>` on the parent card.
- Root-card order is persisted explicitly in `CardRegistry`.

No extra ordering fields are used.

---

## Dioxus Interface Expectations (MVP)

### Theme and styling

- Tailwind CSS
- Dark mode by default
- Light mode available through a plain-text toggle
- Warm orange accent palette

### Navigation

- Click a card to drill into that card's board
- Use the `Back` control to move exactly one level up
- No breadcrumb tree in MVP
- Web builds expose working export/import/clear-cache controls
- Workspace-level label and rule managers live in the top navigation

### Board behavior

- Horizontal scrolling for board columns
- Vertical scrolling inside columns
- `CardItem` supports open, edit, delete, notes, due-date, label, and rule-aware display
- Cards and buckets support drag-and-drop reordering
- Root cards on Home support drag-and-drop reordering
- Cards may move between buckets inside the same board only

### Modal behavior

- Background blur
- Create card
- Edit card
- Create bucket
- Notebook-style notes modal
- Workspace label manager
- Workspace rule manager
- Inline validation if a child card is missing destination bucket context

### Error behavior

- Board load errors render a user-safe fallback panel
- Persistence failures surface a visible warning banner
- Command failures are logged in the application layer and surfaced in the relevant modal when appropriate
- Rule-triggered popups are queued FIFO so multiple matches do not collide

---

## Future Features (Not in MVP)

- Search and filtering across larger workspaces
- Rich-text or attachment support for notes
- More rule actions beyond popup notifications
- Native desktop/mobile persistence
- Recurrence
- Templates
- Event hooks beyond note-open, note-close, and bucket-entry popup behavior
- Multi-user features
- Permissions
- Analytics
- AI planning assistance
- Cloud sync

## Suggested Next Delivery Stages

### Stage 1: UI polish

- Refine drag/drop affordances now that cards can display more metadata
- Improve dense-card readability across light and dark themes
- Tighten modal spacing and settings ergonomics

### Stage 2: Rule expansion

- Add more rule actions once popup-only behavior feels stable
- Consider rule filters or workspace-level enable/disable states
- Evaluate whether rules need auditability or a recent-events panel

### Stage 3: Native persistence

- Add a desktop/mobile persistence backend behind `AppPersistence`
- Keep import/export compatible across browser and native targets
- Verify end-to-end desktop runtime behavior with the persisted backend
