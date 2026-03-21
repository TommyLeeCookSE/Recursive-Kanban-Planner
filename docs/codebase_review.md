# Codebase Review: Recursive Kanban Planner

I have completed a thorough review of the codebase for logic errors, structural integrity, and performance. Below are my findings and recommendations.

---

## 🔍 Critical & High Priority Issues

### 1. Drag-and-Drop "Ghost Failures" across Parents (RESOLVED)
- **Module**: `src/interface/components/board_view/drop_target.rs`
- **Error**: The `apply_card_drop` function previously exclusively used the `DropChildAtPosition` command.
- **Action Taken**: Updated `apply_card_drop` to check if the card's current parent matches the drop target parent. If they differ, it now executes a `ReparentCard` command before the reorder.

### 2. Inefficient State Persistence (Performance) (RESOLVED)
- **Module**: `src/interface/app.rs`
- **Error**: The registry was being cloned in the `App` render body.
- **Action Taken**: Moved the registry read and persistence logic into a `use_effect` hook to prevent redundant cloning on every render.

---

## 🧐 Domain Logic & Structural Observations

### 3. DRY Issues: Repetitive Note Page Lookup
- **Module**: `src/domain/card.rs`
- **Observation**: Methods like `rename_note_page`, `save_note_page_body`, and `delete_note_page` all manually iterate over `self.notes` to find a page by ID.
- **Recommendation**: Refactor this into a private helper method:
  ```rust
  fn find_note_page_mut(&mut self, id: NotePageId) -> Result<&mut NotePage, DomainError>
  ```

### 4. Boilerplate Heavy: Command Dispatch
- **Module**: `src/application/command.rs`
- **Observation**: The `Command` enum uses a `descriptor()` pattern that requires manually maintaining a match arm and a separate `apply_...` function for every single variant. This is highly repetitive and error-prone.
- **Recommendation**: Consider using a macro to generate the dispatch logic or simplify the `apply` method to directly match on `self`.

### 5. Inefficient Child Reordering Validation
- **Module**: `src/domain/card.rs`
- **Observation**: `Card::reorder_children` uses `self.children_ids.contains(id)` inside a loop. Since `children_ids` is a `Vec`, this results in $O(N^2)$ complexity for the validation.
- **Recommendation**: Convert the existing `children_ids` to a `HashSet` before the loop for $O(1)$ lookups, bringing the total validation cost down to $O(N)$.

### 6. Redundant Allocations: Board Projections
- **Module**: `src/domain/registry.rs`
- **Observation**: `CardRegistry::get_children` allocates a fresh `Vec` and performs multiple `HashMap` lookups every time a board is rendered.
- **Recommendation**: While acceptable for current board sizes, consider if the `BoardView` projection can be cached or if the UI can iterate over IDs directly to minimize allocations in tight render loops.

### 7. Flattened DOM: Board Grid (RESOLVED)
- **Module**: `src/interface/components/board_view/grid.rs`
- **Observation**: Redundant nested `.app-board-grid` divs.
- **Action Taken**: Flattened the structure to improve layout predictability.

---

## ✅ Overall Codebase Quality
The codebase follows **Clean Architecture** principles strictly, separating Domain, Application, Infrastructure, and Interface layers effectively. 

- **Strengths**: 
  - Strong use of ULIDs and newtypes for type safety.
  - Excellent documentation with runnable `# Examples`.
  - Consistent naming and project structure.
- **Areas for Growth**:
  - Consolidation of boilerplate in the Application layer.
  - Optimization of domain-level validations (from $O(N^2)$ to $O(N)$).
  - Centralizing repetitive entity-internal lookups.

## 🧪 Testing & Quality Assurance
- **Current State**: 46 passed unit/integration tests and 155 passed doc-tests.
- **Coverage**: Domain logic is well-tested. Application layer dispatch is partially tested.
- **Recommendation**: Add more integration tests for complex command sequences (e.g., move + reorder + delete) to ensure layer interactions remain stable.
