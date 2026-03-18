use crate::domain::label::LabelColor;
use crate::interface::components::visuals::render_label_chip;
use dioxus::prelude::*;

/// A premium, reusable card component for both Workspace and Board views.
///
/// Features:
/// - Clickable area for navigation (opening the card).
/// - Optional "Rename" action.
/// - Optional "Delete" action.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     CardItem {
///         title: "Design API".to_string(),
///         subtitle: "2 nested items".to_string(),
///         on_open: move |_| {},
///     }
/// }
/// ```
#[component]
pub fn CardItem(
    /// Main display text.
    title: String,
    /// Second line of text for counts or metadata.
    subtitle: String,
    #[props(default)] due_date: Option<String>,
    #[props(default)] is_overdue: bool,
    #[props(default)] labels: Vec<(String, LabelColor)>,
    /// Triggered when the main body of the card is clicked.
    on_open: EventHandler<()>,
    /// Optional rename event. If None, the rename button is hidden.
    #[props(default)]
    on_rename: Option<EventHandler<()>>,
    /// Optional delete event. If None, the delete button is hidden.
    #[props(default)]
    on_delete: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        article {
            class: "app-card-surface group flex flex-col rounded-[1.75rem] transition-all hover:border-sunfire/50 hover:-translate-y-0.5",

            button {
                class: "flex-grow w-full rounded-t-[1.75rem] p-6 text-left outline-none transition-colors focus:ring-2 focus:ring-sunfire/30",
                onclick: move |_| on_open.call(()),
                h3 { class: "app-text-primary h-12 overflow-hidden text-lg font-semibold transition-colors group-hover:text-sunfire line-clamp-2",
                    "{title}"
                }
                p { class: "app-text-soft mt-3 text-xs font-medium uppercase tracking-widest",
                    "{subtitle}"
                }
                if let Some(due_date) = due_date {
                    p {
                        class: if is_overdue { "mt-3 text-sm font-semibold text-red-500" } else { "app-text-muted mt-3 text-sm font-semibold" },
                        "Due {due_date}"
                    }
                }
                if !labels.is_empty() {
                    div { class: "mt-4 flex flex-wrap gap-2",
                        for (name, color) in labels {
                            {render_label_chip(name, color)}
                        }
                    }
                }
            }

            if on_rename.is_some() || on_delete.is_some() {
                div { class: "flex items-center justify-end rounded-b-[1.75rem] border-t px-5 py-4", style: "border-color: var(--app-border); background-color: color-mix(in srgb, var(--app-surface-soft) 74%, transparent);",
                    div { class: "flex items-center gap-2",
                        if let Some(delete_handler) = on_delete {
                            button {
                                class: "app-button-secondary rounded-full px-3 py-1.5 text-[11px] font-black uppercase tracking-widest text-red-400 hover:text-red-500",
                                onclick: move |_| delete_handler.call(()),
                                "Delete"
                            }
                        }
                        if let Some(rename_handler) = on_rename {
                            button {
                                class: "app-button-secondary rounded-full px-3 py-1.5 text-[11px] font-black uppercase tracking-widest",
                                onclick: move |_| rename_handler.call(()),
                                "Edit"
                            }
                        }
                    }
                }
            }
        }
    }
}
