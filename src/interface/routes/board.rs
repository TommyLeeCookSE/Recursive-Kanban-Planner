use crate::application::{BoardView, build_board_view, build_card_preview_view};
use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::Route;
use crate::interface::app::IsDragging;
use crate::interface::components::board_view::{
    BoardDragSignals, BoardRenderContext, ColumnRenderModel, render_board_screen,
};
use crate::interface::components::modal::ModalType;
use crate::interface::components::visuals::build_card_display;
use dioxus::prelude::*;
use tracing::{Level, error};

struct BoardScreenState<'a> {
    view: BoardView<'a>,
    back_route: Route,
    back_label: String,
}

/// Renders a single board and its buckets for the selected card.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     Board { card_id: board_id }
/// }
/// ```
#[component]
pub fn Board(card_id: CardId) -> Element {
    let registry = use_context::<Signal<CardRegistry>>();
    let active_modal = use_context::<Signal<Option<ModalType>>>();
    let warning_message = use_context::<Signal<Option<String>>>();
    let popup_queue = use_context::<Signal<Vec<crate::application::PopupNotification>>>();
    let is_dragging = use_context::<Signal<IsDragging>>();
    let bucket_drop_index = use_signal(|| None::<usize>);
    let card_drop_target = use_signal(|| None::<(crate::domain::id::BucketId, usize)>);

    let reg_guard = registry.read();
    let board_state = match load_board_state(card_id, &reg_guard) {
        Ok(state) => state,
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

    let board_id = card_id;
    let board_title = board_state.view.card.title().to_string();
    let label_definitions = reg_guard.label_definitions().to_vec();
    let board_display = build_card_display(board_state.view.card, &label_definitions, None);
    let board_due_date = board_display
        .due_date
        .as_deref()
        .unwrap_or("None")
        .to_string();
    let board_labels = board_display.labels.clone();
    let column_models = board_state
        .view
        .columns
        .iter()
        .map(|column| {
            let cards = column
                .cards
                .iter()
                .map(|card| {
                    let preview_view = build_card_preview_view(card.id(), &reg_guard)?;
                    Ok(build_card_display(
                        card,
                        &label_definitions,
                        Some(&preview_view),
                    ))
                })
                .collect::<Result<Vec<_>, DomainError>>()?;

            Ok(ColumnRenderModel {
                bucket_id: column.bucket.id(),
                bucket_name: column.bucket.name().to_string(),
                cards,
            })
        })
        .collect::<Result<Vec<_>, DomainError>>();
    let column_models = match column_models {
        Ok(models) => models,
        Err(error_value) => {
            error!(%board_id, error = %error_value, "Board route failed while building card previews");
            record_diagnostic(
                Level::ERROR,
                "board-route",
                format!("Board preview load failed for {board_id}: {error_value}"),
            );
            return render_board_load_error();
        }
    };
    let render_context = BoardRenderContext {
        board_id,
        registry,
        active_modal,
        warning_message,
        popup_queue,
        drag: BoardDragSignals {
            bucket_drop_index,
            card_drop_target,
        },
        is_dragging,
    };

    render_board_screen(
        board_title,
        board_state.back_route,
        board_state.back_label,
        board_due_date,
        board_labels,
        column_models,
        render_context,
    )
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
                Route::Board { card_id: parent_id },
                parent.title().to_string(),
            )
        }
        None => (Route::Home {}, "Workspace".to_string()),
    };

    Ok(BoardScreenState {
        view,
        back_route,
        back_label,
    })
}

fn render_board_load_error() -> Element {
    rsx! {
        div { class: "mx-auto max-w-3xl px-6 py-20 text-center lg:px-12",
            div { class: "app-empty-state rounded-[2rem] px-8 py-16",
                h2 { class: "mb-4 text-2xl font-bold text-red-500", "Board could not be loaded" }
                p { class: "app-text-muted", "The board data is unavailable or inconsistent. Check the logs for the full error." }
                button {
                    class: "app-button-primary mt-8 px-6 py-3",
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
