use dioxus::prelude::*;
use crate::interface::Route;
use crate::interface::components::modal::ModalType;
use crate::domain::id::CardId;

use crate::domain::registry::CardRegistry;
use crate::application::build_board_view;

#[component]
pub fn Board(card_id: CardId) -> Element {
    let registry = use_context::<Signal<CardRegistry>>();
    
    // Project the board into owned data to avoid lifetime issues with closures
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();

    let board_data = {
        let registry_read = registry.read();
        match build_board_view(card_id, &registry_read) {
            Ok(view) => {
                let columns: Vec<_> = view.columns.into_iter().map(|col| {
                    let cards: Vec<_> = col.cards.into_iter().map(|c| (c.id(), c.title().to_string())).collect();
                    (col.bucket.id(), col.bucket.name().to_string(), cards)
                }).collect();
                Some((view.card.title().to_string(), view.card.children_ids().len(), columns))
            },
            Err(_) => None,
        }
    };

    let Some((card_title, total_children, columns)) = board_data else {
        return rsx! { div { class: "p-8 text-red-500", "Board not found or inconsistent state." } };
    };

    rsx! {
        div { class: "h-full flex flex-col bg-gray-50 dark:bg-gray-900 transition-colors",
            // ... (Header)
            div { class: "px-8 py-6 flex items-center justify-between bg-white dark:bg-gray-800 border-b dark:border-gray-700 shadow-sm",
                div {
                    h1 { class: "text-2xl font-bold text-gray-900 dark:text-white", "{card_title}" }
                    p { class: "text-sm text-gray-500", "Managing {total_children} items" }
                }
                div { class: "flex gap-3",
                    button { class: "px-4 py-2 bg-sunfire hover:bg-sunfire-dark text-white rounded-lg shadow transition-all",
                        "+ Add Column"
                    }
                }
            }
            
            // Columns Container
            div { class: "flex-grow overflow-x-auto p-8",
                div { class: "flex gap-6 h-full items-start",
                    for (b_id, bucket_name, cards) in columns {
                        div { class: "flex-shrink-0 w-80 bg-gray-200/50 dark:bg-gray-800/50 p-4 rounded-xl flex flex-col max-h-full border border-transparent hover:border-gray-300 dark:hover:border-gray-700 transition-colors",
                            div { class: "flex items-center justify-between mb-4 px-2",
                                h2 { class: "text-sm font-bold uppercase tracking-widest text-gray-400", 
                                    "{bucket_name}"
                                }
                                button { 
                                    class: "text-gray-400 hover:text-sunfire transition-colors",
                                    onclick: move |_| active_modal.set(Some(ModalType::CreateCard { parent_id: Some(card_id), bucket_id: Some(b_id) })),
                                    "+"
                                }
                            }
                            
                            div { class: "flex-grow overflow-y-auto space-y-3",
                                for (id, title) in cards {
                                    div { class: "p-4 bg-white dark:bg-gray-700 rounded-lg shadow hover:shadow-md transition-shadow cursor-pointer border border-transparent hover:border-sunfire group",
                                        onclick: move |_| {
                                            navigator().push(Route::Board { card_id: id });
                                        },
                                        span { class: "font-medium text-gray-800 dark:text-gray-100 group-hover:text-sunfire transition-colors", "{title}" }
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
