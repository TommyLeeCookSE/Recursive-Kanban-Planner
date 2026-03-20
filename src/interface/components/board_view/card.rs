use crate::infrastructure::logging::record_diagnostic;
use crate::interface::Route;
use crate::interface::actions::{delete_card_with_feedback, prime_drag_session};
use crate::interface::app::{DraggedItemKind, IsDragging};
use crate::interface::components::board_view::models::BoardRenderContext;
use crate::interface::components::card_item::CardItem;
use crate::interface::components::modal::ModalType;
use crate::interface::components::visuals::CardDisplayData;
use dioxus::prelude::*;
use tracing::{Level, info};

pub(super) fn render_card_item(card: CardDisplayData, context: BoardRenderContext) -> Element {
    let card_id = card.id;
    let mut active_modal = context.active_modal;
    let mut card_drop_index = context.drag.card_drop_index;
    let mut dragged_item_kind = context.dragged_item_kind;
    let mut is_dragging = context.is_dragging;

    rsx! {
        div {
            key: "{card_id}",
            class: "app-board-card-wrapper",
            CardItem {
                title: card.title,
                subtitle: None,
                draggable: true,
                on_open: move |_| {
                    navigator().push(Route::Board { card_id });
                },
                on_drag_start: move |event| {
                    prime_drag_session(
                        &event,
                        "board-route",
                        format!("card:{card_id}:parent:{}", context.board_id),
                        DraggedItemKind::Card,
                        dragged_item_kind,
                        is_dragging,
                    );
                    info!(card_id = %card_id, board_id = %context.board_id, "Started dragging card");
                    record_diagnostic(
                        Level::INFO,
                        "board-route",
                        format!("Started dragging card {card_id} on board {}", context.board_id),
                    );
                },
                on_drag_end: move |_| {
                    card_drop_index.set(None);
                    is_dragging.set(IsDragging(false));
                    dragged_item_kind.set(DraggedItemKind::None);
                },
                on_rename: move |_| active_modal.set(Some(ModalType::EditCard { id: card_id })),
                due_date: card.due_date,
                is_overdue: card.is_overdue,
                preview_items: card.preview_items,
                on_delete: move |_| {
                    delete_card_with_feedback(
                        card_id,
                        context.registry,
                        context.warning_message,
                        "board-route",
                        format!("delete board card {card_id}"),
                    );
                },
            }
        }
    }
}
