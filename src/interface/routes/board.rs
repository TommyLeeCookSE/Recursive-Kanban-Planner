use crate::application::{BoardView, Command, build_board_view, execute};
use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::Route;
use crate::interface::components::card_item::{CardItem, MoveTarget};
use crate::interface::components::layout::TopBar;
use crate::interface::components::modal::ModalType;
use dioxus::prelude::*;
use tracing::{Level, error};

struct BoardScreenState<'a> {
    view: BoardView<'a>,
    back_route: Route,
    back_label: String,
    move_targets: Vec<MoveTarget>,
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
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();

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
    let nested_item_count = board_state.view.card.children_ids().len();

    rsx! {
        div { class: "flex h-full flex-col",
            TopBar {
                title: board_title.clone(),
                back_route: board_state.back_route.clone(),
                back_label: board_state.back_label.clone(),
                primary_label: "Create Bucket".to_string(),
                on_primary: move |_| active_modal.set(Some(ModalType::CreateBucket { card_id: board_id })),
                secondary_label: "Settings".to_string(),
                on_secondary: move |_| active_modal.set(Some(ModalType::RenameCard {
                    id: board_id,
                    current_title: board_title.clone(),
                })),
            }

            div { class: "app-panel flex items-center justify-between border-b px-6 py-5 lg:px-12",
                p { class: "app-kicker",
                    "Status: Active | {nested_item_count} nested items"
                }
            }

            div { class: "flex-grow overflow-x-auto px-6 py-10 lg:px-12",
                div { class: "flex gap-8 h-full items-start min-w-max",
                    for column in board_state.view.columns.iter() {
                        {render_column(
                            column,
                            board_id,
                            registry,
                            active_modal,
                            &board_state.move_targets,
                        )}
                    }
                }
            }
        }
    }
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

    let move_targets = view
        .card
        .buckets()
        .iter()
        .map(|bucket| MoveTarget {
            id: bucket.id(),
            name: bucket.name().to_string(),
        })
        .collect();

    Ok(BoardScreenState {
        view,
        back_route,
        back_label,
        move_targets,
    })
}

fn render_column(
    column: &crate::application::ColumnView<'_>,
    board_id: CardId,
    registry: Signal<CardRegistry>,
    active_modal: Signal<Option<ModalType>>,
    move_targets: &[MoveTarget],
) -> Element {
    let bucket_id = column.bucket.id();
    let bucket_name = column.bucket.name().to_string();
    let mut active_modal = active_modal;

    rsx! {
        div {
            key: "{bucket_id}",
            class: "app-column-surface group flex max-h-full w-80 flex-shrink-0 flex-col rounded-[2rem] p-5 transition-all hover:border-sunfire/30",
            div { class: "flex items-center justify-between mb-6 px-3",
                h2 { class: "app-kicker transition-colors group-hover:text-sunfire", "{bucket_name}" }
                button {
                    class: "app-button-secondary inline-flex h-8 w-8 items-center justify-center rounded-full border-2 border-dashed p-0 hover:rotate-90",
                    onclick: move |_| active_modal.set(Some(ModalType::CreateCard {
                        parent_id: Some(board_id),
                        bucket_id: Some(bucket_id),
                    })),
                    "+"
                }
            }
            div { class: "flex-grow overflow-y-auto space-y-4 pr-2",
                for card in column.cards.iter() {
                    {render_card_item(
                        card,
                        registry,
                        active_modal,
                        move_targets,
                    )}
                }
            }
        }
    }
}

fn render_card_item(
    card: &crate::domain::card::Card,
    registry: Signal<CardRegistry>,
    active_modal: Signal<Option<ModalType>>,
    move_targets: &[MoveTarget],
) -> Element {
    let card_id = card.id();
    let card_title = card.title().to_string();
    let nested_item_count = card.children_ids().len();
    let current_bucket_id = card.bucket_id();
    let move_targets = move_targets.to_vec();
    let mut active_modal = active_modal;
    let mut registry = registry;

    rsx! {
        CardItem {
            key: "{card_id}",
            title: card_title.clone(),
            subtitle: format!("{nested_item_count} nested items"),
            current_bucket_id,
            move_targets,
            on_open: move |_| {
                navigator().push(Route::Board { card_id });
            },
            on_rename: move |_| active_modal.set(Some(ModalType::RenameCard {
                id: card_id,
                current_title: card_title.clone(),
            })),
            on_move: move |next_bucket_id| {
                let mut reg = registry.write();
                let _ = execute(
                    Command::MoveCardToBucket {
                        card_id,
                        bucket_id: next_bucket_id,
                    },
                    &mut reg,
                );
            },
        }
    }
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
