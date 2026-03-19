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
