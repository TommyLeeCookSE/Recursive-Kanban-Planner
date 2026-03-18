use crate::application::Command;
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::domain::registry::DeleteStrategy;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::Route;
use crate::interface::actions::{
    dragged_root_card_id, execute_command_with_feedback, prime_drag_session, prime_drop_target,
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
        reg.get_root_cards()
            .iter()
            .map(|card| build_card_display(card, &label_definitions))
            .collect::<Vec<_>>()
    };

    rsx! {
        div { class: "mx-auto min-h-full max-w-7xl px-6 py-12 lg:px-12",
            div { class: "mb-12 flex flex-col gap-6 lg:flex-row lg:items-center lg:justify-between",
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
                button {
                    class: "app-button-primary px-8 py-4",
                    onclick: move |_| active_modal.set(Some(ModalType::CreateCard {
                        parent_id: None,
                        bucket_id: None,
                    })),
                    span { class: "text-2xl", "+" }
                    "New Board"
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
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-8",
                    for (index, card) in root_cards.iter().cloned().enumerate() {
                        div { key: "{card.id}", class: "flex flex-col gap-3",
                            {render_root_drop_zone(index, root_drop_index, registry, warning_message, is_dragging)}
                            div {
                                draggable: true,
                                ondragstart: move |event| {
                                    prime_drag_session(&event, "workspace-route", format!("root-card:{}", card.id), is_dragging);
                                    info!(card_id = %card.id, "Started dragging root card");
                                    record_diagnostic(
                                        Level::INFO,
                                        "workspace-route",
                                        format!("Started dragging root card {}", card.id),
                                    );
                                },
                                ondragend: move |_| {
                                    root_drop_index.set(None);
                                    is_dragging.set(IsDragging(false));
                                },
                                CardItem {
                                    title: card.title,
                                    subtitle: format!("{} nested items", card.nested_item_count),
                                    due_date: card.due_date,
                                    is_overdue: card.is_overdue,
                                    labels: card.labels,
                                    on_open: move |_| {
                                        navigator().push(Route::Board { card_id: card.id });
                                    },
                                    on_rename: move |_| {
                                        active_modal.set(Some(ModalType::EditCard { id: card.id }));
                                    },
                                    on_delete: move |_| {
                                        delete_card_with_feedback(card.id, registry, warning_message);
                                    },
                                }
                            }
                        }
                    }
                    {render_root_drop_zone(root_cards.len(), root_drop_index, registry, warning_message, is_dragging)}
                }
            }
        }
    }
}

fn delete_card_with_feedback(
    id: CardId,
    registry: Signal<CardRegistry>,
    warning_message: Signal<Option<String>>,
) {
    if execute_command_with_feedback(
        Command::DeleteCard {
            id,
            strategy: DeleteStrategy::CascadeDelete,
        },
        registry,
        warning_message,
        "workspace-route",
        format!("delete workspace card {id}"),
    )
    .is_err()
    {}
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

                let reordered = reorder_ids(&current_order, dragged_id, index);
                if reordered != current_order {
                    info!(card_id = %dragged_id, drop_index = index, "Attempting root card reorder");
                    record_diagnostic(
                        Level::INFO,
                        "workspace-route",
                        format!("Attempting root reorder for {dragged_id} at index {index}"),
                    );
                    if execute_command_with_feedback(
                        Command::ReorderRootCards { ordered_ids: reordered },
                        registry,
                        warning_message,
                        "workspace-route",
                        format!("reorder root cards with dragged card {dragged_id}"),
                    )
                    .is_err()
                    {
                    }
                }
            },
            if is_dragging().0 {
                "Drop Here"
            }
        }
    }
}

fn reorder_ids<T>(ordered_ids: &[T], dragged_id: T, target_index: usize) -> Vec<T>
where
    T: Copy + Eq,
{
    let mut reordered: Vec<T> = ordered_ids
        .iter()
        .copied()
        .filter(|id| *id != dragged_id)
        .collect();
    let insertion_index = target_index.min(reordered.len());
    reordered.insert(insertion_index, dragged_id);
    reordered
}
