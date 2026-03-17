# Recursive Kanban Planner — Design Document

## Core Philosophy: Cleanliness & Maintainability First
**This project must first and foremost prioritize clean, maintainable code over any other metric.**
We strictly enforce:
- **Maximum Type Safety:** Leverage Rust's type system to make invalid states unrepresentable.
- **Everything Explicit:** No magic, no hidden side-effects. Data flow and mutations must be crystal clear.
- **Minimal Duplication (DRY):** De-duplicate logic where appropriate while keeping boundaries intact.
- **Extremely Human Readable:** Code is written for humans first. Names and logic flows must be self-documenting.
- **Decoupled Architecture:** Strict separation of concerns (Domain → Application → Infrastructure → Interface).

---

## Project Overview
A recursive, card-based planning system. Everything is a `Card`. No separate entity types for tasks, projects, users, or teams.

**Stack:** Rust + Dioxus (WASM/desktop/mobile from one codebase). Local-first. No server. No auth.

---

## Current Implementation Status
*Last reconciled: 2026-03-17. Update this section whenever a phase is completed.*

### ✅ Implemented and Verified
| Item | File | Notes |
|---|---|---|
| `CardId` newtype (ULID) | `src/domain/id.rs` | Full docstrings, unit tests passing |
| `BucketId` newtype (ULID) | `src/domain/id.rs` | Full docstrings, unit tests passing |
| `Bucket` entity | `src/domain/bucket.rs` | Private fields, `new()`, `rename()`, `id()`, `name()` |
| `Card` entity | `src/domain/card.rs` | Two constructors, private fields, controlled mutators |
| Dioxus hello-world shell | `src/app.rs`, `src/main.rs` | Compiles, no real routing yet |

### ⚠️ Designed but Not Yet Enforced
| Item | Gap |
|---|---|
| Non-empty title invariant | `Card::rename` validates, but `Card::new_root` / `Card::new_child` do **not** reject blank titles at construction time |
| `bucket_id: None` only for root | Rule is documented; not enforced by the type system or constructor |
| Bucket reorder validation | `reorder_buckets` checks count but does **not** reject duplicate IDs in the input list |
| `DomainError` enum | Errors are currently `Result<_, String>` — no structured error type exists yet |

### 🔲 Not Yet Implemented
- `src/domain/error.rs` — `DomainError` enum
- `src/domain/registry.rs` — `CardRegistry` (multi-card invariant enforcement)
- `src/application/` — Command enum and dispatch
- `src/infrastructure/` — JSON serialization, localStorage adapter
- Real Dioxus routing (`/`, `/board/:card_id`)
- Any UI components beyond the hello-world shell

---

## Core Concept: Everything is a Card
Cards form a strict tree. Root cards have `parent_id: None`. All non-root cards have both a `parent_id` and a `bucket_id`. Depth is unlimited. Navigation = entering a card = viewing its board.

---

## Authoritative Domain Contract
*This is the binding specification. Implementation must match this exactly.*

### `DomainError`
```rust
// src/domain/error.rs
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Card not found: {0:?}")]
    CardNotFound(CardId),

    #[error("Bucket not found: {0:?}")]
    BucketNotFound(BucketId),

    #[error("Card title cannot be empty or blank")]
    EmptyTitle,

    #[error("A bucket named '{0}' already exists on this card")]
    DuplicateBucketName(String),

    #[error("Cannot delete a non-empty bucket; reassign or delete its cards first")]
    BucketNotEmpty,

    #[error("Cannot delete a card that still has children; choose a DeleteStrategy")]
    CardHasChildren,

    #[error("Reparenting would create a cycle in the card tree")]
    CycleDetected,
}
```

### `DeleteStrategy`
```rust
// src/domain/registry.rs
pub enum DeleteStrategy {
    /// Reject the deletion if the card has any children.
    Reject,
    /// Recursively delete the card and all its descendants.
    CascadeDelete,
    /// Move all immediate children to the deleted card's parent before deletion.
    ReparentToGrandparent,
}
```

### `CardRegistry` — Authoritative API
```rust
// src/domain/registry.rs
pub struct CardRegistry { /* HashMap<CardId, Card> — private */ }

impl CardRegistry {
    pub fn new() -> Self;

    // --- Creation ---
    pub fn create_root_card(&mut self, title: String) -> Result<CardId, DomainError>;
    pub fn create_child_card(&mut self, title: String, parent_id: CardId, bucket_id: BucketId) -> Result<CardId, DomainError>;

    // --- Reads ---
    pub fn get_card(&self, id: CardId) -> Result<&Card, DomainError>;
    pub fn get_root_cards(&self) -> Vec<&Card>;
    pub fn get_children(&self, parent_id: CardId) -> Result<Vec<&Card>, DomainError>;
    pub fn board_projection(&self, card_id: CardId) -> Result<HashMap<BucketId, Vec<&Card>>, DomainError>;

    // --- Single-card mutations (delegate to Card) ---
    pub fn rename_card(&mut self, id: CardId, title: String) -> Result<(), DomainError>;
    pub fn add_bucket(&mut self, card_id: CardId, name: String) -> Result<BucketId, DomainError>;
    pub fn reorder_buckets(&mut self, card_id: CardId, ordered_ids: Vec<BucketId>) -> Result<(), DomainError>;

    // --- Cross-card mutations (registry enforces, then delegates) ---
    pub fn move_card_to_bucket(&mut self, card_id: CardId, bucket_id: BucketId) -> Result<(), DomainError>;
    pub fn remove_bucket(&mut self, card_id: CardId, bucket_id: BucketId) -> Result<(), DomainError>;
    pub fn reparent_card(&mut self, card_id: CardId, new_parent_id: CardId) -> Result<(), DomainError>;
    pub fn delete_card(&mut self, card_id: CardId, strategy: DeleteStrategy) -> Result<(), DomainError>;

    // --- Internal ---
    fn detect_cycle(&self, card_id: CardId, proposed_parent_id: CardId) -> Result<(), DomainError>;
}
```

---

## Invariant Ownership

### Invariants owned by `Card` (self-contained, no external lookup needed)
| Invariant | Enforcement point |
|---|---|
| Title is non-empty and non-blank | `Card::new_root`, `Card::new_child`, `Card::rename` — all must reject at call time |
| Bucket names are unique per card | `Card::add_bucket` |
| Reorder list has no duplicates, no omissions, no unknown IDs | `Card::reorder_buckets` — must check all three conditions |
| The "Unassigned" bucket is never removable | `Card::remove_bucket` |

### Invariants owned by `CardRegistry` (require cross-card lookup)
| Invariant | Enforcement point |
|---|---|
| Parent card must exist before creating a child | `CardRegistry::create_child_card` |
| `bucket_id` must belong to the *parent's* bucket list, not the child's | `CardRegistry::create_child_card`, `CardRegistry::move_card_to_bucket` |
| No cycles in the hierarchy | `CardRegistry::reparent_card` (via `detect_cycle`) |
| A bucket cannot be deleted while children reference it | `CardRegistry::remove_bucket` (scan children's `bucket_id`) |
| A card cannot be deleted while it has children (unless strategy permits) | `CardRegistry::delete_card` |
| After reparenting, the card is moved to the new parent's Unassigned bucket | `CardRegistry::reparent_card` |
| Root cards (`parent_id: None`) must have `bucket_id: None` | `CardRegistry::create_root_card` (enforced structurally via `Card::new_root`) |

### Known Gap to Fix (P0)
`Card::reorder_buckets` currently validates only list length. It must also:
1. Reject duplicate IDs in the input
2. Reject IDs not present in `self.buckets`
These are self-contained checks and belong on `Card`, not the Registry.

---

## Architecture Overview
```
src/
├── domain/          ← Pure logic. No I/O. No Dioxus. No serde yet.
│   ├── id.rs        ← CardId, BucketId (ULID newtypes)
│   ├── bucket.rs    ← Bucket entity
│   ├── card.rs      ← Card entity + pub(crate) escape hatches
│   ├── error.rs     ← DomainError (NOT YET CREATED)
│   ├── registry.rs  ← CardRegistry (NOT YET CREATED)
│   └── mod.rs
├── application/     ← NOT YET CREATED. Command enum + dispatch + BoardView projection.
├── infrastructure/  ← NOT YET CREATED. JsonRepository + LocalStorageRepository.
├── app.rs           ← Dioxus root component (hello-world shell only)
├── components.rs    ← Empty placeholder
├── routes.rs        ← Empty placeholder
├── lib.rs
└── main.rs
```

---

## Persistence Strategy
- **Primary:** Browser `localStorage` / `IndexedDB`. Users warned that clearing cache deletes data.
- **Export/Import:** Download full state as JSON; re-upload to restore.
- **Future:** Google Drive / Dropbox optional sync.

Serde derives (`Serialize`, `Deserialize`) are **deferred** until the infrastructure layer is built. Do not add them to domain types prematurely.

---

## Bucket Validation Rules (Explicit)
1. `add_bucket(name)` — rejects if any existing bucket name matches case-insensitively.
2. `remove_bucket(id)` — `Card` rejects if it is the Unassigned bucket. `CardRegistry` additionally rejects if any child's `bucket_id` matches the target.
3. `reorder_buckets(ids)` — must reject if: (a) count differs, (b) any ID is duplicated in input, (c) any ID is unknown. All three checks must produce a clear `DomainError::BucketNotFound` or a distinct structural error.

---

## Ordering Strategy
- Bucket order = `Vec<Bucket>` position in the parent card. No separate field.
- Child order = `Vec<CardId>` position in the parent card. No separate field.

---

## Future Features (Not in MVP)
Labels, deadlines, recurrence, templates, notes, attachments, multi-user, permissions, analytics, AI planning, card cross-references/linking, cloud sync.

---

## Resolved Architecture Decisions
All binding. Not subject to re-debate.

| Decision | Resolution |
|---|---|
| Identifiers | ULIDs wrapped in `CardId` / `BucketId` newtypes |
| Bucket entity | `Bucket { id: BucketId, name: String }` — rename changes name only |
| Root node | No special type. Cards with `parent_id: None` are root. |
| Ordering | Vec position = order. No separate field. |
| Deletion | Default reject; explicit `DeleteStrategy` enum for cascade or reparent |
| Deployment | Dioxus (WASM + desktop + mobile from one codebase). No server. |
| Persistence | localStorage/IndexedDB primary; JSON export/import; future cloud |
| Framework | Dioxus. Leptos is fully removed. |
| Card linking | Deferred to post-MVP. |
