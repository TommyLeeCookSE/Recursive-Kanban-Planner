use crate::interface::components::board_view::empty_state::render_empty_drop_zone;
use crate::interface::components::board_view::models::BoardRenderContext;
use crate::interface::components::board_view::slots::render_card_slot;
use crate::interface::components::visuals::CardDisplayData;
use dioxus::prelude::*;

pub(super) fn render_board_grid(
    child_models: Vec<CardDisplayData>,
    context: BoardRenderContext,
) -> Element {
    rsx! {
        div { class: "app-board-grid",
            if child_models.is_empty() {
                {render_empty_drop_zone(context.clone())}
            } else {
                for (index, card) in child_models.iter().cloned().enumerate() {
                    {render_card_slot(
                        card,
                        index,
                        index + 1 == child_models.len(),
                        context.clone(),
                    )}
                }
            }
        }
    }
}
