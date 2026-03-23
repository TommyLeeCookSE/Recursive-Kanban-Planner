//! The home route (root workspace).
//!
//! This module renders the workspace view, which is the root of the card
//! hierarchy.
//!
//! For an overview of the routing structure, see
//! `docs/rust-for-python-devs.md`.

use crate::infrastructure::logging::record_diagnostic;
use crate::interface::app::use_board_signals;
use crate::interface::components::board_view::{
    BoardDragSignals, BoardRenderContext, render_board_screen,
};
use crate::interface::routes::board_screen::load_workspace_screen_data;
use dioxus::prelude::*;
use tracing::{Level, error};

/// The home route component representing the root workspace.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     Home {}
/// }
/// ```
#[component]
pub fn Home() -> Element {
    let signals = use_board_signals();
    let card_drop_index = use_signal(|| None::<usize>);

    let screen_data_result = use_memo(move || {
        let reg = signals.registry.read();
        load_workspace_screen_data(&reg)
    });

    let screen_data_result = screen_data_result.read();
    let screen_data = match &*screen_data_result {
        Ok(data) => data.clone(),
        Err(error_value) => {
            error!(error = %error_value, "Workspace route failed to load workspace card");
            record_diagnostic(
                Level::ERROR,
                "workspace-route",
                format!("Workspace load failed: {error_value}"),
            );
            return render_workspace_load_error();
        }
    };

    let render_context = BoardRenderContext {
        board_id: screen_data.board_id,
        registry: signals.registry,
        active_modal: signals.active_modal,
        warning_message: signals.warning_message,
        drag: BoardDragSignals { card_drop_index },
        dragged_item_kind: signals.dragged_item_kind,
        is_dragging: signals.is_dragging,
    };

    render_board_screen(
        screen_data.board_title.clone(),
        None,
        screen_data.back_label,
        screen_data.board_due_date,
        screen_data.child_cards,
        render_context,
    )
}

fn render_workspace_load_error() -> Element {
    rsx! {
        div { class: "app-page-shell",
            div { class: "app-empty-state app-empty-state-panel",
                h2 { class: "app-error-title", "Workspace could not be loaded" }
                p { class: "app-error-message",
                    "The workspace data is unavailable or inconsistent. Check the logs for the full error."
                }
            }
        }
    }
}
