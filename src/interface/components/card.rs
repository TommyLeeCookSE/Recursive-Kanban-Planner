use crate::domain::id::BucketId;
use dioxus::prelude::*;
use std::str::FromStr;

#[derive(Clone, PartialEq)]
pub struct MoveTarget {
    pub id: BucketId,
    pub name: String,
}

#[component]
pub fn CardItem(
    title: String,
    subtitle: String,
    current_bucket_id: Option<BucketId>,
    move_targets: Vec<MoveTarget>,
    on_open: EventHandler<()>,
    on_rename: EventHandler<()>,
    on_move: EventHandler<BucketId>,
) -> Element {
    let current_bucket_value = current_bucket_id
        .map(|id| id.to_string())
        .unwrap_or_default();

    rsx! {
        article { class: "rounded-2xl border border-transparent bg-white dark:bg-gray-700 shadow-sm transition-all hover:border-sunfire/50 hover:shadow-lg",
            button {
                class: "w-full text-left p-4 cursor-pointer",
                onclick: move |_| on_open.call(()),
                h3 { class: "font-semibold text-gray-900 dark:text-gray-100",
                    "{title}"
                }
                p { class: "mt-2 text-sm text-gray-500 dark:text-gray-400",
                    "{subtitle}"
                }
            }

            div { class: "border-t border-gray-200/80 dark:border-gray-600/80 px-4 py-3 flex flex-wrap items-center gap-3",
                if !move_targets.is_empty() {
                    label { class: "flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.2em] text-gray-400 dark:text-gray-500",
                        span { "Move" }
                        select {
                            class: "rounded-full border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm font-medium text-gray-700 dark:text-gray-100 focus:border-sunfire focus:outline-none",
                            value: "{current_bucket_value}",
                            onchange: move |e| {
                                if let Ok(bucket_id) = BucketId::from_str(&e.value()) {
                                    on_move.call(bucket_id);
                                }
                            },
                            for target in move_targets {
                                option { value: "{target.id}", "{target.name}" }
                            }
                        }
                    }
                }

                button {
                    class: "ml-auto inline-flex items-center gap-2 rounded-full border border-gray-300 dark:border-gray-600 px-3 py-2 text-sm font-medium text-gray-600 dark:text-gray-200 hover:border-sunfire hover:text-sunfire transition-colors",
                    onclick: move |_| on_rename.call(()),
                    "Rename"
                }
            }
        }
    }
}
