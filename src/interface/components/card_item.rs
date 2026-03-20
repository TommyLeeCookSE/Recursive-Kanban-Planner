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
    /// Optional second line of text for metadata.
    #[props(default)]
    subtitle: Option<String>,
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
                    "app-card-body app-card-body--draggable"
                } else {
                    "app-card-body"
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
                div { class: "app-card-title-stack",
                    h3 { class: "app-card-title",
                        "{title}"
                    }
                    if let Some(subtitle) = subtitle {
                        p { class: "app-card-subtitle",
                            "{subtitle}"
                        }
                    }
                    if let Some(due_date) = due_date {
                        p {
                            class: if is_overdue { "mt-4 text-left text-sm font-semibold text-red-500" } else { "app-text-muted mt-4 text-left text-sm font-semibold" },
                            "Due {due_date}"
                        }
                    }
                    if !preview_items.is_empty() {
                        div { class: "app-card-preview-shell",
                            div { class: "app-card-preview-items",
                                for item in preview_items {
                                    span {
                                        class: "app-card-preview-chip",
                                        "{item}"
                                    }
                                }
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
