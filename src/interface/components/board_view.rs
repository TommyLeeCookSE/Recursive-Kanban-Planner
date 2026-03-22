//! The board view component for the Kanban Planner.
//!
//! This module coordinates the rendering of a specific board, including its
//! header, grid, and individual card items.
//!
//! For an overview of the board architecture, see
//! `docs/rust-for-python-devs.md`.

mod card;
mod drop_target;
mod empty_state;
mod grid;
mod models;
mod slots;

use crate::interface::Route;
use crate::interface::components::board_view::grid::render_board_grid;
use crate::interface::components::layout::BottomBar;
use crate::interface::components::map::Minimap;
use crate::interface::components::modal::ModalType;
use crate::interface::components::visuals::{
    CardDisplayData, render_note_icon, render_plus_icon, render_settings_icon,
};
use dioxus::prelude::*;

pub(crate) use models::{BoardDragSignals, BoardRenderContext};

/// Renders the complete board screen.
///
/// # Examples
///
/// ```ignore
/// render_board_screen(
///     "My Board".to_string(),
///     Some(Route::Home {}),
///     "Home".to_string(),
///     "Tomorrow".to_string(),
///     vec![card_display_data],
///     render_context,
/// )
/// ```
pub(crate) fn render_board_screen(
    _board_title: String,
    back_route: Option<Route>,
    back_label: String,
    _board_due_date: String,
    child_models: Vec<CardDisplayData>,
    render_context: BoardRenderContext,
) -> Element {
    let board_id = render_context.board_id;
    let mut active_modal = render_context.active_modal;
    let back_label_for_button = back_label.clone();
    rsx! {
        div {
            class: "app-board-screen flex flex-col h-full",
            style: format!("view-transition-name: card-{};", board_id),
            Minimap { current_card_id: board_id }
            div { class: "app-board-screen-content flex-1 overflow-y-auto",
                {render_board_grid(child_models, render_context)}
            }
            BottomBar { back_route: back_route.clone(), back_label: back_label_for_button,
                button {
                    class: "app-bar-button app-bar-button--accent",
                    onclick: move |_| active_modal.set(Some(ModalType::CreateCard { parent_id: Some(board_id) })),
                    title: "Create Card",
                    "aria-label": "Create Card",
                    span { class: "app-bar-button-icon", {render_plus_icon()} }
                    span { class: "app-bar-button-label", "Create Card" }
                }
                button {
                    class: "app-bar-button",
                    onclick: move |_| {
                        active_modal.set(Some(ModalType::CardNotes { card_id: board_id }));
                    },
                    title: "Open notes",
                    "aria-label": "Notes",
                    span { class: "app-bar-button-icon", {render_note_icon()} }
                    span { class: "app-bar-button-label", "Notes" }
                }
                button {
                    class: "app-bar-button",
                    onclick: move |_| {
                        active_modal.set(Some(ModalType::EditCard { id: board_id }));
                    },
                    title: "Open settings",
                    "aria-label": "Settings",
                    span { class: "app-bar-button-icon", {render_settings_icon()} }
                    span { class: "app-bar-button-label", "Settings" }
                }
            }
        }
    }
}
