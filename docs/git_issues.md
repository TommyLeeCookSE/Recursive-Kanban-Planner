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
