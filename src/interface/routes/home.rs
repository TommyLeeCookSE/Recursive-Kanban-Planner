use crate::domain::registry::CardRegistry;
use crate::interface::Route;
use crate::interface::components::card_item::CardItem;
use crate::interface::components::modal::ModalType;
use dioxus::prelude::*;

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
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();

    let root_cards = {
        let reg = registry.read();
        reg.get_root_cards()
            .iter()
            .map(|card| {
                (
                    card.id(),
                    card.title().to_string(),
                    card.children_ids().len(),
                )
            })
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
                    for (id, title, count) in root_cards {
                        CardItem {
                            key: "{id}",
                            title,
                            subtitle: format!("{count} nested items"),
                            current_bucket_id: None,
                            on_open: move |_| {
                                navigator().push(Route::Board { card_id: id });
                            },
                        }
                    }
                }
            }
        }
    }
}
