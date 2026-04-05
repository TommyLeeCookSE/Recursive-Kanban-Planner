# Workspace Git Issues Tracking (OPEN)

## Issue #104: Add Card Color Controls
**Status:** OPEN
**Priority:** MEDIUM
**GitHub:** [#84](https://github.com/TommyLeeCookSE/Recursive-Kanban-Planner/issues/84)
**Observation:** Cards do not currently provide a direct UI control for changing their display color.
**Recommendation:** Add a button or action in the card controls that opens a color picker or cycles through preset card colors.

## Issue #105: Add Closed-Card Title Strikethrough
**Status:** OPEN
**Priority:** LOW
**GitHub:** [#85](https://github.com/TommyLeeCookSE/Recursive-Kanban-Planner/issues/85)
**Observation:** There is no quick visual affordance for marking a card as closed/completed without changing its title state manually.
**Recommendation:** Add a button that toggles a strikethrough style on the card title to visually resemble a closed card.

## Issue #106: Support Drag-to-Reassign Parent Card
**Status:** OPEN
**Priority:** HIGH
**GitHub:** [#86](https://github.com/TommyLeeCookSE/Recursive-Kanban-Planner/issues/86)
**Observation:** Cards can be reordered, but there is no direct drag-and-drop workflow for moving a card under a different parent card.
**Recommendation:** Implement drag-and-drop reparenting so a dragged card can be reassigned to another card as its new parent.

## Issue #107: Replace Drop Zones with Live Reflow During Drag
**Status:** OPEN
**Priority:** HIGH
**GitHub:** [#87](https://github.com/TommyLeeCookSE/Recursive-Kanban-Planner/issues/87)
**Observation:** Showing explicit drop zones makes nested card placement harder to preview while dragging.
**Recommendation:** Use live layout reflow during drag so nearby cards slide out of the way, letting the user see the exact destination and resulting layout in real time.
