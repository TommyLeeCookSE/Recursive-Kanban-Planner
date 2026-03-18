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
| :--- | :--- | :--- |
| `CardId` newtype (ULID) | `src/domain/id.rs` | Full docstrings, unit tests, `PartialOrd`/`Ord` derived |
| `BucketId` newtype (ULID) | `src/domain/id.rs` | Full docstrings, unit tests, `PartialOrd`/`Ord` derived |
| `Bucket` entity | `src/domain/bucket.rs` | Private fields, `new()`, `rename()`, `id()`, `name()` |
| `Card` entity | `src/domain/card.rs` | Two constructors, private fields, controlled mutators |
| Non-empty title validation | `src/domain/card.rs` | Enforced at construction **and** rename — `DomainError::EmptyTitle` |
| Strict bucket reorder validation | `src/domain/card.rs` | Rejects duplicate IDs (`DuplicateBucketId`) and unknown IDs (`BucketNotFound`) |
| `DomainError` enum | `src/domain/error.rs` | Full variant set; all `Result<_, String>` replaced |
| `CardRegistry` | `src/domain/registry.rs` | Full implementation; see API contract below |
| `DeleteStrategy` enum | `src/domain/registry.rs` | `Reject`, `CascadeDelete`, `ReparentToGrandparent` |
| `Command` enum and `execute` | `src/application/mod.rs` | Full dispatcher for all registry mutations |
| `BoardView` / `ColumnView` | `src/application/mod.rs` | Read-only projections for UI rendering |
| Serde support | `src/domain/*.rs` | `Serialize`/`Deserialize` derives added to all domain types |
| JsonRepository | `src/infrastructure/mod.rs` | Basic registry persistence |
| Dioxus Interface | `src/interface/` | `app`, `components`, `routes`, `error_templates` |

### 🔲 Not Yet Implemented

- Real Dioxus routing (`/`, `/board/:card_id`)
- Any UI components beyond the hello-world shell
- `LocalStorageRepository` for browser persistence

### ⚠️ Open Behavioral Decisions (spec gaps the next implementer must resolve in code)

See "Unresolved Architectural Decisions" below. Two specific behaviors in `registry.rs` have been
left in a tolerated-but-undocumented state and must be given explicit, tested contracts:

1. **Same-parent reparenting** — `reparent_card(id, current_parent)` currently runs full logic
   as if it were a real move (re-appending child, moving to Unassigned). The contract must specify:
   no-op returning `Ok(())` OR a distinct short-circuit behavior.
2. **Read-path corruption** — `get_children` and `board_projection` currently silently skip
   missing children and fall back to Unassigned for unknown bucket IDs. The contract must be
   explicit about whether this is fail-loud or tolerated.

---

## Core Concept: Everything is a Card

Cards form a strict tree. Root cards have `parent_id: None`. All non-root cards have both a `parent_id` and a `bucket_id`. Depth is unlimited. Navigation = entering a card = viewing its board.

---

## Authoritative Domain Contract

*This is the binding specification. Implementation must match this exactly.*

### `DomainError`

```rust
// src/domain/error.rs  — IMPLEMENTED
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("Card not found: {0}")]
    CardNotFound(CardId),

    #[error("Bucket not found: {0}")]
    BucketNotFound(BucketId),

    #[error("Duplicate bucket ID during reorder: {0}")]
    DuplicateBucketId(BucketId),

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

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}
```

### `DeleteStrategy`

```rust
// src/domain/registry.rs  — IMPLEMENTED
pub enum DeleteStrategy {
    /// Reject the deletion if the card has any children.
    Reject,
    /// Recursively delete the card and all its descendants.
    CascadeDelete,
    /// Move all immediate children to the deleted card's parent before deletion.
    /// Returns `DomainError::InvalidOperation` if the card being deleted is a root card
    /// (no grandparent to reparent to).
    ReparentToGrandparent,
}
```

### `CardRegistry` — Authoritative API

```rust
// src/domain/registry.rs  — IMPLEMENTED
pub struct CardRegistry { /* HashMap<CardId, Card> — private */ }

impl CardRegistry {
    pub fn new() -> Self;

    // --- Creation ---
    pub fn create_root_card(&mut self, title: String) -> Result<CardId, DomainError>;
    pub fn create_child_card(&mut self, title: String, parent_id: CardId, bucket_id: BucketId) -> Result<CardId, DomainError>;

    // --- Reads ---
    pub fn get_card(&self, id: CardId) -> Result<&Card, DomainError>;
    pub fn get_root_cards(&self) -> Vec<&Card>; // sorted by ULID for determinism
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
| :--- | :--- |
| Title is non-empty and non-blank | `Card::new_root`, `Card::new_child`, `Card::rename` — all enforced |
| Bucket names are unique per card | `Card::add_bucket` |
| Reorder list has no duplicates, no omissions, no unknown IDs | `Card::reorder_buckets` — all three enforced |
| The "Unassigned" bucket is never removable | `Card::remove_bucket` |

### Invariants owned by `CardRegistry` (require cross-card lookup)

| Invariant | Enforcement point |
| :--- | :--- |
| Parent card must exist before creating a child | `CardRegistry::create_child_card` |
| `bucket_id` must belong to the *parent's* bucket list | `CardRegistry::create_child_card`, `CardRegistry::move_card_to_bucket` |
| No cycles in the hierarchy | `CardRegistry::reparent_card` (via `detect_cycle`) |
| A bucket cannot be deleted while children reference it | `CardRegistry::remove_bucket` (scans children's `bucket_id`) |
| A card cannot be deleted while it has children (unless strategy permits) | `CardRegistry::delete_card` |
| After reparenting, the card is moved to the new parent's Unassigned bucket | `CardRegistry::reparent_card` |
| Root cards (`parent_id: None`) must have `bucket_id: None` | `CardRegistry::create_root_card` (enforced by `Card::new_root`) |

---

## Unresolved Architectural Decisions

*These are open specs. Do not code them in without recording the decision here first.*

### 1. Same-Parent Reparenting Contract

**The question:** What should `reparent_card(card_id, new_parent_id)` do when `new_parent_id` is
already the card's current parent?

**Decision (binding):**
`reparent_card` MUST be a no-op when `new_parent_id == card.parent_id()`. It returns `Ok(())`
immediately, preserving child ordering and bucket assignment unchanged.

**Required change:** Add an early-return guard after existence validation:

```rust
if card.parent_id() == Some(new_parent_id) {
    return Ok(());
}
```

**Required test:** `test_reparent_to_same_parent_is_noop` — verify child ordering and bucket
assignment are unchanged after the call.

---

### 2. Read-Path Corruption Policy

**The question:** When `get_children` or `board_projection` encounters an internally inconsistent
state (a parent's `children_ids` list references a `CardId` not in the store, or a child's
`bucket_id` is not in the parent's bucket list), should the registry fail loudly or silently
degrade?

**Current implementation:** Silent tolerance — missing children are skipped; unknown bucket IDs
fall back to Unassigned.

**Decision (binding):** **Fail loudly.**

The silent fallback masks bugs introduced by incorrect registry mutations. The registry MUST
maintain the invariant that every ID in a `children_ids` list resolves to a stored card, and
every child's `bucket_id` is valid on its parent. If this invariant is violated, it is a bug in
the registry code, not a legitimate data state. The read path should surface it immediately.

**Required changes:**

1. `get_children`: replace the `if let Some(child)` guard with a hard failure:

   ```rust
   // Instead of silently skipping:
   if let Some(child) = self.store.get(child_id) { ... }
   // Use:
   let child = self.get_card(*child_id)?;  // returns DomainError::CardNotFound if inconsistent
   ```

2. `board_projection`: replace the Unassigned fallback for unknown bucket IDs with a hard failure:

   ```rust
   // Instead of falling back:
   let target_bucket_id = match child.bucket_id() {
       Some(id) if projection.contains_key(&id) => id,
       _ => unassigned_bucket_id,  // REMOVE THIS fallback
   };
   // Use:
   let target_bucket_id = child.bucket_id()
       .filter(|id| projection.contains_key(id))
       .ok_or_else(|| DomainError::BucketNotFound(child.bucket_id().unwrap_or_default()))?;
   ```

3. No new `DomainError` variant is needed. `CardNotFound` and `BucketNotFound` cover both cases.

**Why not tolerate:** There is no legitimate code path by which a stored card's `bucket_id` can
reference a bucket that does not exist on its parent, if all mutations go through `CardRegistry`.
Tolerating it silently makes bugs invisible at the read layer that would only surface as rendering
anomalies in the UI, which are extremely hard to trace.

---

## Architecture Overview

```text
src/
├── domain/          ← Pure logic. No I/O.
│   ├── id.rs
│   ├── bucket.rs
│   ├── card.rs
│   ├── error.rs
│   ├── registry.rs
│   └── mod.rs
├── application/     ← Commands & Projections.
│   └── mod.rs
├── infrastructure/  ← JSON & localStorage adapters.
│   └── mod.rs
├── interface/       ← Dioxus UI Layer.
│   ├── app.rs
│   ├── components.rs
│   ├── routes.rs
│   ├── error_templates.rs
│   └── mod.rs
├── lib.rs           ← Crate root (exports all layers).
└── main.rs          ← Binary entry point.
```

---

## Persistence Strategy

- **Primary:** Browser `localStorage` / `IndexedDB`. Users warned that clearing cache deletes data.
- **Export/Import:** Download full state as JSON; re-upload to restore.
- **Future:** Google Drive / Dropbox optional sync.

Serde derives (`Serialize`, `Deserialize`) are now **implemented** across all domain types.
Verify WASM compatibility during the next infrastructure pass.

---

## Bucket Validation Rules (Explicit)

1. `add_bucket(name)` — rejects if any existing bucket name matches case-insensitively → `DuplicateBucketName`.
2. `remove_bucket(id)` — `Card` rejects if it is the Unassigned bucket → `InvalidOperation`. `CardRegistry`
   additionally rejects if any child's `bucket_id` matches the target → `BucketNotEmpty`.
3. `reorder_buckets(ids)` — rejects if: (a) count differs → `InvalidOperation`, (b) any ID is
   duplicated → `DuplicateBucketId`, (c) any ID is unknown → `BucketNotFound`. All three enforced.

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
| :--- | :--- |
| Identifiers | ULIDs wrapped in `CardId` / `BucketId` newtypes |
| Bucket entity | `Bucket { id: BucketId, name: String }` — rename changes name only |
| Root node | No special type. Cards with `parent_id: None` are root. |
| Ordering | Vec position = order. No separate field. |
| Deletion | Default reject; explicit `DeleteStrategy` enum for cascade or reparent |
| Deployment | Dioxus (WASM + desktop + mobile from one codebase). No server. |
| Persistence | localStorage/IndexedDB primary; JSON export/import; future cloud |
| Framework | Dioxus. Leptos is fully removed. |
| Card linking | Deferred to post-MVP. |
| Same-parent reparent | No-op, returns `Ok(())`. Does not reset bucket or ordering. |
| Read-path corruption | Fail loudly. `CardNotFound` / `BucketNotFound` are the error shapes. |
