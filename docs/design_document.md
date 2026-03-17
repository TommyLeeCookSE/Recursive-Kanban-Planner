# Recursive Kanban Planner (Rust Implementation) - Design Document

## Core Philosophy: Cleanliness & Maintainability First
**This project must first and foremost prioritize clean, maintainable code over any other metric.** Extensibility, correctness, and developer experience are paramount. 
We strictly enforce:
- **Maximum Type Safety:** Leverage Rust's type system to make invalid states unrepresentable. 
- **Everything Explicit:** No magic, no hidden side-effects. Data flow and mutations must be crystal clear and traceable.
- **Minimal Duplication (DRY):** De-duplicate logic where appropriate while keeping boundaries intact.
- **Extremely Human Readable:** Code is written for humans first. Variable names, system boundaries, and logic flows must be self-documenting.
- **Decoupled Architecture:** Strict separation of concerns (Domain, Application, Infrastructure, Interface). Modules should communicate through clear interfaces.

## Project Overview
This project is a recursive, card-based planning system designed to unify task management across individuals, teams, households, and organizations into a single coherent model. The system is built around a strict abstraction: everything is a card. There are no separate entity types for tasks, projects, users, or teams. All of these are represented using the same structure and behavior.

The system is being implemented directly in Rust, compiled to WebAssembly (WASM), with a strong emphasis on:
- strict typing
- explicit data flow
- modular architecture
- high performance
- long-term maintainability
- portability (WASM runs in any modern browser on desktop, mobile, or tablet)

The goal is to build a local-first, web-based application that runs in the browser, stores data locally, and remains fast and responsive with minimal frontend complexity.

## Core Concept: Everything is a Card

The system uses a single fundamental entity: the `Card`.
A card can represent a task, a project, a sub-project, a user, a team, a department, an organization, a household, or any logical grouping. There are no special container types. Every node in the system is structurally identical.

### Recursive Hierarchy
Cards form a tree structure. Each card has:
- exactly one parent (except root)
- zero or more children

Example hierarchy: Organization → Department → Team → User → Project → Task
There is no limit to depth. This allows the same system to represent personal planning, team workflows, company structures, and nested project management.

### Boards Exist Inside Cards
Every card can act as a Kanban board. Each card defines its own list of buckets (columns) (e.g., Backlog, In Progress, Review, Done). The card’s children are grouped into these buckets.
This means:
- every card is both a node in the tree and a board
- navigation = entering a card and viewing its board

## Core Data Model
### `Card` (Rust struct)
The core entity will resemble:
```rust
struct Card {
    id: CardId,               // unique ULID identifier
    title: String,            // display name
    parent_id: Option<CardId>,// parent reference (None = root/top-level card)
    children_ids: Vec<CardId>,// ordered list of children
    bucket: Option<BucketId>, // which bucket this card belongs to in its parent
    buckets: Vec<Bucket>,     // ordered bucket definitions for this card's board
}
```

## Core Rules and Invariants
These must always hold:
1. Each card has at most one parent
2. No cycles in the hierarchy
3. All children_ids must exist
4. A child cannot reference itself
5. A child’s bucket must exist in the parent’s buckets
6. Bucket names are unique per card
7. Ordering of children is preserved
8. Ordering of buckets is preserved
Invalid state is not allowed to exist in memory.

## Board Behavior & Navigation Model
- Each card produces a board view: columns = buckets, items = child cards grouped by bucket.
- Board rendering is a projection, not stored separately.
- Navigation is hierarchical: open card → view its board, go to parent → move up, traverse children → move down.
- There is no global flat task list. Everything is contextual.

## Command-Based Architecture
All mutations happen through explicit commands enforcing invariants:
- `CreateCard`, `RenameCard`, `DeleteCard`
- `MoveCardToBucket`, `ReparentCard`, `ReorderCardWithinBucket`
- `CreateBucket`, `RenameBucket`, `DeleteBucket`, `ReorderBuckets`
No direct mutation of structs from outside the domain layer.

## Architecture Overview
The system follows a clean, layered architecture:
1. **Domain Layer**: `Card`, `CardId`, `Bucket`, invariants, domain errors. Pure logic. No I/O.
2. **Application Layer**: command handlers, orchestration logic, board projections, navigation helpers. Uses domain objects but does not handle persistence directly.
3. **Infrastructure Layer**: JSON serialization, browser storage adapters (`localStorage` / `IndexedDB`), export/import handlers.
4. **Interface Layer**: Leptos CSR components, client-side routing, board rendering.

## Persistence Strategy
Initial goals: local-first, no external server, single-user, pure browser.
- **Primary**: Browser `localStorage` / `IndexedDB` for session persistence. Users are warned that clearing cache will delete data.
- **Export/Import**: Users can download their entire state as a JSON file and re-upload it to restore.
- **Future**: Optional cloud sync via Google Drive / Dropbox API integration.
The persistence layer is abstracted behind a repository interface (`Repository Pattern`) that loads, saves, and fetches cards from browser storage.

## Ordering Strategy
- **Bucket ordering**: defined by `Vec<Bucket>`, stable and explicit.
- **Card ordering within bucket**: simple index-based ordering (initial), more advanced ordering later if needed. Must be deterministic and preserved.

## Error Handling
Explicit domain errors: `CardNotFound`, `InvalidParent`, `CycleDetected`, `InvalidBucket`, `DuplicateBucket`, `InvalidOperation`. Errors are structured and handled at boundaries.

## Local-First Design
System is designed to work without a backend server, without authentication, without cloud sync. All data lives locally. Future expansion may include sync layer, collaboration, multi-device support.

## Performance Model
Assumptions: entire card registry can fit in memory, board views computed on demand, no premature caching, simple algorithms first, optimize later. Rust ensures predictable performance.

## Future Features (Not in MVP)
labels, deadlines, recurrence, templates, notes/pages, attachments, user assignments, multi-user collaboration, permissions, analytics, AI-assisted planning, card cross-references/linking, cloud sync (Google Drive/Dropbox).

## Design Philosophy
This system prioritizes:
- simplicity over feature sprawl
- strict invariants over flexibility
- explicit data flow over magic
- composability over specialization
- local-first over cloud dependency

## MVP Goal
The first working version must support:
- create cards
- define buckets
- move cards between buckets
- reparent cards
- navigate hierarchy
- persist and reload state
If this works cleanly and reliably, the architecture is validated. Everything else is secondary.

## Resolved Architecture Decisions
*These decisions were finalized during the planning phase and are now binding for all implementation work.*

### 1. Data Types & Core Domain ✅
* **Identifiers:** `Ulid` (time-sortable). Wrapped in strict Newtype structs (`CardId`, `BucketId`) to prevent accidental mixing. **Implemented in `src/domain/id.rs`.**
* **Bucket Entity:** Strict struct (`Bucket { id: BucketId, name: String }`). Renaming a bucket changes only the display name; all child card references use the stable `BucketId`. This guarantees referential integrity.
* **The Root Node:** There is NO special "Workspace" type. Top-level cards are simply cards with `parent_id: None`. "Everything is a Card" — a card named "Home" or "Work" is structurally identical to a task card. Hierarchy is traversed from whichever root card is selected.
* **Ordering:** `children_ids: Vec<CardId>` preserves insertion order implicitly. Reordering is handled by mutating the Vec directly.

### 2. Domain Invariants & Rules ✅
* **Orphaned Cards (Deletion):** **Option C with bypass.** By default, the system rejects deletion of a card that still has children. The user must explicitly choose to:
  * (A) Cascade-delete all children recursively, OR
  * (B) Reparent all children to the deleted card's parent.
* **Bucket Deletion:** Same principle — reject deletion of a bucket that still has cards assigned to it. User must reassign or delete the cards first.
* **Cycle Detection:** Must be enforced. Before any reparent operation, walk up the ancestor chain of the proposed new parent to ensure the card being moved is not an ancestor of the target. Since the entire registry fits in memory, this is a simple traversal.

### 3. Application & Commands ✅
* **Command Granularity:** Keep commands atomic and separate (e.g., `ReparentCard` and `MoveCardToBucket` are distinct). Composite operations are orchestrated at the Application layer, not baked into a single command.
* **Board Loading:** Load only immediate children when navigating to a card's board. Deep tree loading is deferred until the user navigates deeper.

### 4. Infrastructure & Persistence ✅
* **Deployment Model: Pure WASM Browser App.** No Axum server. No Tokio runtime. Leptos runs in CSR (Client-Side Rendering) mode only. The entire Rust application compiles to WebAssembly and runs directly in the user's browser.
* **Persistence Strategy:**
  * **Primary:** Browser `localStorage` / `IndexedDB` for session persistence. Users will be warned that clearing browser cache will delete their data.
  * **Export/Import:** Users can download their entire state as a JSON file and re-upload it to restore.
  * **Future:** Optional cloud sync via Google Drive / Dropbox API integration.
* **Dependencies to REMOVE:** `axum`, `tokio`, `leptos_axum`, `tracing`, `tracing-subscriber`. These are server-side dependencies that are unnecessary in a pure WASM app.
* **Dependencies to KEEP/ADD:** `leptos` (CSR features only), `serde`, `serde_json`, `ulid`, `thiserror` (for domain errors), `web-sys` / `wasm-bindgen` (for browser storage APIs).

### 5. Frontend & UI Architecture (Leptos) ✅
* **Routing:** `/board/:card_id` maps directly to "enter a card and view its board". Top-level route `/` shows the list of root cards (those with `parent_id: None`).
* **State Management:** The entire card registry lives in a Leptos `RwSignal` in memory. Board views are computed projections from this signal. Mutations go through Commands which update the signal and trigger re-renders.

## Open Questions (Remaining)
*All architectural questions have been resolved. No open blockers remain.*

* **Card Linking (Cross-References):** Deferred to **Future**. Cards will be able to link/reference other cards outside the parent-child tree (e.g., shortcuts, bookmarks). Not part of MVP.

