//! The card item component for the Kanban Planner.
//!
//! This module provides the main interactive card component used across
//! the workspace and board views to represent individual tasks or projects.
//!
//! For an overview of how components are structured, see
//! `docs/rust-for-python-devs.md`.

use crate::interface::actions::interop::confirm_destructive_action;
use crate::interface::components::visuals::{
    render_drag_handle_icon, render_edit_icon, render_trash_icon,
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
    let delete_title = title.clone();

    rsx! {
        article {
            class: "app-card-surface app-card-shell group",

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
                title: "Open {title}",
                onclick: move |_| on_open.call(()),
                div { class: "app-card-title-stack relative w-full",
                    if draggable {
                        div { class: "app-card-drag-handle",
                            {render_drag_handle_icon()}
                        }
                    }
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
                            class: if is_overdue { "app-card-due app-card-due--overdue" } else { "app-card-due app-card-due-normal" },
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
                div { class: "app-card-actions",
                    div { class: "app-card-actions-inner",
                        if let Some(delete_handler) = on_delete {
                            button {
                                class: "app-bar-button app-bar-button--danger app-bar-button--sm",
                                title: "Delete this card",
                                draggable: false,
                                onclick: move |_| {
                                    if confirm_destructive_action(&format!(
                                        "Delete the card '{delete_title}' and all of its descendants?"
                                    )) {
                                        delete_handler.call(());
                                    }
                                },
                                span { class: "app-bar-button-icon", {render_trash_icon()} }
                            }
                        }
                        if let Some(rename_handler) = on_rename {
                            button {
                                class: "app-bar-button app-bar-button--sm",
                                title: "Edit this card",
                                draggable: false,
                                onclick: move |_| rename_handler.call(()),
                                span { class: "app-bar-button-icon", {render_edit_icon()} }
                            }
                        }
                    }
                }
            }
        }
    }
}
