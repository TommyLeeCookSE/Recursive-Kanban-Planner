use crate::interface::components::shared_forms::confirm_destructive_action;
use crate::interface::components::visuals::{
    render_trash_icon, surface_action_button_classes, surface_destructive_icon_button_classes,
};
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
    #[props(default)] preview_items: Vec<String>,
    #[props(default = false)] draggable: bool,
    /// Triggered when the main body of the card is clicked.
    on_open: EventHandler<()>,
    #[props(default)] on_drag_start: Option<EventHandler<DragEvent>>,
    #[props(default)] on_drag_end: Option<EventHandler<DragEvent>>,
    /// Optional rename event. If None, the rename button is hidden.
    #[props(default)]
    on_rename: Option<EventHandler<()>>,
    /// Optional delete event. If None, the delete button is hidden.
    #[props(default)]
    on_delete: Option<EventHandler<()>>,
) -> Element {
    let card_title_for_open = title.clone();
    let card_title_for_delete = title.clone();

    rsx! {
        article {
            class: "app-card-surface group flex flex-col rounded-[1.75rem] transition-all hover:border-sunfire/50 hover:-translate-y-0.5",

            button {
                class: if draggable {
                    "flex-grow w-full cursor-grab rounded-t-[1.75rem] p-6 text-left outline-none transition-colors focus:ring-2 focus:ring-sunfire/30 active:cursor-grabbing"
                } else {
                    "flex-grow w-full rounded-t-[1.75rem] p-6 text-left outline-none transition-colors focus:ring-2 focus:ring-sunfire/30"
                },
                draggable: draggable,
                ondragstart: move |event| {
                    if let Some(handler) = &on_drag_start {
                        handler.call(event);
                    }
                },
                ondragend: move |event| {
                    if let Some(handler) = &on_drag_end {
                        handler.call(event);
                    }
                },
                title: "Open {card_title_for_open}",
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
                if !preview_items.is_empty() {
                    div { class: "app-card-preview mt-5",
                        p { class: "app-card-preview-header", "Contains" }
                        div { class: "mt-3 flex flex-wrap gap-2",
                            for item in preview_items {
                                span { class: "app-card-preview-item rounded-xl border px-3 py-2 text-xs font-semibold", "{item}" }
                            }
                        }
                    }
                }
            }

            if on_rename.is_some() || on_delete.is_some() {
                div { class: "flex items-center justify-end rounded-b-[1.75rem] border-t px-5 py-4", style: "border-color: var(--app-border); background-color: color-mix(in srgb, var(--app-surface-soft) 74%, transparent);",
                    div { class: "flex items-center gap-2",
                        if let Some(delete_handler) = on_delete {
                            button {
                                class: "{surface_destructive_icon_button_classes()}",
                                title: "Delete this card",
                                draggable: false,
                                onclick: move |_| {
                                    if confirm_destructive_action(&format!(
                                        "Delete the card '{card_title_for_delete}' and all of its descendants?"
                                    )) {
                                        delete_handler.call(());
                                    }
                                },
                                span { class: "shrink-0", {render_trash_icon()} }
                            }
                        }
                        if let Some(rename_handler) = on_rename {
                            button {
                                class: "{surface_action_button_classes()}",
                                title: "Edit this card",
                                draggable: false,
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
