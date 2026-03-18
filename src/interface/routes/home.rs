use crate::domain::registry::CardRegistry;
use crate::interface::Route;
use crate::interface::components::modal::ModalType;
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    let registry = use_context::<Signal<CardRegistry>>();
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();
    let root_data: Vec<_> = registry
        .read()
        .get_root_cards()
        .iter()
        .map(|c| (c.id(), c.title().to_string(), c.children_ids().len()))
        .collect();

    rsx! {
        div { class: "p-8 max-w-6xl mx-auto",
            h1 { class: "text-4xl font-extrabold mb-8 text-sunfire tracking-tight", "My Workspace" }

            if root_data.is_empty() {
                div { class: "flex flex-col items-center justify-center p-20 bg-white dark:bg-gray-800 rounded-2xl shadow-xl border border-dashed border-gray-300 dark:border-gray-600",
                    p { class: "text-xl text-gray-500 mb-6", "Your workspace is empty." }
                    button {
                        class: "px-6 py-3 bg-sunfire hover:bg-sunfire-dark text-white font-bold rounded-lg shadow-lg transition-all transform hover:scale-105",
                        onclick: move |_| active_modal.set(Some(ModalType::CreateCard { parent_id: None, bucket_id: None })),
                        "Create Your First Card"
                    }
                }
            } else {
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                    for (id, title, count) in root_data {
                        div {
                            class: "p-6 bg-white dark:bg-gray-800 rounded-xl shadow-lg hover:shadow-2xl transition-all border border-transparent hover:border-sunfire cursor-pointer group",
                            onclick: move |_| {
                                navigator().push(Route::Board { card_id: id });
                            },
                            h3 { class: "text-xl font-bold text-gray-900 dark:text-white group-hover:text-sunfire transition-colors",
                                "{title}"
                            }
                            p { class: "mt-2 text-sm text-gray-500 dark:text-gray-400",
                                "{count} nested items"
                            }
                        }
                    }
                }
            }
        }
    }
}
