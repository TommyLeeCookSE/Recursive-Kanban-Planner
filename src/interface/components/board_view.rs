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
mod header;
mod models;
mod slots;

use crate::interface::Route;
use crate::interface::components::board_view::grid::render_board_grid;
use crate::interface::components::modal::ModalType;
use crate::interface::components::visuals::{
    render_back_icon, render_note_icon, render_plus_icon, render_settings_icon, CardDisplayData,
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
    board_title: String,
    back_route: Option<Route>,
    back_label: String,
    board_due_date: String,
    child_models: Vec<CardDisplayData>,
    render_context: BoardRenderContext,
) -> Element {
    let board_id = render_context.board_id;
    let mut active_modal = render_context.active_modal;
    let back_label_for_button = back_label.clone();
    rsx! {
        div {
            class: "app-board-screen",
            style: format!("view-transition-name: card-{};", board_id),
            div { class: "app-board-screen-content",
                {render_board_grid(child_models, render_context)}
            }
            footer { class: "app-bottombar",
                if let Some(route) = back_route {
                    button {
                        class: "app-topbar-back app-topbar-back--board group w-full",
                        onclick: move |_| {
                            navigator().push(route.clone());
                        },
                        title: "Back to {back_label_for_button}",
                        span { class: "app-topbar-back-icon", {render_back_icon()} }
                        span { class: "app-topbar-back-label", "Back to: {back_label_for_button}" }
                    }
                } else {
                    button {
                        class: "app-topbar-back app-topbar-back--disabled app-topbar-back--board group w-full",
                        disabled: true,
                        span { class: "app-topbar-back-icon", {render_back_icon()} }
                        span { class: "app-topbar-back-label", "Back to: {back_label_for_button}" }
                    }
                }

                button {
                    class: "app-toolbar-button app-toolbar-button-accent w-full",
                    onclick: move |_| active_modal.set(Some(ModalType::CreateCard { parent_id: Some(board_id) })),
                    title: "Create Card",
                    "aria-label": "Create Card",
                    span { class: "app-toolbar-icon", {render_plus_icon()} }
                    span { class: "app-toolbar-label", "Create Card" }
                }
                button {
                    class: "app-toolbar-button w-full",
                    onclick: move |_| {
                        active_modal.set(Some(ModalType::CardNotes { card_id: board_id }));
                    },
                    title: "Open notes",
                    "aria-label": "Notes",
                    span { class: "app-toolbar-icon", {render_note_icon()} }
                    span { class: "app-toolbar-label", "Notes" }
                }
                button {
                    class: "app-toolbar-button w-full",
                    onclick: move |_| {
                        active_modal.set(Some(ModalType::EditCard { id: board_id }));
                    },
                    title: "Open settings",
                    "aria-label": "Settings",
                    span { class: "app-toolbar-icon", {render_settings_icon()} }
                    span { class: "app-toolbar-label", "Settings" }
                }
            }
        }
    }
}
