use crate::interface::components::board_view::models::BoardRenderContext;
use dioxus::prelude::*;

pub(super) fn render_board_header(
    board_title: String,
    _board_due_date: String,
    _context: BoardRenderContext,
) -> Element {
    rsx! {
        div { class: "app-board-hero",
            div { class: "app-board-hero-card",
                h2 { class: "app-board-hero-title", "{board_title}" }
            }
        }
    }
}
