use crate::application::CardPreviewView;
use crate::domain::card::Card;
use crate::domain::id::CardId;
use crate::domain::label::{LabelColor, LabelDefinition};
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropZoneKind {
    Root,
    Bucket,
    Card,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardPreviewDisplaySection {
    pub bucket_name: String,
    pub items: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardDisplayData {
    pub id: CardId,
    pub title: String,
    pub nested_item_count: usize,
    pub due_date: Option<String>,
    pub is_overdue: bool,
    pub labels: Vec<(String, LabelColor)>,
    pub preview_sections: Vec<CardPreviewDisplaySection>,
}

pub fn build_card_display(
    card: &Card,
    label_definitions: &[LabelDefinition],
    preview_view: Option<&CardPreviewView>,
) -> CardDisplayData {
    let preview_sections = preview_view
        .map(|view| {
            view.sections
                .iter()
                .map(|section| CardPreviewDisplaySection {
                    bucket_name: section.bucket.name().to_string(),
                    items: section
                        .cards
                        .iter()
                        .map(|child| child.title().to_string())
                        .collect(),
                })
                .collect()
        })
        .unwrap_or_default();

    CardDisplayData {
        id: card.id(),
        title: card.title().to_string(),
        nested_item_count: card.children_ids().len(),
        due_date: card.due_date().map(|due| due.to_string()),
        is_overdue: card.due_date().map(|due| due.is_overdue()).unwrap_or(false),
        labels: card
            .label_ids()
            .iter()
            .filter_map(|label_id| {
                label_definitions
                    .iter()
                    .find(|label| label.id() == *label_id)
                    .map(|label| (label.name().to_string(), label.color()))
            })
            .collect(),
        preview_sections,
    }
}

pub fn drop_zone_classes(kind: DropZoneKind, is_active: bool, is_dragging: bool) -> &'static str {
    match (kind, is_active, is_dragging) {
        (DropZoneKind::Bucket, true, _) => {
            "app-drop-zone app-drop-zone--bucket app-drop-zone--active"
        }
        (DropZoneKind::Bucket, false, true) => {
            "app-drop-zone app-drop-zone--bucket app-drop-zone--dragging"
        }
        (DropZoneKind::Bucket, false, false) => {
            "app-drop-zone app-drop-zone--bucket app-drop-zone--hidden"
        }
        (DropZoneKind::Card, true, _) => "app-drop-zone app-drop-zone--card app-drop-zone--active",
        (DropZoneKind::Card, false, true) => {
            "app-drop-zone app-drop-zone--card app-drop-zone--dragging"
        }
        (DropZoneKind::Card, false, false) => {
            "app-drop-zone app-drop-zone--card app-drop-zone--hidden"
        }
        (DropZoneKind::Root, true, _) => "app-drop-zone app-drop-zone--root app-drop-zone--active",
        (DropZoneKind::Root, false, true) => {
            "app-drop-zone app-drop-zone--root app-drop-zone--dragging"
        }
        (DropZoneKind::Root, false, false) => {
            "app-drop-zone app-drop-zone--root app-drop-zone--hidden"
        }
    }
}

pub fn render_label_chip(name: String, color: LabelColor) -> Element {
    let (background, text_color) = color.palette();
    rsx! {
        span {
            class: "rounded-full px-3 py-1 text-xs font-black uppercase tracking-widest",
            style: "background-color: {background}; color: {text_color};",
            "{name}"
        }
    }
}

pub fn surface_action_button_classes() -> &'static str {
    "app-button-secondary inline-flex h-8 min-w-[4.5rem] items-center justify-center rounded-full px-3 text-[11px] font-black uppercase tracking-widest"
}

pub fn surface_icon_button_classes() -> &'static str {
    "app-button-secondary inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full border-2 border-dashed p-0 text-sm leading-none"
}

pub fn surface_destructive_icon_button_classes() -> &'static str {
    "app-button-secondary inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full p-0 text-sm leading-none text-red-400 hover:text-red-500"
}

pub fn toolbar_button_classes() -> &'static str {
    "app-toolbar-button"
}

pub fn toolbar_icon_button_classes() -> &'static str {
    "app-toolbar-button app-toolbar-button--icon"
}

pub fn toolbar_button_label_classes() -> &'static str {
    "hidden sm:inline"
}

pub fn toolbar_button_mobile_icon_classes() -> &'static str {
    "text-lg leading-none sm:hidden"
}

pub fn render_plus_icon() -> Element {
    rsx! {
        svg {
            class: "h-5 w-5",
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
            class: "h-5 w-5",
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

pub fn render_label_icon() -> Element {
    rsx! {
        svg {
            class: "h-5 w-5",
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

pub fn render_settings_icon() -> Element {
    rsx! {
        svg {
            class: "h-5 w-5",
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
            class: "h-4.5 w-4.5",
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
