# Workspace Git Issues Tracking (OPEN)

## Issue #52: Split modal layer further to reduce maintainability risk
**Status:** OPEN
**Description:** The modal layer is still concentrated in a broad feature hub.Notes, labels, rules, card, and bucket flows keep converging on the same implementation surface.
**Scope:** Continue decomposing the modal system into smaller, feature-specific modules and shared helpers. Reduce the amount of code that changes when a single modal flow is updated.

## Issue #59: Repetitive Note Page Lookup (DRY)
**Status:** OPEN
**Description:** Methods in `src/domain/card.rs` (rename, save body, delete) manually iterate over `self.notes` to find a page by ID.
**Recommendation:** Implement a private helper method `find_note_page_mut(&mut self, id: NotePageId) -> Result<&mut NotePage, DomainError>`.

## Issue #60: Boilerplate Heavy Command Dispatch
**Status:** OPEN
**Description:** The `Command` enum requires maintaining repetitive `descriptor()` match arms and separate `apply_...` functions for every variant.
**Recommendation:** Refactor `src/application/command.rs` to use a macro or direct matching in `apply` to reduce boilerplate.

## Issue #61: O(N^2) Child Reordering Validation
**Status:** OPEN
**Description:** `Card::reorder_children` uses `Vec::contains` inside a loop, leading to suboptimal performance as board sizes grow.
**Recommendation:** Optimize the validation loop in `src/domain/card.rs` using a temporary `HashSet` to achieve O(N) complexity.

## Issue #82: Dragging a card copies all text from all cards
**Status:** OPEN
**Description:** When a card is dragged, it appears that all text from all cards in the view is being copied or included in the drag ghost/data. This causes performance issues and visual clutter during drag operations.

