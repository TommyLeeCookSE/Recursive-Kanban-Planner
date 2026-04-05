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
    BarButton, CardDisplayData, render_note_icon, render_plus_icon, render_settings_icon,
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
            BottomBar {
                back_route: back_route.clone(),
                back_label: back_label_for_button,
                BarButton {
                    label: "Create Card".to_string(),
                    title: Some("Create Card".to_string()),
                    aria_label: Some("Create Card".to_string()),
                    class_name: Some("app-bar-button--accent".to_string()),
                    icon: Some(render_plus_icon()),
                    on_click: move |_| {
                        active_modal.set(Some(ModalType::CreateCard {
                            parent_id: Some(board_id),
                        }))
                    },
                }
                BarButton {
                    label: "Notes".to_string(),
                    title: Some("Open notes".to_string()),
                    aria_label: Some("Notes".to_string()),
                    icon: Some(render_note_icon()),
                    on_click: move |_| {
                        active_modal.set(Some(ModalType::CardNotes { card_id: board_id }));
                    },
                }
                BarButton {
                    label: "Settings".to_string(),
                    title: Some("Open settings".to_string()),
                    aria_label: Some("Settings".to_string()),
                    icon: Some(render_settings_icon()),
                    on_click: move |_| active_modal.set(Some(ModalType::EditCard { id: board_id })),
                }
            }
        }
    }
}
