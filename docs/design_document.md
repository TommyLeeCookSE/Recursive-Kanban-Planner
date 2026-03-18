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

**Current validation state:** The current worktree is **verification-clean**.
`cargo test --all`, `cargo clippy --all-targets -- -D warnings`, and
`cargo fmt -- --check` all pass.

### ✅ Implemented in Current Worktree

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
| Dioxus Interface | `src/interface/` | `app`, `components`, `routes`, `error_templates` exist; interface layer is clippy/rustfmt clean |
| `TopBar`, `CardItem`, rename/create modals | `src/interface/components/` | Reusable board header, interactive card controls, blurred modal flows |
| `LocalStorageRepository` | `src/infrastructure/repository.rs` | Saves/loads registry to browser storage on `wasm32` |
| `AppPersistence` facade | `src/infrastructure/repository.rs` | Platform-aware persistence boundary used by the interface layer |

### 🔲 Not Yet Implemented / Not Yet Verified

- End-to-end `dx serve` verification for WASM and desktop
- Native desktop/mobile persistence backend beyond browser storage

### ✅ Recently Implemented Behavioral Decisions

1. **Same-parent reparenting** — `reparent_card(id, current_parent)` is a **no-op** returning `Ok(())`. It preserves child ordering and bucket assignment.
2. **Read-path corruption** — `get_children` and `board_projection` **fail loudly**. `CardNotFound` or `BucketNotFound` are returned if the registry's internal pointers refer to missing cards or buckets.

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

## Resolved Behavioral Decisions

*These decisions are resolved and implemented in the current domain layer.*

### 1. Same-Parent Reparenting Contract

**Decision (binding):**
`reparent_card` MUST be a no-op when `new_parent_id == card.parent_id()`. It returns `Ok(())`
immediately, preserving child ordering and bucket assignment unchanged.

**Implemented behavior:** The registry now performs an early return after existence validation:

```rust
if card.parent_id() == Some(new_parent_id) {
    return Ok(());
}
```

**Regression test:** `test_reparent_to_same_parent_is_noop` — verify child ordering and bucket
assignment are unchanged after the call.

---

### 2. Read-Path Corruption Policy

**Decision (binding):** **Fail loudly.**

The silent fallback masks bugs introduced by incorrect registry mutations. The registry MUST
maintain the invariant that every ID in a `children_ids` list resolves to a stored card, and
every child's `bucket_id` is valid on its parent. If this invariant is violated, it is a bug in
the registry code, not a legitimate data state. The read path should surface it immediately.

**Implemented behavior:**

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

- **Primary:** Browser `localStorage` on `wasm32`. State is serialized to JSON via `JsonRepository`
  and stored via `LocalStorageRepository`.
- **Interface boundary:** The UI talks to `AppPersistence`, not directly to browser storage.
- **Non-web policy:** Native/non-browser targets are currently session-only. `AppPersistence`
  returns `DomainError::InvalidOperation("Persistence is not yet supported on this platform")`,
  and the app falls back to an empty in-memory registry while surfacing a visible warning banner.
- **Export/Import:** Download full state as JSON; re-upload to restore.
- **Future:** Add a native desktop/mobile persistence backend, then optional cloud sync.

Serde derives (`Serialize`, `Deserialize`) are **implemented** across all domain types.
WASM compatibility has been verified (including `getrandom` JS feature gates).

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

---

## Dioxus Interface UI/UX Specifications (Phase 4)

**The dashboard built with Dioxus MUST follow these exact design parameters.**

### Theme & Styling

- **Technology:** Tailwind CSS (via Tailwind CLI watch process).
  - *Why?* Tailwind provides utility classes that keep our UI tightly coupled, DRY (Don't Repeat Yourself), and easy to maintain as the application scales.
- **Palette:** Dark Mode by default. Light Mode accessible via a toggle button. The primary accent color is a "Warm Orange" (like a sunrise), designed to yield a dynamic, rich, and premium feel.
- **Implementation:** Use Tailwind's `class="dark"` toggling strategy on the global `<html>` or `<body>` tag. Define the custom warm orange colors in `tailwind.config.js`.

### Navigation

- "Drill down" mechanics only: Clicking on a card's target area immediately navigates to that card's board, fully replacing the screen.
- Only forward/backward navigation is supported. The top-left of the screen features a back arrow alongside the *Previous Card Name* to jump up exactly one parent level.

### Layout Details

- **Top Navigation Bar (Flexbox):**
  - **Left:** `[ < Previous Card Name ]` (Navigates up).
  - **Center:** `[ Current Card Name ]` (Hero title).
  - **Center-Right:** `[ + Create Bucket ]` button (Placed flexibly between the title and the right-side actions).
  - **Right:** `[ Import ]`, `[ Export ]`, and `[ ☼/☾ Toggle ]`.
- **Main Board Area:**
  - Designed for infinite horizontal and vertical scrolling (`overflow-x: auto`, `overflow-y: auto`).
  - Buckets are vertical columns. Cards auto-format into rows within those columns based on screen size constraints.

### Mutating State

- **Moving Cards:** No drag-and-drop for the MVP. Each `CardItem` exposes a move dropdown listing the parent card's available buckets. Cards *cannot* be moved to different parent cards in the MVP (only to different buckets within the same board).
- **Buckets:** Buckets cannot be reorganized after creation. The user must delete and recreate them to reorder.
  - *Rule:* Deleting a bucket executes the `RemoveBucket` command. The domain rejects deletion while any cards are still assigned to that bucket.
  - *Rule:* The "Unassigned" bucket is hidden unless it contains cards or if it is the *only* bucket on the board.
- **Modals:** Creating/Editing occurs in clean modal pop-ups.
  - Modals must blur the background (`backdrop-filter: blur(5px)` or similar) to prevent accidental clicks.
  - Clicking "X" closes the modal and auto-saves the state immediately. Deletion requires an explicit, separate "Delete" button.
