//! The board route for individual cards.
//!
//! This module renders the board view for a specific card, including its
//! children, title, and due date.
//!
//! For an overview of the routing structure, see
//! `docs/rust-for-python-devs.md`.

use crate::domain::id::CardId;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::Route;
use crate::interface::app::use_board_signals;
use crate::interface::components::board_view::{
    BoardDragSignals, BoardRenderContext, render_board_screen,
};
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
    let signals = use_board_signals();
    let card_drop_index = use_signal(|| None::<usize>);

    // Back the non-reactive `card_id` prop with a signal so `use_memo` can track changes
    // when the router reuses this component instance for a new route.
    let mut tracked_card_id = use_signal(|| card_id);
    if *tracked_card_id.peek() != card_id {
        tracked_card_id.set(card_id);
    }

    // Memoize the board data so we don't re-calculate it (and all card previews)
    // during drag-and-drop operations or other transient state changes.
    let screen_data_result = use_memo(move || {
        let current_id = tracked_card_id();
        let reg = signals.registry.read();
        load_board_screen_data(current_id, &reg)
    });

    let screen_data_result = screen_data_result.read();
    let screen_data = match &*screen_data_result {
        Ok(data) => data.clone(),
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
        registry: signals.registry,
        active_modal: signals.active_modal,
        warning_message: signals.warning_message,
        drag: BoardDragSignals { card_drop_index },
        dragged_item_kind: signals.dragged_item_kind,
        is_dragging: signals.is_dragging,
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
                p { class: "app-error-message",
                    "The board data is unavailable or inconsistent. Check the logs for the full error."
                }
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
    use crate::domain::registry::CardRegistry;
    use crate::interface::routes::board_screen::load_board_screen_data;

    #[test]
    fn load_board_state_preserves_domain_errors() {
        let registry = CardRegistry::new();
        let missing_id = CardId::default();

        let result = load_board_screen_data(missing_id, &registry);

        assert!(matches!(result, Err(DomainError::CardNotFound(id)) if id == missing_id));
    }
}
