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
pub struct CardDisplayData {
    pub id: CardId,
    pub title: String,
    pub nested_item_count: usize,
    pub due_date: Option<String>,
    pub is_overdue: bool,
    pub labels: Vec<(String, LabelColor)>,
}

pub fn build_card_display(card: &Card, label_definitions: &[LabelDefinition]) -> CardDisplayData {
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
