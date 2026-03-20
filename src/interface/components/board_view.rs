mod card;
mod empty_state;
mod grid;
mod header;
mod models;
mod slots;

use crate::interface::Route;
use crate::interface::components::board_view::grid::render_board_grid;
use crate::interface::components::board_view::header::render_board_header;
use crate::interface::components::visuals::CardDisplayData;
use dioxus::prelude::*;

pub(crate) use models::{BoardDragSignals, BoardRenderContext};

pub(crate) fn render_board_screen(
    board_title: String,
    back_route: Option<Route>,
    back_label: String,
    board_due_date: String,
    child_models: Vec<CardDisplayData>,
    render_context: BoardRenderContext,
) -> Element {
    let board_id = render_context.board_id;
    rsx! {
        div {
            class: "app-board-screen",
            style: format!("view-transition-name: card-{};", board_id),
            {render_board_header(
                board_title,
                back_route,
                back_label,
                board_due_date,
                render_context.clone(),
            )}
            div { class: "app-board-screen-content",
                {render_board_grid(child_models, render_context)}
            }
        }
    }
}
