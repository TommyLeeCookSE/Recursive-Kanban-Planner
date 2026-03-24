//! Fuzzy search and command palette modal.

use crate::domain::registry::CardRegistry;
use crate::interface::Route;
use crate::interface::components::modal::Modal;
use dioxus::prelude::*;

#[component]
pub fn SearchModal(on_close: EventHandler<()>, registry: Signal<CardRegistry>) -> Element {
    let mut query = use_signal(String::new);

    let results = use_memo(move || {
        let q = query().to_lowercase();
        if q.is_empty() {
            return vec![];
        }

        let reg = registry.read();
        let mut matches: Vec<(crate::domain::id::CardId, String, String)> = Vec::new();

        for card in reg.all_cards() {
            if card.title().to_lowercase().contains(&q) {
                let parent_name = match card.parent_id() {
                    Some(parent_id) => reg
                        .get_card(parent_id)
                        .ok()
                        .map(|p| p.title().to_string())
                        .unwrap_or_else(|| "Workspace".to_string()),
                    None => "Workspace".to_string(),
                };
                matches.push((card.id(), card.title().to_string(), parent_name));
            }

            for note in card.notes() {
                if note.title().to_lowercase().contains(&q)
                    || note.body().to_lowercase().contains(&q)
                {
                    matches.push((
                        card.id(),
                        format!("{} (Note: {})", card.title(), note.title()),
                        "Notes".to_string(),
                    ));
                }
            }
        }

        matches.truncate(10);
        matches
    });

    rsx! {
        Modal { title: "Search Workspace".to_string(), on_close,

            div { class: "app-search-palette flex flex-col gap-4",
                input {
                    class: "app-input-primary w-full text-lg p-4 bg-[var(--app-surface-soft)] border-[var(--app-border-strong)] rounded-lg focus:ring-2 focus:ring-[#f59e0b] outline-none",
                    placeholder: "Search cards and notes...",
                    autofocus: true,
                    value: "{query}",
                    oninput: move |e| query.set(e.value()),
                }

                div { class: "app-search-results flex flex-col gap-2 max-h-[60vh] overflow-y-auto",
                    if results().is_empty() && !query().is_empty() {
                        p { class: "text-center text-[var(--app-text-soft)] py-8",
                            "No results found."
                        }
                    }

                    for (id , label , kind) in results() {
                        button {
                            class: "app-search-result-item group flex items-center justify-between p-4 rounded-lg bg-[var(--app-surface)] hover:bg-[var(--app-surface-soft)] border border-[var(--app-border)] transition-all text-left",
                            onclick: move |_| {
                                navigator().push(Route::Board { card_id: id });
                                on_close.call(());
                            },

                            div { class: "flex flex-col",
                                span { class: "font-bold text-[var(--app-text-primary)] group-hover:text-[#f59e0b]",
                                    "{label}"
                                }
                                span { class: "text-xs text-[var(--app-text-soft)] uppercase tracking-widest",
                                    "{kind}"
                                }
                            }

                            span { class: "text-[var(--app-text-soft)] opacity-0 group-hover:opacity-100 transition-opacity",
                                "Jump to →"
                            }
                        }
                    }
                }

                div { class: "app-search-footer pt-4 border-t border-[var(--app-border)] flex justify-between text-xs text-[var(--app-text-soft)]",
                    span { "Tip: Type to filter results" }
                    span { "Esc to close" }
                }
            }
        }
    }
}
