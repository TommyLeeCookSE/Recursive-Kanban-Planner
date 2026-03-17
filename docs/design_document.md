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

The system is being implemented directly in Rust, with a strong emphasis on:
- strict typing
- explicit data flow
- modular architecture
- high performance
- long-term maintainability
- portability (including future WebAssembly support)

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
    id: CardId,               // unique identifier (UUID/ULID)
    title: String,            // display name
    parent_id: Option<CardId>,// parent reference
    children_ids: Vec<CardId>,// ordered list of children
    bucket: Option<BucketId>, // which bucket this card belongs to in its parent
    buckets: Vec<Bucket>,     // ordered bucket definitions for this card’s board
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
3. **Infrastructure Layer**: persistence (SQLite, file, etc.), serialization, storage adapters.
4. **Interface Layer (Future)**: web UI, API endpoints, WASM bindings.

## Persistence Strategy
Initial goals: local-first, no external server, single-user.
- **Primary target**: SQLite
- **Secondary**: JSON (for debugging/import/export)
- **Future**: browser-based storage (SQLite compiled to WebAssembly, persisted via IndexedDB or OPFS).
The persistence layer will be abstracted behind a repository interface (`Repository Pattern`) that loads, saves, fetches cards, and handles persistence mutations/transactions.

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
labels, deadlines, recurrence, templates, notes/pages, attachments, user assignments, multi-user collaboration, permissions, analytics, AI-assisted planning.

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

---

## Open Architecture Questions & Design Choices
*We must resolve these questions before restructuring the application. We will document the reasoning for each choice here.*

### 1. Data Types & Core Domain
* **Identifiers:** `Uuid` (v4) vs `Ulid` (time-sortable).
* **Bucket Entity:** Should `Bucket` be a raw `String` or a strict struct (`BucketId`, `name`) so renaming doesn't break relationships?
* **The Root Node:** Is there an implicit forest, or one singular "Root" workspace card that holds everything?
* **Ordering:** Are we storing explicit indices, or simply relying on the order of `children_ids: Vec<CardId>`?

### 2. Domain Invariants & Rules
* **Orphaned Cards:** If a parent is deleted, are children cascade-deleted or moved to root?
* **Bucket Deletion:** If a user deletes a Bucket, do we move cards to a default "Backlog" or reject the deletion?
* **Cycle Detection:** How do we efficiently prevent a card from becoming a child of its own descendant?

### 3. Application & Commands
* **Command Granularity:** Do we need atomic commands (e.g., `ReparentAndAssignBucket`) or keep them separate?
* **Board Loading:** Does navigating to a board load *only* immediate children, or the whole tree recursively?

### 4. Infrastructure & Persistence
* **Deployment Model:** Are we using Axum + Server-side SQLite (local desktop app via browser), or compiling SQLite to WASM for a pure in-browser OPFS storage?
* **Database Schema:** Do we store `children_ids` as JSON arrays, or strictly use Foreign Keys (`parent_id`, `sort_index`)?
* **Dependencies:** `sqlx` vs `rusqlite`? Use `thiserror` for domain errors?

### 5. Frontend & UI Architecture (Leptos)
* **Routing:** Should the route strictly be `/board/:card_id`?
* **State Management:** How are we caching the board state locally in Leptos Signals without creating stale data?
