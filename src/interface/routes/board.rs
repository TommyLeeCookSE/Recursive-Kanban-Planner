//! The board route for individual cards.
//!
//! This module renders the board view for a specific card, including its
//! children, title, and due date.
//!
//! For an overview of the routing structure, see
//! `docs/rust-for-python-devs.md`.

use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::Route;
use crate::interface::app::{DraggedItemKind, IsDragging};
use crate::interface::components::board_view::{
    BoardDragSignals, BoardRenderContext, render_board_screen,
};
use crate::interface::components::modal::ModalType;
use crate::interface::routes::board_screen::load_board_screen_data;
use dioxus::prelude::*;
use tracing::{Level, error};

/// The board route component for a specific card.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     Board { card_id: my_card_id }
/// }
/// ```
#[component]
pub fn Board(card_id: CardId) -> Element {
    let registry = use_context::<Signal<CardRegistry>>();
    let active_modal = use_context::<Signal<Option<ModalType>>>();
    let warning_message = use_context::<Signal<Option<String>>>();
    let is_dragging = use_context::<Signal<IsDragging>>();
    let dragged_item_kind = use_context::<Signal<DraggedItemKind>>();
    let card_drop_index = use_signal(|| None::<usize>);

    let reg_guard = registry.read();
    let screen_data = match load_board_screen_data(card_id, &reg_guard) {
        Ok(screen_data) => screen_data,
        Err(error_value) => {
            error!(%card_id, error = %error_value, "Board route failed to load board state");
            record_diagnostic(
                Level::ERROR,
                "board-route",
                format!("Board load failed for {card_id}: {error_value}"),
            );
            return render_board_load_error();
        }
    };

    let render_context = BoardRenderContext {
        board_id: card_id,
        registry,
        active_modal,
        warning_message,
        drag: BoardDragSignals { card_drop_index },
        dragged_item_kind,
        is_dragging,
    };

    render_board_screen(
        screen_data.board_title,
        screen_data.back_route,
        screen_data.back_label,
        screen_data.board_due_date,
        screen_data.child_cards,
        render_context,
    )
}

fn render_board_load_error() -> Element {
    rsx! {
        div { class: "app-page-shell",
            div { class: "app-empty-state app-empty-state-panel",
                h2 { class: "app-error-title", "Board could not be loaded" }
                p { class: "app-error-message", "The board data is unavailable or inconsistent. Check the logs for the full error." }
                button {
                    class: "app-button-primary-compact app-empty-state-action",
                    onclick: |_| {
                        navigator().push(Route::Home {});
                    },
                    "Back to Workspace"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::error::DomainError;
    use crate::interface::routes::board_screen::load_board_screen_data;

    #[test]
    fn load_board_state_preserves_domain_errors() {
        let registry = CardRegistry::new();
        let missing_id = CardId::default();

        let result = load_board_screen_data(missing_id, &registry);

        assert!(matches!(result, Err(DomainError::CardNotFound(id)) if id == missing_id));
    }
}
