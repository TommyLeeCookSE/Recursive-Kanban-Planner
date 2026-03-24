# Workspace Git Issues Tracking (OPEN)

## Issue #52: Split modal layer further to reduce maintainability risk
**Status:** OPEN
**Description:** The modal layer is still concentrated in a broad feature hub.Notes, labels, rules, card, and bucket flows keep converging on the same implementation surface.
**Scope:** Continue decomposing the modal system into smaller, feature-specific modules and shared helpers. Reduce the amount of code that changes when a single modal flow is updated.

## Issue #59: Repetitive Note Page Lookup (DRY)
**Status:** CLOSED
**Resolution:** Implemented `find_note_page_mut` helper in `src/domain/card.rs` and refactored note lifecycle methods to use it.

## Issue #60: Boilerplate Heavy Command Dispatch
**Status:** CLOSED
**Resolution:** Consolidated `Command` variants and refactored `application/command.rs` to use match-based dispatch, removing redundant descriptors.

## Issue #61: O(N^2) Child Reordering Validation
**Status:** CLOSED
**Resolution:** Optimized reorder validation in `src/domain/card.rs` using a `HashSet` for O(N) complexity.

## Issue #82: Dragging a card copies all text from all cards
**Status:** OPEN
**Description:** When a card is dragged, it appears that all text from all cards in the view is being copied or included in the drag ghost/data. This causes performance issues and visual clutter during drag operations.

## Issue #83: GitHub Deployment Pipeline is Broken
**Status:** CLOSED
**Resolution:** Updated GitHub Actions workflow to use `dx build` with explicit output directories and resolved WASM-specific compilation errors in `src/interface/app.rs`. Verified build path consistency between local and CI environments.

## Issue #84: Enhanced Map Navigation and Split-Header Layout
**Status:** CLOSED
**Resolution:** Implemented dynamic map zoom/panning constraints, fixed minimap night theme contrast, and introduced a 5vh `ContextBar` for card metadata (title, due date, description) and split action navigation.


