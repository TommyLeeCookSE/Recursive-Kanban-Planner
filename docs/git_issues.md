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

## Issue #72: UI Performance Optimization for Large Workspaces
**Status:** CLOSED
**Resolution:** Implemented memoization for board views, capped preview items, optimized serialization by removing redundant clones, and added stable keys to the grid.

## Issue #73: Board Navigation Does Not Update View
**Status:** CLOSED
**Resolution:** Fixed `use_memo` prop capture bug in the `Board` component by tracking the `card_id` prop with a local `use_signal`.

## Issue #74: Dynamic Button Labels and Top Bar Title Refinement
**Status:** CLOSED
**Resolution:** Added "Current Card: " to the top bar title. Re-introduced button labels to the bottom bar and web utilities, implementing a CSS flex-wrap trick (`max-height` with `overflow: hidden`) to dynamically hide labels when width scaling restricts available space, ensuring smooth regression to icon-only mode.

## Issue #75: UI Codebase DRY Refactoring and Modernization
**Status:** CLOSED
**Resolution:** Simplified `drop_zone_classes` logic, implemented `define_icon!` macro to reduce SVG boilerplate, and optimized `CardItem` to reduce redundant string allocations.

## Issue #76: Architectural Alignment for Platform Interop
**Status:** CLOSED
**Resolution:** Moved `confirm_destructive_action` from generic `shared_forms.rs` to `actions/interop.rs` and updated design guidelines to enforce this boundary.

## Issue #77: Codebase Refactoring and Performance Optimization
**Status:** CLOSED
**Resolution:** Unified `BoardView` and `CardPreviewView` into `CardView` projection. Hardened command execution with automatic registry validation. Implemented debounced persistence in the root `App` component to reduce I/O overhead. Introduced `use_execute_command` hook and `form_row!` macro to reduce UI boilerplate.

## Issue #78: P6 UI Polish
**Status:** CLOSED
**Resolution:** Tightened drag-and-drop affordances by adding a visual drag handle icon and hover states. Rebalanced card density by reducing border radii and adjusting padding and font sizes. Improved modal ergonomics by allowing the note textarea to flex and fill available space.

## Issue #79: Comprehensive Web Logging and Session Export
**Status:** CLOSED
**Resolution:** Implemented external `logging.toml` configuration for dynamic log levels. Added a custom `tracing` layer to automatically capture all session logs into an expanded in-memory buffer. Added a "Download Logs" utility to export captured diagnostics as a text file directly from the UI.


