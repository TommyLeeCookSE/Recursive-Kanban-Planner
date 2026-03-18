use crate::application::{Command, execute};
use crate::domain::id::{BucketId, CardId};
use crate::domain::registry::CardRegistry;
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ModalType {
    CreateCard {
        parent_id: Option<CardId>,
        bucket_id: Option<BucketId>,
    },
    RenameCard {
        id: CardId,
        current_title: String,
    },
    CreateBucket {
        card_id: CardId,
    },
}

#[component]
pub fn Modal(on_close: EventHandler<()>, title: String, children: Element) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm animate-in fade-in duration-200",
            onclick: move |_| on_close.call(()),

            div {
                class: "w-full max-w-md overflow-hidden rounded-3xl border border-white/20 bg-white/95 shadow-2xl backdrop-blur-md dark:border-gray-700/70 dark:bg-gray-900/95 animate-in zoom-in-95 duration-200",
                onclick: |e| e.stop_propagation(),

                div { class: "px-6 py-4 flex items-center justify-between border-b border-gray-200 dark:border-gray-800",
                    h2 { class: "text-lg font-bold text-gray-900 dark:text-white", "{title}" }
                    button {
                        class: "p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors",
                        onclick: move |_| on_close.call(()),
                        "X"
                    }
                }

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
    registry: Signal<CardRegistry>,
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
                        disabled: input_title().trim().is_empty(),
                        onclick: move |_| {
                            let mut reg = registry.write();
                            let trimmed_title = input_title().trim().to_string();
                            let cmd = match parent_id {
                                Some(p_id) => Command::CreateChildCard {
                                    title: trimmed_title,
                                    parent_id: p_id,
                                    bucket_id: bucket_id.unwrap_or_default(),
                                },
                                None => Command::CreateRootCard {
                                    title: trimmed_title,
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
pub fn RenameCardModal(
    on_close: EventHandler<()>,
    id: CardId,
    current_title: String,
    registry: Signal<CardRegistry>,
) -> Element {
    let mut input_title = use_signal(move || current_title.clone());

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "Rename Item",
            div { class: "flex flex-col gap-4",
                input {
                    class: "px-4 py-2 border rounded bg-white dark:bg-gray-700 dark:border-gray-600 focus:outline-none focus:ring-2 focus:ring-sunfire text-gray-900 dark:text-gray-100",
                    placeholder: "Enter a new title...",
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
                        disabled: input_title().trim().is_empty(),
                        onclick: move |_| {
                            let mut reg = registry.write();
                            let _ = execute(
                                Command::RenameCard {
                                    id,
                                    title: input_title().trim().to_string(),
                                },
                                &mut reg,
                            );
                            on_close.call(());
                        },
                        "Save Changes"
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
    registry: Signal<CardRegistry>,
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
                        disabled: input_name().trim().is_empty(),
                        onclick: move |_| {
                            let mut reg = registry.write();
                            let _ = execute(
                                Command::AddBucket {
                                    card_id,
                                    name: input_name().trim().to_string(),
                                },
                                &mut reg,
                            );
                            on_close.call(());
                        },
                        "Add Column"
                    }
                }
            }
        }
    }
}
