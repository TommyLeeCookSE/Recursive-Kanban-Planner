use crate::application::Command;
use crate::application::build_card_preview_view;
use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::Route;
use crate::interface::actions::{
    ReorderFeedbackContext, delete_card_with_feedback, dragged_root_card_id,
    execute_reorder_with_feedback, prime_drag_session, prime_drop_target,
};
use crate::interface::app::IsDragging;
use crate::interface::components::card_item::CardItem;
use crate::interface::components::modal::ModalType;
use crate::interface::components::visuals::{DropZoneKind, build_card_display, drop_zone_classes};
use dioxus::prelude::*;
use tracing::{Level, info};

/// The Home/Workspace view showing all top-level boards.
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
    let registry = use_context::<Signal<CardRegistry>>();
    let warning_message = use_context::<Signal<Option<String>>>();
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();
    let mut is_dragging = use_context::<Signal<IsDragging>>();
    let mut root_drop_index = use_signal(|| None::<usize>);

    let root_cards = {
        let reg = registry.read();
        let label_definitions = reg.label_definitions().to_vec();
        match reg
            .get_root_cards()
            .iter()
            .map(|card| {
                let preview_view = build_card_preview_view(card.id(), &reg)?;
                Ok(build_card_display(
                    card,
                    &label_definitions,
                    Some(&preview_view),
                ))
            })
            .collect::<Result<Vec<_>, DomainError>>()
        {
            Ok(cards) => cards,
            Err(error_value) => {
                return rsx! {
                    HomeLoadError {
                        message: error_value.to_string(),
                    }
                };
            }
        }
    };

    rsx! {
        div { class: "mx-auto min-h-full max-w-7xl px-6 py-12 lg:px-12",
            div { class: "mb-12 flex flex-col gap-6 lg:flex-row lg:flex-wrap lg:items-center lg:justify-between",
                div {
                    div { class: "app-kicker mb-3",
                        "Workspace"
                    }
                    h1 { class: "app-text-primary mb-2 text-5xl font-black tracking-tight",
                        "My Workspace"
                    }
                    p { class: "app-text-muted max-w-2xl text-base font-medium lg:text-lg",
                        "Organize your world with nested recursive boards."
                    }
                }
            }

            if root_cards.is_empty() {
                div { class: "app-empty-state flex flex-col items-center justify-center rounded-[2rem] py-32 text-center",
                    div { class: "app-kicker mb-6 text-sm",
                        "EMPTY WORKSPACE"
                    }
                    p { class: "app-text-muted mb-8 text-2xl font-bold",
                        "No boards found in your workspace."
                    }
                    button {
                        class: "app-button-secondary px-8 py-4",
                        onclick: move |_| active_modal.set(Some(ModalType::CreateCard {
                            parent_id: None,
                            bucket_id: None,
                        })),
                        "Create Your First Board"
                    }
                }
            } else {
                div { class: "grid grid-cols-1 gap-8 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4",
                    for (index, card) in root_cards.iter().cloned().enumerate() {
                        div { key: "{card.id}", class: "flex flex-col gap-3",
                            {render_root_drop_zone(index, root_drop_index, registry, warning_message, is_dragging)}
                            CardItem {
                                title: card.title,
                                subtitle: format!("{} nested items", card.nested_item_count),
                                due_date: card.due_date,
                                is_overdue: card.is_overdue,
                                labels: card.labels,
                                preview_sections: card.preview_sections,
                                draggable: true,
                                on_open: move |_| {
                                    navigator().push(Route::Board { card_id: card.id });
                                },
                                on_drag_start: move |event| {
                                    prime_drag_session(&event, "workspace-route", format!("root-card:{}", card.id), is_dragging);
                                    info!(card_id = %card.id, "Started dragging root card");
                                    record_diagnostic(
                                        Level::INFO,
                                        "workspace-route",
                                        format!("Started dragging root card {}", card.id),
                                    );
                                },
                                on_drag_end: move |_| {
                                    root_drop_index.set(None);
                                    is_dragging.set(IsDragging(false));
                                },
                                on_rename: move |_| {
                                    active_modal.set(Some(ModalType::EditCard { id: card.id }));
                                },
                                on_delete: move |_| {
                                    delete_card_with_feedback(
                                        card.id,
                                        registry,
                                        warning_message,
                                        "workspace-route",
                                        format!("delete workspace card {}", card.id),
                                    );
                                },
                            }
                        }
                    }
                    {render_root_drop_zone(root_cards.len(), root_drop_index, registry, warning_message, is_dragging)}
                }
            }
        }
    }
}

#[component]
fn HomeLoadError(message: String) -> Element {
    rsx! {
        div { class: "mx-auto max-w-3xl px-6 py-20 text-center lg:px-12",
            div { class: "app-empty-state rounded-[2rem] px-8 py-16",
                h2 { class: "mb-4 text-2xl font-bold text-red-500", "Workspace could not be loaded" }
                p { class: "app-text-muted", "The workspace data is unavailable or inconsistent. Check the logs for the full error." }
                p { class: "app-text-muted mt-3 text-sm", "{message}" }
            }
        }
    }
}

fn render_root_drop_zone(
    index: usize,
    mut root_drop_index: Signal<Option<usize>>,
    registry: Signal<CardRegistry>,
    warning_message: Signal<Option<String>>,
    is_dragging: Signal<IsDragging>,
) -> Element {
    let is_active = root_drop_index() == Some(index);
    let class_name = drop_zone_classes(DropZoneKind::Root, is_active, is_dragging().0);

    rsx! {
        div {
            class: "{class_name}",
            ondragover: move |event| {
                prime_drop_target(&event);
                root_drop_index.set(Some(index));
            },
            ondragleave: move |_| {
                if root_drop_index() == Some(index) {
                    root_drop_index.set(None);
                }
            },
            ondrop: move |event| {
                event.prevent_default();

                let Some(dragged_id) = dragged_root_card_id(&event, "workspace-route") else {
                    return;
                };

                root_drop_index.set(None);

                let current_order: Vec<CardId> = {
                    let reg = registry.read();
                    reg.get_root_cards().iter().map(|card| card.id()).collect()
                };

                let _ = execute_reorder_with_feedback(
                    &current_order,
                    dragged_id,
                    index,
                    ReorderFeedbackContext::new(
                        registry,
                        warning_message,
                        "workspace-route",
                        format!("reorder root cards with dragged card {dragged_id}"),
                    ),
                    |ordered_ids| Command::ReorderRootCards { ordered_ids },
                );
            },
            if is_dragging().0 {
                "Drop Here"
            }
        }
    }
}
