use crate::application::Command;
use crate::interface::actions::{
    execute_command_with_feedback, prime_drop_target, run_with_view_transition,
};
use crate::interface::components::board_view::card::render_card_item;
use crate::interface::components::board_view::models::BoardRenderContext;
use crate::interface::components::visuals::{CardDisplayData, DropZoneKind, drop_zone_classes};
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
            {render_card_drop_zone(index, false, context.clone(), true)}
            div {
                class: "app-board-slot-card",
                style: format!("view-transition-name: card-{};", card_id),
                {render_card_item(card, context.clone())}
            }
            if is_last {
                {render_card_drop_zone(index + 1, false, context, true)}
            } else {
                div { class: "app-drop-slot-spacer" }
            }
        }
    }
}

pub(super) fn render_card_drop_zone(
    index: usize,
    emphasized: bool,
    context: BoardRenderContext,
    side_oriented: bool,
) -> Element {
    let board_id = context.board_id;
    let registry = context.registry;
    let warning_message = context.warning_message;
    let mut card_drop_index = context.drag.card_drop_index;
    let dragged_item_kind = context.dragged_item_kind;
    let is_dragging = context.is_dragging;
    let is_active = card_drop_index() == Some(index);
    let class_name = drop_zone_classes(
        if emphasized {
            DropZoneKind::Board
        } else {
            DropZoneKind::Card
        },
        is_active,
        dragged_item_kind(),
    );

    let class_name = if emphasized {
        format!("{class_name} min-h-[3.25rem] rounded-2xl")
    } else if side_oriented {
        format!("{class_name} app-drop-slot-lane")
    } else {
        format!("{class_name} h-4 rounded-full")
    };

    rsx! {
        div {
            class: "{class_name}",
            ondragover: move |event| {
                prime_drop_target(&event);
                card_drop_index.set(Some(index));
            },
            ondragleave: move |_| {
                if card_drop_index() == Some(index) {
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
                            target_index: index,
                        },
                        registry,
                        warning_message,
                        "board-route",
                        format!("drop card {card_id} at index {index} on board {board_id}"),
                    );
                });
            },
            if emphasized && is_dragging().0 {
                div { class: "app-drop-zone-content",
                    span { class: "app-kicker", "Drop Here" }
                }
            }
        }
    }
}
