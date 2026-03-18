use crate::application::{Command, build_board_view, execute};
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::Route;
use crate::interface::components::card::{CardItem, MoveTarget};
use crate::interface::components::layout::TopBar;
use crate::interface::components::modal::ModalType;
use dioxus::prelude::*;
use tracing::{Level, error};

#[component]
pub fn Board(card_id: CardId) -> Element {
    let mut registry = use_context::<Signal<CardRegistry>>();
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();

    let board_data = {
        let registry_read = registry.read();
        match build_board_view(card_id, &registry_read) {
            Ok(view) => {
                let (back_route, back_label) = match view.card.parent_id() {
                    Some(parent_id) => match registry_read.get_card(parent_id) {
                        Ok(parent) => (
                            Route::Board { card_id: parent_id },
                            parent.title().to_string(),
                        ),
                        Err(_) => (Route::Home {}, "Workspace".to_string()),
                    },
                    None => (Route::Home {}, "Workspace".to_string()),
                };

                let move_targets: Vec<_> = view
                    .card
                    .buckets()
                    .iter()
                    .map(|bucket| MoveTarget {
                        id: bucket.id(),
                        name: bucket.name().to_string(),
                    })
                    .collect();

                let columns: Vec<_> = view
                    .columns
                    .into_iter()
                    .map(|col| {
                        let cards: Vec<_> = col
                            .cards
                            .into_iter()
                            .map(|card| {
                                (
                                    card.id(),
                                    card.title().to_string(),
                                    card.children_ids().len(),
                                    card.bucket_id(),
                                )
                            })
                            .collect();
                        (col.bucket.id(), col.bucket.name().to_string(), cards)
                    })
                    .collect();

                Some((
                    view.card.title().to_string(),
                    view.card.children_ids().len(),
                    back_route,
                    back_label,
                    move_targets,
                    columns,
                ))
            }
            Err(error_value) => {
                error!(%card_id, error = %error_value, "Board view could not be built");
                record_diagnostic(
                    Level::ERROR,
                    "board-route",
                    format!("Board {card_id} failed to load: {error_value}"),
                );
                None
            }
        }
    };

    let Some((card_title, total_children, back_route, back_label, move_targets, columns)) =
        board_data
    else {
        return rsx! { div { class: "p-8 text-red-500", "Board not found or inconsistent state." } };
    };

    let rename_title = card_title.clone();

    rsx! {
        div { class: "h-full flex flex-col bg-gray-50 dark:bg-gray-900 transition-colors",
            TopBar {
                title: card_title.clone(),
                back_route,
                back_label,
                primary_label: "Create Bucket".to_string(),
                on_primary: move |_| active_modal.set(Some(ModalType::CreateBucket { card_id })),
                secondary_label: "Board".to_string(),
                on_secondary: move |_| active_modal.set(Some(ModalType::RenameCard {
                    id: card_id,
                    current_title: rename_title.clone(),
                })),
            }

            div { class: "px-8 py-4 border-b border-gray-200 dark:border-gray-800 bg-white/70 dark:bg-gray-950/60",
                p { class: "text-sm text-gray-500 dark:text-gray-400",
                    "Managing {total_children} nested items"
                }
            }

            div { class: "flex-grow overflow-x-auto p-8",
                div { class: "flex gap-6 h-full items-start min-w-max",
                    for (bucket_id, bucket_name, cards) in columns {
                        div { class: "flex-shrink-0 w-80 bg-gray-200/50 dark:bg-gray-800/50 p-4 rounded-3xl flex flex-col max-h-full border border-transparent hover:border-gray-300 dark:hover:border-gray-700 transition-colors",
                            div { class: "flex items-center justify-between mb-4 px-2",
                                h2 { class: "text-sm font-bold uppercase tracking-widest text-gray-400",
                                    "{bucket_name}"
                                }
                                button {
                                    class: "inline-flex items-center justify-center h-9 w-9 rounded-full border border-dashed border-gray-300 dark:border-gray-600 text-gray-500 hover:border-sunfire hover:text-sunfire transition-colors",
                                    onclick: move |_| active_modal.set(Some(ModalType::CreateCard {
                                        parent_id: Some(card_id),
                                        bucket_id: Some(bucket_id),
                                    })),
                                    "+"
                                }
                            }

                            div { class: "flex-grow overflow-y-auto space-y-3 pr-1",
                                for (child_id, child_title, nested_count, current_bucket_id) in cards {
                                    CardItem {
                                        title: child_title.clone(),
                                        subtitle: format!("{nested_count} nested items"),
                                        current_bucket_id,
                                        move_targets: move_targets.clone(),
                                        on_open: move |_| {
                                            navigator().push(Route::Board { card_id: child_id });
                                        },
                                        on_rename: move |_| active_modal.set(Some(ModalType::RenameCard {
                                            id: child_id,
                                            current_title: child_title.clone(),
                                        })),
                                        on_move: move |next_bucket_id| {
                                            let mut reg = registry.write();
                                            if let Err(error_value) = execute(
                                                Command::MoveCardToBucket {
                                                    card_id: child_id,
                                                    bucket_id: next_bucket_id,
                                                },
                                                &mut reg,
                                            ) {
                                                error!(
                                                    card_id = %child_id,
                                                    bucket_id = %next_bucket_id,
                                                    error = %error_value,
                                                    "Move card action failed from board route"
                                                );
                                                record_diagnostic(
                                                    Level::ERROR,
                                                    "board-route",
                                                    format!(
                                                        "Move card action failed for {child_id} into {next_bucket_id}: {error_value}"
                                                    ),
                                                );
                                            }
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
