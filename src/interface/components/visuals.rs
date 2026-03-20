use crate::application::CardPreviewView;
use crate::domain::card::Card;
use crate::domain::id::CardId;
use crate::interface::app::DraggedItemKind;
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropZoneKind {
    Board,
    Card,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardDisplayData {
    pub id: CardId,
    pub title: String,
    pub due_date: Option<String>,
    pub is_overdue: bool,
    pub preview_items: Vec<String>,
}

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

pub fn surface_action_button_classes() -> &'static str {
    "app-surface-action-button"
}

pub fn surface_icon_button_classes() -> &'static str {
    "app-surface-icon-button"
}

pub fn surface_destructive_icon_button_classes() -> &'static str {
    "app-surface-icon-button app-surface-icon-button--danger"
}

pub fn toolbar_button_classes() -> &'static str {
    "app-toolbar-button"
}

pub fn toolbar_icon_button_classes() -> &'static str {
    "app-toolbar-button app-toolbar-button--icon"
}

pub fn toolbar_button_label_classes() -> &'static str {
    "app-toolbar-label"
}

pub fn toolbar_action_icon_classes() -> &'static str {
    "app-toolbar-icon"
}

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
