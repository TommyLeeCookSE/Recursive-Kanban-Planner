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
The workspace is the single top-level card, and every other board is a normal child card.
There are no separate bucket, task, or project entities.

Stack:

- Rust
- Dioxus
- WASM, desktop, and mobile targets from one codebase
- No server
- No auth

---

## Current Implementation Status

Last reconciled: 2026-03-19

Canonical verification command:

- `pwsh ./scripts/test-all.ps1`

That script runs:

- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all`
- `cargo test --doc`
- `cargo check --target wasm32-unknown-unknown`
- `cargo check --no-default-features --features desktop`
- `npm run check:css`

Implemented in the current worktree:

| Item | File | Notes |
| :--- | :--- | :--- |
| `CardId` and feature id newtypes | `src/domain/id.rs` | ULID-backed ids for cards, notes, and rules |
| `Card` entity | `src/domain/card.rs` | Local invariants enforced inside the entity, including notes and due dates |
| `NotePage`, `DueDate` | `src/domain/note.rs`, `src/domain/due_date.rs` | Feature-specific value objects and definitions |
| `DomainError` | `src/domain/error.rs` | Typed domain errors throughout the domain/application layers |
| `CardRegistry` | `src/domain/registry.rs` | Cross-card invariant boundary plus ordered tree management |
| `Command` dispatcher | `src/application/mod.rs` | `execute` owns command lifecycle logging |
| `BoardView`, `CardPreviewView` | `src/application/mod.rs` | Read-only UI projections |
| JSON persistence | `src/infrastructure/repository.rs` | Serialize/deserialize full registry state |
| Browser persistence | `src/infrastructure/repository.rs` | `localStorage` on `wasm32` |
| `AppPersistence` facade | `src/infrastructure/repository.rs` | Explicit browser-first persistence boundary |
| Runtime logging | `src/infrastructure/logging.rs` | `tracing`, diagnostics buffer, native/web setup |
| Dioxus shell and routing | `src/interface/` | App shell, routes, modal system, board/home views |
| `TopBar`, `CardItem`, modal flows | `src/interface/components/` | Create/edit card, notes, and settings UI |
| Deterministic child-card creation | `src/interface/components/modal.rs` | Child creation requires a real parent card |
| Fail-loud board fallback | `src/interface/routes/board.rs` | Board load preserves the real `DomainError` for logs |
| Public API docs with examples | `src/application/`, `src/infrastructure/`, `src/interface/` | Public entry points now have `# Examples` blocks |
| Dynamic "Smart" Navigation | `src/interface/components/layout.rs` | Content-aware auto-collapse based on real-time measurement |

Not yet implemented or not yet fully verified:

- Desktop-target `dx serve --platform desktop` runtime verification
- Native desktop/mobile persistence backend beyond browser storage
- Manual browser sanity pass across the newer notes, due-date, and drag/drop flows
- Release tagging workflow

Recent binding decisions already implemented:

1. Same-parent reparenting is a no-op.
2. Registry read-path corruption fails loudly.
3. Child-card creation in the UI must never synthesize a fallback id.
4. The board route must keep the underlying `DomainError` for logs and show a user-safe fallback.
5. Command lifecycle logging belongs to `application::execute`.
6. Split navigation: Action bar at bottom, centered title at top.
7. "Math-based" responsive: Measure rendered content to toggle icon-only labels.

---

## Core Concept: Cards Only

The current shipped model is intentionally recursive and card-only.

- `Card` is the primary planning item and board owner.
- A card's children are ordered child cards, not bucket columns.
- Opening a card means viewing that card's immediate children.

Cards form a strict tree:

- The workspace card has `parent_id: None`.
- Nested cards have `parent_id: Some(...)`.
- Depth is unlimited.
- Navigation means entering a card and viewing that card's board.

Child previews are compact and shallow:

- a card shows only its immediate children
- previews are rendered as small card boxes or chips
- recursive preview nesting is intentionally limited to one level in the UI

---

## Authoritative Domain Contract

This section is the source-of-truth contract for the implemented domain layer.

### `DomainError`

```rust
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DomainError {
    CardNotFound(CardId),
    EmptyTitle,
    CardHasChildren,
    CycleDetected,
    InvalidOperation(String),
    IncompatibleLegacyData(String),
}
```

### `CardRegistry`

```rust
pub struct CardRegistry { /* store is private */ }

impl CardRegistry {
    pub fn new() -> Self;

    pub fn workspace_card(&self) -> Result<&Card, DomainError>;
    pub fn workspace_card_id(&self) -> Result<CardId, DomainError>;
    pub fn get_card(&self, id: CardId) -> Result<&Card, DomainError>;
    pub fn get_card_mut(&mut self, id: CardId) -> Result<&mut Card, DomainError>;
    pub fn get_children(&self, parent_id: CardId) -> Result<Vec<&Card>, DomainError>;
    pub fn workspace_child_count(&self) -> usize;
    pub fn validate(&self) -> Result<(), DomainError>;

    pub fn create_workspace_child_card(&mut self, title: String) -> Result<CardId, DomainError>;
    pub fn create_child_card(&mut self, title: String, parent_id: CardId) -> Result<CardId, DomainError>;
    pub fn rename_card(&mut self, id: CardId, title: String) -> Result<(), DomainError>;
    
    pub fn add_note_page(&mut self, card_id: CardId, title: String) -> Result<NotePageId, DomainError>;
    pub fn rename_note_page(&mut self, card_id: CardId, note_page_id: NotePageId, title: String) -> Result<(), DomainError>;
    pub fn save_note_page_body(&mut self, card_id: CardId, note_page_id: NotePageId, body: String) -> Result<(), DomainError>;
    pub fn delete_note_page(&mut self, card_id: CardId, note_page_id: NotePageId) -> Result<(), DomainError>;
    
    pub fn set_due_date(&mut self, card_id: CardId, due_date: DueDate) -> Result<(), DomainError>;
    pub fn clear_due_date(&mut self, card_id: CardId) -> Result<(), DomainError>;

    pub fn reparent_card(&mut self, card_id: CardId, new_parent_id: CardId) -> Result<(), DomainError>;
    pub fn reorder_children(&mut self, parent_id: CardId, ordered_ids: Vec<CardId>) -> Result<(), DomainError>;
    pub fn delete_card(&mut self, card_id: CardId, strategy: DeleteStrategy) -> Result<(), DomainError>;
}
```

### `Command` and Dispatcher

```rust
pub enum Command {
    CreateWorkspaceChildCard { title: String },
    CreateChildCard { title: String, parent_id: CardId },
    RenameCard { id: CardId, title: String },
    AddNotePage { card_id: CardId, title: String },
    RenameNotePage { card_id: CardId, note_page_id: NotePageId, title: String },
    SaveNotePageBody { card_id: CardId, note_page_id: NotePageId, body: String },
    DeleteNotePage { card_id: CardId, note_page_id: NotePageId },
    SetDueDate { card_id: CardId, due_date: DueDate },
    ClearDueDate { card_id: CardId },
    DeleteCard { id: CardId, strategy: DeleteStrategy },
    ReparentCard { card_id: CardId, new_parent_id: CardId },
    ReorderChildren { parent_id: CardId, ordered_ids: Vec<CardId> },
    DropChildAtPosition { parent_id: CardId, card_id: CardId, target_index: usize },
}

pub fn execute(command: Command, registry: &mut CardRegistry) -> Result<(), DomainError>;
```

### Projections

```rust
pub struct BoardView<'a> {
    pub card: &'a Card,
    pub children: Vec<&'a Card>,
}

pub struct CardPreviewView<'a> {
    pub card: &'a Card,
    pub children: Vec<&'a Card>,
}

pub fn build_board_view(card_id: CardId, registry: &CardRegistry) -> Result<BoardView<'_>, DomainError>;
pub fn build_card_preview_view(card_id: CardId, registry: &CardRegistry) -> Result<CardPreviewView<'_>, DomainError>;
```

---

## Invariant Ownership

### `Card` owns local invariants

These do not require cross-card lookup:

- Title must be non-empty in constructors and `rename`
- Child reorder must reject duplicates, omissions, and unknown ids
- Notes and due dates validate their own local shape

### `CardRegistry` owns cross-card invariants

These require looking across cards:

- Workspace card must exist before board projections are used
- Parent card must exist before creating a child
- Reparenting must reject cycles
- Delete strategies must be enforced at the registry level
- Same-parent reparenting must return `Ok(())` without mutating ordering
- Corrupt read paths must fail loudly rather than skipping data or silently falling back

---

## Resolved Behavioral Decisions

### Same-parent reparenting

`reparent_card(id, current_parent)` is a no-op that returns `Ok(())`.
It must preserve:

- child ordering
- `parent_id`

Regression test:

- `test_reparent_to_same_parent_is_noop`

### Read-path corruption policy

The registry fails loudly if:

- a parent references a missing child card
- a child tree is corrupt during readback or projection

Error shapes:

- `DomainError::CardNotFound`
- `DomainError::InvalidOperation`

Regression tests:

- `test_get_children_fails_on_missing_child`
- `test_board_projection_fails_on_corrupt_tree`

### Interface determinism

The interface layer must not invent domain identifiers.

Current enforced behavior:

- child-card creation without a selected parent is rejected before any command is built
- the modal stays open and shows inline validation feedback
- the application layer remains the single owner of command logging
- note-open and note-close events are routed through the shared modal dispatcher in `App`

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
- legacy bucket-based snapshots are treated as incompatible and reset to a clean workspace
- workspace data is normalized to the current card-only shape before the UI uses it

---

## Architecture Overview

```text
src/
|- domain/          <- Pure domain logic, no I/O
|  |- id.rs
|  |- card.rs
|  |- due_date.rs
|  |- note.rs
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

### Styling Workflow

- `src/interface/tailwind.css` is the editable source of truth for theme and UI styling.
- `assets/app.css` is generated output and should be refreshed with `npm run build:css`.
- `npm run check:css` validates that the generated stylesheet matches the committed asset.
- Theme, surface, and action classes should be defined semantically so both light and dark modes stay in sync.

### Browser Smoke Coverage

- `tests/smoke.spec.ts` provides a small Playwright smoke test for app boot, workspace creation, card creation, child preview visibility, theme toggle, and top-nav utility visibility.
- The smoke test is intended as a fast regression check, not a full end-to-end suite.
- `npm run smoke` runs the CSS parity check and then the smoke suite.

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

## Planned Hotspot Decomposition

These changes are planned-only and must preserve current behavior:

- card-only domain model stays intact
- workspace-first routing stays intact
- command names and lifecycle logging stay stable
- no new user-facing flows are introduced

### 1. `board_view/grid`

Current problem:

- `src/interface/components/board_view/grid.rs` currently mixes layout composition, card rendering, drag lifecycle, drop-zone rendering, navigation, modal dispatch, delete flows, and diagnostics.

Target split:

- `src/interface/components/board_view/grid/mod.rs`
  - keeps `render_board_grid(...)` as the only entry point used by `board_view.rs`
  - owns only grid composition and slot sequencing
  - keeps the simple `render_card_slot(...)` wrapper because it is pure layout glue
- `src/interface/components/board_view/grid/card.rs`
  - owns `render_board_card(...)`
  - owns board-card interactions only: open board route, open edit modal, start drag, end drag, delete card
  - may use private helper functions for drag-start diagnostics and drag-state reset, but exposes no new public API
- `src/interface/components/board_view/grid/drop_zone.rs`
  - owns `render_empty_board_drop_zone(...)`
  - owns `render_board_drop_zone(...)`
  - owns drop-zone presentation selection, hover-state updates, dragged-card extraction, and transition-wrapped `DropChildAtPosition` dispatch
  - keeps the duplicated empty/non-empty drop logic out of `mod.rs`

Implementation notes:

- keep `BoardRenderContext` in `src/interface/components/board_view/models.rs` for now; do not widen its public surface unless a private helper struct is clearly needed
- if the Implementer needs an extra seam for testing, prefer a private `DropZoneSpec` or pure class-selection helper inside `drop_zone.rs` instead of creating more files
- `board_view.rs` should continue importing only `render_board_grid(...)`; all new files remain private to the `board_view::grid` module

### 2. `application/command` + `application/execute`

Current problem:

- the command catalog is maintained separately in the enum definition, `name()`, `log_start()`, and the `execute(...)` match
- child-drop policy lives in `application::execute` even though it is really ordered-child domain behavior

Target split:

- `src/application/command/mod.rs`
  - remains the home of the public `Command` enum
  - no public command names change
- `src/application/command/dispatch.rs`
  - becomes the single authoritative command catalog
  - owns one `match` over `Command`
  - each branch does three things together: emit the existing start log fields, call the correct domain/application operation, and return the stable command label used by `execute`
  - this removes the need for separate `Command::name()` and `Command::log_start()` tables
- `src/application/execute.rs`
  - becomes a thin lifecycle wrapper only
  - responsibilities: call dispatch, emit generic success/failure logs, write diagnostics on failure, return the domain result

Child-drop relocation:

- add `CardRegistry::drop_child_at_position(parent_id, card_id, target_index)` as the domain entry point
- move the remove-clamp-insert policy out of `application::execute`
- inside the domain, prefer:
  - `CardRegistry` for parent lookup and cross-card error ownership
  - `Card` for the local ordered-children mutation helper if the Implementer wants the reorder algorithm fully owned by the entity
- `Command::DropChildAtPosition` continues to exist with the same name, but its dispatch branch should delegate directly to the new registry method

Why this is the minimal stable shape:

- the public application surface stays `application::Command` plus `application::execute`
- only one place enumerates the command catalog for behavior and start-logging
- child-drop semantics become domain-owned without changing UI routing or drag/drop command names

### Public API and test impact

Public API changes the Implementer should expect:

- `CardRegistry` gains a new public ordered-child operation for drop-at-position
- `application::Command` remains externally unchanged
- `application::execute(...)` remains externally unchanged
- `board_view` public entry points remain unchanged; the split is internal module structure only

Tests the Implementer should add or move:

- domain tests for the new child-drop entry point:
  - moves an existing child to a lower index
  - clamps insertion past the end
  - rejects dropping a non-child into the parent
- if a `Card`-level helper is introduced, add direct entity tests there for the reorder algorithm
- keep one application-level regression test proving `execute(Command::DropChildAtPosition { .. })` still works end-to-end
- for the grid split, prefer small tests around any extracted pure helper/spec logic rather than brittle RSX snapshot tests

Non-goals for this decomposition:

- no bucket/column model
- no routing changes away from workspace-first navigation
- no drag payload format redesign unless needed purely as a private helper extraction
- no modal or command-name changes

---

## Ordering Strategy

- Child order is the order of `Vec<CardId>` on the parent card.
- The workspace card is the single top-level card in the tree.

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
- The workspace back button is visible but disabled
- No breadcrumb tree in MVP
- Web builds expose working export/import/clear-cache controls

### Board behavior

- The board uses a wrapping grid instead of horizontal scrolling
- Vertical scrolling is allowed inside the viewport
- `CardItem` supports open, edit, delete, notes, due-date, and preview display
- Cards may move within their parent ordering
- Cards may move between parents only when the UI explicitly supports it

### Modal behavior

- Background blur
- Create card
- Edit card
- Notebook-style notes modal
- Inline validation when required parent context is missing

### Error behavior

- Board load errors render a user-safe fallback panel
- Persistence failures surface a visible warning banner
- Command failures are logged in the application layer and surfaced in the relevant modal when appropriate

---

## Future Features (Not in MVP)

- Search and filtering across larger workspaces
- Rich-text or attachment support for notes
- More rule actions beyond popup notifications
- Native desktop/mobile persistence
- Recurrence
- Templates
- Multi-user features
- Permissions
- Analytics
- AI planning assistance
- Cloud sync

## Suggested Next Delivery Stages

### Stage 1: UI polish

- Refine drag/drop affordances now that cards carry more metadata
- Improve dense-card readability across light and dark themes
- Tighten modal spacing and settings ergonomics

### Stage 2: Search and navigation

- Add search and filtering once large workspaces become hard to scan
- Consider shortcuts or quick-open behavior for deep trees

### Stage 3: Native persistence

- Add a desktop/mobile persistence backend behind `AppPersistence`
- Keep import/export compatible across browser and native targets
- Verify end-to-end desktop runtime behavior with the persisted backend
