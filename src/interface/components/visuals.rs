//! Visual components, icons, and styling utilities for the user interface.
//!
//! This module contains SVG icon renderers, CSS class builders for dynamic
//! UI states (like drag-and-drop), and data structures for card display.
//!
//! For more on Rust's module system and documentation, see `docs/rust-for-python-devs.md`.

use crate::application::CardPreviewView;
use crate::domain::card::Card;
use crate::domain::id::CardId;
use crate::interface::app::DraggedItemKind;
use dioxus::prelude::*;

/// Categories of drop zones in the application.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropZoneKind {
    /// A drop zone representing the entire board or a major section.
    Board,
    /// A drop zone representing a specific card.
    Card,
}

/// A simplified view of a card's data for rendering in the UI.
///
/// # Examples
///
/// ```ignore
/// use crate::interface::components::visuals::CardDisplayData;
/// use crate::domain::id::CardId;
///
/// let data = CardDisplayData {
///     id: CardId::new(),
///     title: "Fix bug".to_string(),
///     due_date: Some("2023-12-31".to_string()),
///     is_overdue: false,
///     preview_items: vec!["Task 1".to_string()],
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardDisplayData {
    /// Unique identifier for the card.
    pub id: CardId,
    /// The card's title.
    pub title: String,
    /// Optional formatted due date string.
    pub due_date: Option<String>,
    /// Whether the card's due date has passed.
    pub is_overdue: bool,
    /// Titles of immediate child cards for a quick preview.
    pub preview_items: Vec<String>,
}

/// Transforms a domain `Card` into `CardDisplayData`.
///
/// # Examples
///
/// ```ignore
/// let display_data = build_card_display(&card, Some(&preview_view));
/// ```
pub fn build_card_display(card: &Card, preview_view: Option<&CardPreviewView>) -> CardDisplayData {
    let preview_items = preview_view
        .map(|view| {
            view.children
                .iter()
                .map(|child| child.title().to_string())
                .collect()
        })
        .unwrap_or_default();

    CardDisplayData {
        id: card.id(),
        title: card.title().to_string(),
        due_date: card.due_date().map(|due| due.to_string()),
        is_overdue: card.due_date().map(|due| due.is_overdue()).unwrap_or(false),
        preview_items,
    }
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
/// assert_eq!(toolbar_button_classes(), "app-toolbar-button");
/// ```
pub fn toolbar_button_classes() -> &'static str {
    "app-toolbar-button"
}

/// Standard CSS classes for an icon-only button in the main toolbar.
///
/// # Examples
///
/// ```
/// use kanban_planner::interface::components::visuals::toolbar_icon_button_classes;
/// assert_eq!(toolbar_icon_button_classes(), "app-toolbar-button app-toolbar-button--icon");
/// ```
pub fn toolbar_icon_button_classes() -> &'static str {
    "app-toolbar-button app-toolbar-button--icon"
}

/// Standard CSS classes for a label within a toolbar button.
///
/// # Examples
///
/// ```
/// use kanban_planner::interface::components::visuals::toolbar_button_label_classes;
/// assert_eq!(toolbar_button_label_classes(), "app-toolbar-label");
/// ```
pub fn toolbar_button_label_classes() -> &'static str {
    "app-toolbar-label"
}

/// Standard CSS classes for an icon within a toolbar button.
///
/// # Examples
///
/// ```
/// use kanban_planner::interface::components::visuals::toolbar_action_icon_classes;
/// assert_eq!(toolbar_action_icon_classes(), "app-toolbar-icon");
/// ```
pub fn toolbar_action_icon_classes() -> &'static str {
    "app-toolbar-icon"
}

/// Renders a "+" (plus) icon.
///
/// # Examples
///
/// ```ignore
/// rsx! { render_plus_icon() }
/// ```
pub fn render_plus_icon() -> Element {
    rsx! {
        svg {
            class: "app-icon-md",
            view_box: "0 0 20 20",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.9",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M10 4.5v11" }
            path { d: "M4.5 10h11" }
        }
    }
}

/// Renders a document/note icon.
///
/// # Examples
///
/// ```ignore
/// rsx! { render_note_icon() }
/// ```
pub fn render_note_icon() -> Element {
    rsx! {
        svg {
            class: "app-icon-md",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M7 3.75h7.75L19.5 8.5v11.75a1.5 1.5 0 0 1-1.5 1.5H7a1.5 1.5 0 0 1-1.5-1.5V5.25A1.5 1.5 0 0 1 7 3.75Z" }
            path { d: "M14.5 3.75V8.5H19.5" }
            path { d: "M8.5 12h7" }
            path { d: "M8.5 15.5h7" }
        }
    }
}

/// Renders a book icon.
///
/// # Examples
///
/// ```ignore
/// rsx! { render_book_icon() }
/// ```
pub fn render_book_icon() -> Element {
    rsx! {
        svg {
            class: "app-icon-md",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M6.5 4.75h9.75a2 2 0 0 1 2 2v12.5a1.5 1.5 0 0 0-1.5-1.5H6.5a2 2 0 0 1-2-2v-9a2 2 0 0 1 2-2Z" }
            path { d: "M6.5 4.75a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h10.25" }
            path { d: "M8.25 8h6.75" }
            path { d: "M8.25 11h6.75" }
        }
    }
}

/// Renders a label/tag icon.
///
/// # Examples
///
/// ```ignore
/// rsx! { render_label_icon() }
/// ```
pub fn render_label_icon() -> Element {
    rsx! {
        svg {
            class: "app-icon-md",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M11 4.5H6.75A2.25 2.25 0 0 0 4.5 6.75V11L12.75 19.25a1.5 1.5 0 0 0 2.12 0l4.38-4.38a1.5 1.5 0 0 0 0-2.12L11 4.5Z" }
            circle { cx: "8.25", cy: "8.25", r: "1.1" }
        }
    }
}

/// Renders an import/download icon.
///
/// # Examples
///
/// ```ignore
/// rsx! { render_import_icon() }
/// ```
pub fn render_import_icon() -> Element {
    rsx! {
        svg {
            class: "app-icon-md",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M12 4.75v10" }
            path { d: "m8.25 11.25 3.75 3.75 3.75-3.75" }
            path { d: "M6.5 16.5v1.75a2 2 0 0 0 2 2h7a2 2 0 0 0 2-2V16.5" }
        }
    }
}

/// Renders an export/upload icon.
///
/// # Examples
///
/// ```ignore
/// rsx! { render_export_icon() }
/// ```
pub fn render_export_icon() -> Element {
    rsx! {
        svg {
            class: "app-icon-md",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M12 19.25v-10" }
            path { d: "m15.75 12.75-3.75-3.75-3.75 3.75" }
            path { d: "M6.5 7.5V5.75a2 2 0 0 1 2-2h7a2 2 0 0 1 2 2V7.5" }
        }
    }
}

/// Renders a trash/delete icon.
///
/// # Examples
///
/// ```ignore
/// rsx! { render_trash_icon() }
/// ```
pub fn render_trash_icon() -> Element {
    rsx! {
        svg {
            class: "app-icon-md",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M4.75 7.5h14.5" }
            path { d: "M9 4.75h6" }
            path { d: "M8.25 7.5V5.75a1 1 0 0 1 1-1h5.5a1 1 0 0 1 1 1V7.5" }
            path { d: "M9.5 10.5v6.25" }
            path { d: "M14.5 10.5v6.25" }
            path { d: "M6.75 7.5l.75 10.25a2 2 0 0 0 2 1.85h4.95a2 2 0 0 0 2-1.85L17.2 7.5" }
        }
    }
}

/// Renders a sunrise icon.
///
/// # Examples
///
/// ```ignore
/// rsx! { render_sunrise_icon() }
/// ```
pub fn render_sunrise_icon() -> Element {
    rsx! {
        svg {
            class: "app-icon-md",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            circle { cx: "12", cy: "12", r: "5" }
            path { d: "M12 1v2" }
            path { d: "M12 21v2" }
            path { d: "M4.22 4.22l1.42 1.42" }
            path { d: "M18.36 18.36l1.42 1.42" }
            path { d: "M1 12h2" }
            path { d: "M21 12h2" }
            path { d: "M4.22 19.78l1.42-1.42" }
            path { d: "M18.36 5.64l1.42-1.42" }
        }
    }
}

/// Renders a moon/evening icon.
///
/// # Examples
///
/// ```ignore
/// rsx! { render_evening_icon() }
/// ```
pub fn render_evening_icon() -> Element {
    rsx! {
        svg {
            class: "app-icon-md",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M12 3a9 9 0 1 0 9 9 9.75 9.75 0 0 1-9-9Z" }
        }
    }
}

/// Renders a settings icon.
///
/// # Examples
///
/// ```ignore
/// rsx! { render_settings_icon() }
/// ```
pub fn render_settings_icon() -> Element {
    rsx! {
        svg {
            class: "app-icon-md",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M12 4.75 13.62 6.1l2.08-.33.9 1.9 1.97.74-.2 2.1 1.46 1.52-1.46 1.52.2 2.1-1.97.74-.9 1.9-2.08-.33L12 19.25l-1.62-1.35-2.08.33-.9-1.9-1.97-.74.2-2.1-1.46-1.52 1.46-1.52-.2-2.1 1.97-.74.9-1.9 2.08.33L12 4.75Z" }
            circle { cx: "12", cy: "12", r: "2.6" }
        }
    }
}

/// Renders a "back" arrow icon.
///
/// # Examples
///
/// ```ignore
/// rsx! { render_back_icon() }
/// ```
pub fn render_back_icon() -> Element {
    rsx! {
        svg {
            class: "app-icon-back",
            view_box: "0 0 20 20",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.9",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M11.5 4.5 6 10l5.5 5.5" }
            path { d: "M6.5 10h8" }
        }
    }
}
