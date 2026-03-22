//! CSS class builders for dynamic UI states.
//!
//! This module provides helper functions to compute Tailwind CSS classes
//! based on application state, ensuring consistent styling across components.
//!
//! For more on Rust's string handling, see `docs/rust-for-python-devs.md`.

use crate::interface::app::DraggedItemKind;

/// Categories of drop zones in the application.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropZoneKind {
    /// A drop zone representing the entire board or a major section.
    Board,
    /// A drop zone representing a specific card.
    Card,
}

/// Computes the CSS classes for a drop zone based on its state.
///
/// # Examples
///
/// ```ignore
/// let classes = drop_zone_classes(DropZoneKind::Card, true, DraggedItemKind::Card);
/// ```
pub fn drop_zone_classes(
    kind: DropZoneKind,
    is_active: bool,
    dragged_item_kind: DraggedItemKind,
) -> &'static str {
    match kind {
        DropZoneKind::Board => match (dragged_item_kind, is_active) {
            (DraggedItemKind::None, _) => {
                "app-drop-zone app-drop-zone--board app-drop-zone--hidden"
            }
            (_, true) => "app-drop-zone app-drop-zone--board app-drop-zone--active",
            _ => "app-drop-zone app-drop-zone--board app-drop-zone--dragging",
        },
        DropZoneKind::Card => match (dragged_item_kind, is_active) {
            (DraggedItemKind::None, _) => "app-drop-zone app-drop-zone--card app-drop-zone--hidden",
            (_, true) => "app-drop-zone app-drop-zone--card app-drop-zone--active",
            _ => "app-drop-zone app-drop-zone--card app-drop-zone--dragging",
        },
    }
}

/// Standard CSS classes for a button in the main toolbar.
///
/// # Examples
///
/// ```
/// use kanban_planner::interface::components::visuals::toolbar_button_classes;
/// assert_eq!(toolbar_button_classes(), "app-bar-button");
/// ```
pub fn toolbar_button_classes() -> &'static str {
    "app-bar-button"
}

/// Standard CSS classes for an icon-only button in the main toolbar.
///
/// # Examples
///
/// ```
/// use kanban_planner::interface::components::visuals::toolbar_icon_button_classes;
/// assert_eq!(toolbar_icon_button_classes(), "app-bar-button");
/// ```
pub fn toolbar_icon_button_classes() -> &'static str {
    "app-bar-button"
}

/// Standard CSS classes for an icon within a toolbar button.
///
/// # Examples
///
/// ```
/// use kanban_planner::interface::components::visuals::toolbar_action_icon_classes;
/// assert_eq!(toolbar_action_icon_classes(), "app-bar-button-icon");
/// ```
pub fn toolbar_action_icon_classes() -> &'static str {
    "app-bar-button-icon"
}
