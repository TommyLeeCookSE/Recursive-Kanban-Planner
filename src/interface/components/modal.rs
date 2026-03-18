use dioxus::prelude::*;
use crate::domain::id::{CardId, BucketId};

#[derive(Clone, Debug, PartialEq)]
pub enum ModalType {
    CreateCard { parent_id: Option<CardId>, bucket_id: Option<BucketId> },
    RenameCard { id: CardId, current_title: String },
    CreateBucket { card_id: CardId },
}

#[component]
pub fn Modal(on_close: EventHandler<()>, title: String, children: Element) -> Element {
    rsx! {
        div { 
            class: "fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm animate-in fade-in duration-200",
            onclick: move |_| on_close.call(()),
            
            div { 
                class: "bg-white dark:bg-gray-800 w-full max-w-md rounded-2xl shadow-2xl overflow-hidden animate-in zoom-in-95 duration-200",
                onclick: |e| e.stop_propagation(),
                
                // Header
                div { class: "px-6 py-4 flex items-center justify-between border-b dark:border-gray-700",
                    h2 { class: "text-lg font-bold text-gray-900 dark:text-white", "{title}" }
                    button { 
                        class: "p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }
                
                // Content
                div { class: "p-6",
                    {children}
                }
            }
        }
    }
}

#[component]
pub fn CardModal(
    on_close: EventHandler<()>,
    parent_id: Option<CardId>,
    bucket_id: Option<BucketId>,
    registry: Signal<crate::domain::registry::CardRegistry>,
) -> Element {
    let mut input_title = use_signal(String::new);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: if parent_id.is_some() { "New Child Card" } else { "New Root Board" },
            div { class: "flex flex-col gap-4",
                input { 
                   class: "px-4 py-2 border rounded bg-white dark:bg-gray-700 dark:border-gray-600 focus:outline-none focus:ring-2 focus:ring-sunfire text-gray-900 dark:text-gray-100",
                   placeholder: "Enter title...",
                   value: "{input_title}",
                   oninput: move |e| input_title.set(e.value()),
                   autofocus: true,
                }
                div { class: "flex justify-end gap-2",
                    button { 
                       class: "px-4 py-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200",
                       onclick: move |_| on_close.call(()),
                       "Cancel"
                    }
                    button { 
                       class: "px-6 py-2 bg-sunfire hover:bg-sunfire-dark text-white font-bold rounded shadow transition-all disabled:opacity-50",
                       disabled: input_title().is_empty(),
                       onclick: move |_| {
                           use crate::application::{execute, Command};
                           let mut reg = registry.write();
                           let cmd = match parent_id {
                               Some(p_id) => Command::CreateChildCard { 
                                   title: input_title().to_string(),
                                   parent_id: p_id,
                                   bucket_id: bucket_id.unwrap_or_default()
                               },
                               None => Command::CreateRootCard { 
                                   title: input_title().to_string() 
                               },
                           };
                           let _ = execute(cmd, &mut reg);
                           on_close.call(());
                       },
                       "Create Item"
                    }
                }
            }
        }
    }
}

#[component]
pub fn BucketModal(
    on_close: EventHandler<()>,
    card_id: CardId,
    registry: Signal<crate::domain::registry::CardRegistry>,
) -> Element {
    let mut input_name = use_signal(String::new);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "New Column",
            div { class: "flex flex-col gap-4",
                input { 
                   class: "px-4 py-2 border rounded bg-white dark:bg-gray-700 dark:border-gray-600 focus:outline-none focus:ring-2 focus:ring-sunfire text-gray-900 dark:text-gray-100",
                   placeholder: "Column Name (e.g., Todo, Doing)",
                   value: "{input_name}",
                   oninput: move |e| input_name.set(e.value()),
                   autofocus: true,
                }
                div { class: "flex justify-end gap-2",
                    button { 
                       class: "px-4 py-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200",
                       onclick: move |_| on_close.call(()),
                       "Cancel"
                    }
                    button { 
                       class: "px-6 py-2 bg-sunfire hover:bg-sunfire-dark text-white font-bold rounded shadow transition-all disabled:opacity-50",
                       disabled: input_name().is_empty(),
                       onclick: move |_| {
                           use crate::application::{execute, Command};
                           let mut reg = registry.write();
                           let cmd = Command::AddBucket { 
                               card_id, 
                               name: input_name().to_string() 
                           };
                           let _ = execute(cmd, &mut reg);
                           on_close.call(());
                       },
                       "Add Column"
                    }
                }
            }
        }
    }
}
