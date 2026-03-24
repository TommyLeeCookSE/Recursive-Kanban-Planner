# Workspace Git Issues Tracking (OPEN)

## Issue #52: Split modal layer further to reduce maintainability risk
**Status:** CLOSED
**Resolution:** Consolidated modal command dispatching into `modal_dispatch_command` helper and applied `form_row!` macro across `CardModal`, `EditCardModal`, and `NotesModal`. Removed redundant error handling and registry locking boilerplate.

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
**Status:** CLOSED
**Resolution:** Implemented a global `.app-is-dragging` class on the root shell that applies `user-select: none !important` to all elements during drag operations. Added `select-none` to `CardItem` for local selection prevention.

## Issue #83: GitHub Deployment Pipeline is Broken
**Status:** CLOSED
**Resolution:** Updated GitHub Actions workflow to use `dx build` with explicit output directories and resolved WASM-specific compilation errors in `src/interface/app.rs`. Verified build path consistency between local and CI environments.

## Issue #84: Enhanced Map Navigation and Split-Header Layout
**Status:** CLOSED
**Resolution:** Implemented dynamic map zoom/panning constraints (clamped to layout edges), fixed minimap night theme contrast (white text on dark backgrounds), and introduced a 5vh `ContextBar` for card metadata.
