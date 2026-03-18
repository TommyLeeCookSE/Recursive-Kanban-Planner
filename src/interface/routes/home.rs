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
        div { class: "p-12 max-w-7xl mx-auto min-h-full",
            div { class: "flex items-center justify-between mb-12",
                div {
                    h1 { class: "text-5xl font-black text-gray-900 dark:text-white tracking-tight mb-2",
                        "My Workspace"
                    }
                    p { class: "text-gray-500 dark:text-gray-400 font-medium",
                        "Organize your world with nested recursive boards."
                    }
                }
                button {
                    class: "px-8 py-4 bg-sunfire hover:bg-sunfire-dark text-white font-bold rounded-2xl shadow-lg hover:shadow-sunfire/20 transition-all transform hover:scale-105 active:scale-95 flex items-center gap-2",
                    onclick: move |_| active_modal.set(Some(ModalType::CreateCard {
                        parent_id: None,
                        bucket_id: None,
                    })),
                    span { class: "text-2xl", "+" }
                    "New Board"
                }
            }

            if root_cards.is_empty() {
                div { class: "flex flex-col items-center justify-center py-32 bg-white/50 dark:bg-gray-800/30 rounded-3xl border-2 border-dashed border-gray-200 dark:border-gray-700/50 backdrop-blur-sm",
                    div { class: "text-sm font-black uppercase tracking-[0.4em] text-gray-300 dark:text-gray-600 mb-6",
                        "EMPTY WORKSPACE"
                    }
                    p { class: "text-2xl font-bold text-gray-400 dark:text-gray-600 mb-8",
                        "No boards found in your workspace."
                    }
                    button {
                        class: "px-8 py-4 bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-bold rounded-2xl shadow-xl border border-gray-200 dark:border-gray-600 hover:border-sunfire transition-all",
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
