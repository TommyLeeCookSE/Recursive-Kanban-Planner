# Workspace Git Issues Tracking

## Issue #12: Theme Label Inversion
**Status:** CLOSED
**Resolution:** Swapped labels in `src/interface/components/layout.rs`.

## Issue #13: Sunrise (Light Mode) Aesthetic Inconsistency
**Status:** CLOSED
**Resolution:** Updated `theme-light .app-atmosphere` in `src/interface/tailwind.css`.

## Issue #14: Non-Standardized Drop Zone Styling
**Status:** CLOSED
**Resolution:** Introduced `--app-drop-*` CSS variables and standardized transitions.

## Issue #15: Context Ambiguity and Drop Here Visibility
**Status:** CLOSED
**Resolution:** Introduced `IsDark` and `IsDragging` newtypes.

## Issue #16: Sunrise Theme Re-Design
**Status:** CLOSED
**Resolution:** Redesigned `.theme-light` atmosphere with warm gradients.

## Issue #55: Redundant Module File (visuals.rs)
**Status:** CLOSED
**Resolution:** Deleted redundant `src/interface/components/visuals.rs`.

## Issue #62: Drag-and-Drop "Ghost Failures" across Parents
**Status:** CLOSED
**Resolution:** Implemented automatic `ReparentCard` command in `apply_card_drop`.

## Issue #56: Inefficient State Persistence (Performance)
**Status:** CLOSED
**Resolution:** Moved persistence to `use_effect` hook.

## Issue #57: Root Workspace Vulnerable to Reparenting
**Status:** CLOSED
**Resolution:** Protected root from reparenting in `src/domain/registry/mutations.rs`.

## Issue #58: Redundant DIV Nesting in Board Grid
**Status:** CLOSED
**Resolution:** Flattened DOM structure in `src/interface/components/board_view/grid.rs`.

## Issue #59: Repetitive Note Page Lookup (DRY)
**Status:** CLOSED
**Resolution:** Refactored `Card` to use private helper `find_note_page_mut`.

## Issue #60: Boilerplate Heavy Command Dispatch
**Status:** CLOSED
**Resolution:** Simplified `Command` enum methods `name` and `apply` to match directly.

## Issue #61: O(N^2) Child Reordering Validation
**Status:** CLOSED
**Resolution:** Optimized `Card::reorder_children` with a temporary `HashSet`.

## Issue #63: Unified Adaptive Top/Bottom Bars
**Status:** CLOSED
**Resolution:** Introduced unified `.app-bar` system with responsive sizing and 3-slot layout.

## Issue #64: Adaptive Icon Resizing
**Status:** CLOSED
**Resolution:** Implemented flex-compression and intrinsic SVG resizing for icons.

## Issue #65: App Bar Refinement and Card Action Sizing
**Status:** CLOSED
**Resolution:** Made title always visible, implemented equal icon distribution in BottomBar, and reduced card action sizes.

## Issue #71: Card Title Character Limit
**Status:** CLOSED
**Resolution:** Implemented 80-character limit for card and note titles in both domain validation and UI inputs.

