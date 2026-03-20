use crate::application::Command;
use crate::interface::actions::{
    execute_command_with_feedback, prime_drop_target, run_with_view_transition,
};
use crate::interface::components::board_view::models::BoardRenderContext;
use crate::interface::components::visuals::{DropZoneKind, drop_zone_classes};
use dioxus::prelude::*;

pub(super) fn render_empty_drop_zone(context: BoardRenderContext) -> Element {
    let mut card_drop_index = context.drag.card_drop_index;
    let dragged_item_kind = context.dragged_item_kind;
    let is_dragging = context.is_dragging;
    let registry = context.registry;
    let warning_message = context.warning_message;
    let board_id = context.board_id;
    let is_active = card_drop_index() == Some(0);
    let class_name = drop_zone_classes(DropZoneKind::Board, is_active, dragged_item_kind());

    rsx! {
        div {
            class: "{class_name} app-drop-empty-zone app-drop-empty-zone-outline",
            ondragover: move |event| {
                prime_drop_target(&event);
                card_drop_index.set(Some(0));
            },
            ondragleave: move |_| {
                if card_drop_index() == Some(0) {
                    card_drop_index.set(None);
                }
            },
            ondrop: move |event| {
                event.prevent_default();

                let Some(card_id) = crate::interface::actions::dragged_card_id(&event, "board-route") else {
                    return;
                };

                card_drop_index.set(None);
                run_with_view_transition(move || {
                    let _ = execute_command_with_feedback(
                        Command::DropChildAtPosition {
                            parent_id: board_id,
                            card_id,
                            target_index: 0,
                        },
                        registry,
                        warning_message,
                        "board-route",
                        format!("drop card {card_id} into empty board {board_id}"),
                    );
                });
            },
            if is_dragging().0 {
                div { class: "app-drop-empty-zone-content",
                    span { class: "app-kicker", "Drop a card here" }
                }
            } else {
                div { class: "app-drop-empty-zone-content",
                    p { class: "app-empty-message", "No child cards yet. Create one to start this board." }
                }
            }
        }
    }
}
