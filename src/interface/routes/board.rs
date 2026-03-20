use crate::application::{build_board_view, build_card_preview_view};
use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::Route;
use crate::interface::app::{DraggedItemKind, IsDragging};
use crate::interface::components::board_view::{
    BoardDragSignals, BoardRenderContext, render_board_screen,
};
use crate::interface::components::modal::ModalType;
use crate::interface::components::visuals::build_card_display;
use dioxus::prelude::*;
use tracing::{Level, error};

#[derive(Clone, PartialEq, Eq)]
struct BoardScreenData {
    board_title: String,
    board_due_date: String,
    back_route: Option<Route>,
    back_label: String,
    child_cards: Vec<crate::interface::components::visuals::CardDisplayData>,
}

#[component]
pub fn Board(card_id: CardId) -> Element {
    let registry = use_context::<Signal<CardRegistry>>();
    let active_modal = use_context::<Signal<Option<ModalType>>>();
    let warning_message = use_context::<Signal<Option<String>>>();
    let is_dragging = use_context::<Signal<IsDragging>>();
    let dragged_item_kind = use_context::<Signal<DraggedItemKind>>();
    let card_drop_index = use_signal(|| None::<usize>);

    let reg_guard = registry.read();
    let Some(screen_data) = build_board_screen_data(card_id, &reg_guard) else {
        return render_board_load_error();
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

fn build_board_screen_data(card_id: CardId, registry: &CardRegistry) -> Option<BoardScreenData> {
    let board_state = match load_board_state(card_id, registry) {
        Ok(state) => state,
        Err(error_value) => {
            error!(%card_id, error = %error_value, "Board route failed to load board state");
            record_diagnostic(
                Level::ERROR,
                "board-route",
                format!("Board load failed for {card_id}: {error_value}"),
            );
            return None;
        }
    };

    let board_display = build_card_display(board_state.view.card, None);
    let board_due_date = board_display
        .due_date
        .as_deref()
        .unwrap_or("None")
        .to_string();

    let child_cards = match board_state
        .view
        .children
        .iter()
        .map(|card| {
            let preview_view = build_card_preview_view(card.id(), registry)?;
            Ok(build_card_display(card, Some(&preview_view)))
        })
        .collect::<Result<Vec<_>, DomainError>>()
    {
        Ok(cards) => cards,
        Err(error_value) => {
            error!(
                %card_id,
                error = %error_value,
                "Board route failed while building card previews"
            );
            record_diagnostic(
                Level::ERROR,
                "board-route",
                format!("Board preview load failed for {card_id}: {error_value}"),
            );
            return None;
        }
    };

    Some(BoardScreenData {
        board_title: board_state.view.card.title().to_string(),
        board_due_date,
        back_route: board_state.back_route,
        back_label: board_state.back_label,
        child_cards,
    })
}

struct BoardScreenState<'a> {
    view: crate::application::BoardView<'a>,
    back_route: Option<Route>,
    back_label: String,
}

fn load_board_state<'a>(
    card_id: CardId,
    registry: &'a CardRegistry,
) -> Result<BoardScreenState<'a>, DomainError> {
    let view = build_board_view(card_id, registry)?;

    let (back_route, back_label) = match view.card.parent_id() {
        Some(parent_id) => {
            let parent = registry.get_card(parent_id)?;
            (
                Some(Route::Board { card_id: parent_id }),
                parent.title().to_string(),
            )
        }
        None => (None, view.card.title().to_string()),
    };

    Ok(BoardScreenState {
        view,
        back_route,
        back_label,
    })
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

    #[test]
    fn load_board_state_preserves_domain_errors() {
        let registry = CardRegistry::new();
        let missing_id = CardId::default();

        let result = load_board_state(missing_id, &registry);

        assert!(matches!(result, Err(DomainError::CardNotFound(id)) if id == missing_id));
    }
}
