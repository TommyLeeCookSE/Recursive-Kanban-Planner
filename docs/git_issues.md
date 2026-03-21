# Workspace Git Issues Tracking

## Issue #12: Theme Label Inversion
**Description:** The theme toggle labels were inverted (showing 'Sunrise' in dark mode and 'Evening' in light mode).
**Status:** CLOSED
**Resolution:** Swapped labels in `src/interface/components/layout.rs`.

## Issue #13: Sunrise (Light Mode) Aesthetic Inconsistency
**Description:** Sunrise mode was using outdated/evening-style background and lacked the 'dense star' effect requested for dark mode.
**Status:** CLOSED
**Resolution:** Updated `theme-light .app-atmosphere` in `src/interface/tailwind.css` with a high-density particle/star field appropriate for the light theme.

## Issue #14: Non-Standardized Drop Zone Styling
**Description:** Drop zones were hardcoded to 'sunfire' colors, which didn't always contrast well in light mode and lacked a unified settings approach.
**Status:** CLOSED
**Resolution:** 
- Introduced `--app-drop-*` CSS variables in both `.theme-dark` and `.theme-light`.
- Updated the shared drop-zone render helpers in the interface layer to use these variables.
- Standardized transition effects to `transition-all`.

## Issue #15: Context Ambiguity and Drop Here Visibility
**Description:** Boolean context signals (`is_dark` and `is_dragging`) were ambiguous, causing the "Drop Here" text to show in dark mode even when not dragging.
**Status:** CLOSED
**Resolution:** Introduced `IsDark` and `IsDragging` newtypes in `src/interface/app.rs` and updated all components to use them.

## Issue #16: Sunrise Theme Re-Design
**Description:** Sunrise mode lacked the vibrant orange sunrise feel and had too many star-like particles.
**Status:** CLOSED
**Resolution:** Redesigned `.theme-light` atmosphere and backdrop in `src/interface/tailwind.css` with warm, glowing orange and yellow gradients.

## Issue #55: Redundant Module File (visuals.rs)
**Description:** Found both `src/interface/components/visuals.rs` and `src/interface/components/visuals/mod.rs` during codebase review, which blocked static analysis and compilation.
**Status:** CLOSED
**Resolution:** Deleted the redundant `src/interface/components/visuals.rs` file.

## Issue #62: Drag-and-Drop "Ghost Failures" across Parents
**Description:** Moving a card between different parent boards silently failed because `DropChildAtPosition` only supported reordering within the same parent.
**Status:** CLOSED
**Resolution:** Updated `apply_card_drop` in `src/interface/components/board_view/drop_target.rs` to detect parent changes and execute `Command::ReparentCard` before reordering.

## Issue #56: Inefficient State Persistence (Performance)
**Description:** The root `App` component was cloning the entire registry and executing persistence logic on every render, causing UI stuttering.
**Status:** CLOSED
**Resolution:** Moved registry cloning and persistence into a `use_effect` hook in `src/interface/app.rs`.

## Issue #57: Root Workspace Vulnerable to Reparenting
**Description:** The domain layer did not explicitly prevent the root workspace card from being reparented, which could lead to a "dangling" root or corrupt tree state.
**Status:** CLOSED
**Resolution:** Added an explicit check in `reparent_card` in `src/domain/registry/mutations.rs` to reject operations on the workspace root.

## Issue #58: Redundant DIV Nesting in Board Grid
**Description:** The board grid was using nested `.app-board-grid` divs, complicating the DOM and flexbox layout.
**Status:** CLOSED
**Resolution:** Flattened the structure in `src/interface/components/board_view/grid.rs`.

## Issue #59: Repetitive Note Page Lookup (DRY)
**Description:** Methods in `src/domain/card.rs` (rename, save body, delete) manually iterate over `self.notes` to find a page by ID.
**Status:** OPEN
**Recommendation:** Implement a private helper method `find_note_page_mut(&mut self, id: NotePageId) -> Result<&mut NotePage, DomainError>`.

## Issue #60: Boilerplate Heavy Command Dispatch
**Description:** The `Command` enum requires maintaining repetitive `descriptor()` match arms and separate `apply_...` functions for every variant.
**Status:** OPEN
**Recommendation:** Refactor `src/application/command.rs` to use a macro or direct matching in `apply` to reduce boilerplate.

## Issue #61: O(N^2) Child Reordering Validation
**Description:** `Card::reorder_children` uses `Vec::contains` inside a loop, leading to suboptimal performance as board sizes grow.
**Status:** OPEN
**Recommendation:** Optimize the validation loop in `src/domain/card.rs` using a temporary `HashSet` to achieve O(N) complexity.
