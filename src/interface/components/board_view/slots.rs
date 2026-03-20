use crate::interface::components::board_view::card::render_card_item;
use crate::interface::components::board_view::drop_target::{
    DropZoneMode, activate_drop_zone, apply_card_drop, clear_drop_zone, styled_drop_zone_classes,
};
use crate::interface::components::board_view::models::BoardRenderContext;
use crate::interface::components::visuals::CardDisplayData;
use dioxus::prelude::*;

pub(super) fn render_card_slot(
    card: CardDisplayData,
    index: usize,
    is_last: bool,
    context: BoardRenderContext,
) -> Element {
    let card_id = card.id;
    rsx! {
        div {
            class: "app-board-slot",
            {render_card_drop_zone(index, DropZoneMode::CardLane, context.clone())}
            div {
                class: "app-board-slot-card",
                style: format!("view-transition-name: card-{};", card_id),
                {render_card_item(card, context.clone())}
            }
            if is_last {
                {render_card_drop_zone(index + 1, DropZoneMode::CardLane, context)}
            } else {
                div { class: "app-drop-slot-spacer" }
            }
        }
    }
}

pub(super) fn render_card_drop_zone(
    index: usize,
    mode: DropZoneMode,
    context: BoardRenderContext,
) -> Element {
    let card_drop_index = context.drag.card_drop_index;
    let dragged_item_kind = context.dragged_item_kind;
    let is_dragging = context.is_dragging;
    let is_active = card_drop_index() == Some(index);
    let show_drop_label = mode == DropZoneMode::BoardSurface;
    let class_name = styled_drop_zone_classes(mode, is_active, dragged_item_kind());

    rsx! {
        div {
            class: "{class_name}",
            ondragover: move |event| {
                activate_drop_zone(&event, card_drop_index, index);
            },
            ondragleave: move |_| {
                clear_drop_zone(card_drop_index, index);
            },
            ondrop: move |event| {
                apply_card_drop(
                    event,
                    index,
                    context.clone(),
                    format!(
                        "drop card at index {index} on board {}",
                        context.board_id
                    ),
                );
            },
            if show_drop_label && is_dragging().0 {
                div { class: "app-drop-zone-content",
                    span { class: "app-kicker", "Drop Here" }
                }
            }
        }
    }
}
