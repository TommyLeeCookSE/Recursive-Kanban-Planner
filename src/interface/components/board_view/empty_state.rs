use crate::interface::components::board_view::drop_target::{
    DropZoneMode, activate_drop_zone, apply_card_drop, clear_drop_zone, styled_drop_zone_classes,
};
use crate::interface::components::board_view::models::BoardRenderContext;
use dioxus::prelude::*;

pub(super) fn render_empty_drop_zone(context: BoardRenderContext) -> Element {
    let card_drop_index = context.drag.card_drop_index;
    let dragged_item_kind = context.dragged_item_kind;
    let is_dragging = context.is_dragging;
    let is_active = card_drop_index() == Some(0);
    let class_name = format!(
        "{} app-drop-empty-zone app-drop-empty-zone-outline",
        styled_drop_zone_classes(DropZoneMode::BoardSurface, is_active, dragged_item_kind())
    );

    rsx! {
        div {
            class: "{class_name}",
            ondragover: move |event| {
                activate_drop_zone(&event, card_drop_index, 0);
            },
            ondragleave: move |_| {
                clear_drop_zone(card_drop_index, 0);
            },
            ondrop: move |event| {
                let board_id = context.board_id;
                apply_card_drop(
                    event,
                    0,
                    context.clone(),
                    format!("drop card into empty board {board_id}"),
                );
            },
            if is_dragging().0 {
                div { class: "app-drop-empty-zone-content",
                    span { class: "app-kicker", "Drop a card here" }
                }
            } else {
                div { class: "app-drop-empty-zone-content",
                    p { class: "app-empty-message",
                        "No child cards yet. Create one to start this board."
                    }
                }
            }
        }
    }
}
