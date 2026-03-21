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
    let dragged_class = match dragged_item_kind {
        DraggedItemKind::None => {
            return match kind {
                DropZoneKind::Board => "app-drop-zone app-drop-zone--board app-drop-zone--hidden",
                DropZoneKind::Card => "app-drop-zone app-drop-zone--card app-drop-zone--hidden",
            };
        }
        DraggedItemKind::Card => match kind {
            DropZoneKind::Card => "app-drop-zone app-drop-zone--card",
            DropZoneKind::Board => "app-drop-zone app-drop-zone--board",
        },
    };

    match (kind, is_active) {
        (DropZoneKind::Card, true) => "app-drop-zone app-drop-zone--card app-drop-zone--active",
        (DropZoneKind::Card, false) => {
            if dragged_item_kind == DraggedItemKind::Card {
                "app-drop-zone app-drop-zone--card app-drop-zone--dragging"
            } else {
                dragged_class
            }
        }
        (DropZoneKind::Board, true) => "app-drop-zone app-drop-zone--board app-drop-zone--active",
        (DropZoneKind::Board, false) => {
            if dragged_item_kind == DraggedItemKind::Card {
                "app-drop-zone app-drop-zone--board app-drop-zone--dragging"
            } else {
                dragged_class
            }
        }
    }
}

/// Standard CSS classes for a primary action button on a surface.
///
/// # Examples
///
/// ```
/// use kanban_planner::interface::components::visuals::surface_action_button_classes;
/// assert_eq!(surface_action_button_classes(), "app-surface-action-button");
/// ```
pub fn surface_action_button_classes() -> &'static str {
    "app-surface-action-button"
}

/// Standard CSS classes for an icon-only button on a surface.
///
/// # Examples
///
/// ```
/// use kanban_planner::interface::components::visuals::surface_icon_button_classes;
/// assert_eq!(surface_icon_button_classes(), "app-surface-icon-button");
/// ```
pub fn surface_icon_button_classes() -> &'static str {
    "app-surface-icon-button"
}

/// Standard CSS classes for a destructive icon-only button on a surface.
///
/// # Examples
///
/// ```
/// use kanban_planner::interface::components::visuals::surface_destructive_icon_button_classes;
/// assert_eq!(surface_destructive_icon_button_classes(), "app-surface-icon-button app-surface-icon-button--danger");
/// ```
pub fn surface_destructive_icon_button_classes() -> &'static str {
    "app-surface-icon-button app-surface-icon-button--danger"
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

/// Standard CSS classes for a label within a toolbar button.
///
/// # Examples
///
/// ```
/// use kanban_planner::interface::components::visuals::toolbar_button_label_classes;
/// assert_eq!(toolbar_button_label_classes(), "app-bar-button-label");
/// ```
pub fn toolbar_button_label_classes() -> &'static str {
    "app-bar-button-label"
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
