//! Reusable form components and utility functions for the interface.
//!
//! This module includes small-scale UI elements like error displays,
//! checkboxes, and data-transformation helpers for cards.
//!
//! For more on Rust's module system and documentation, see `docs/rust-for-python-devs.md`.

use crate::application::Command;
use crate::domain::due_date::DueDate;
use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use dioxus::prelude::*;

/// A macro for generating a standard form row with a label, input, and optional error.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     form_row! {
///         label: "Title",
///         id: "card-title",
///         input: rsx! {
///             input {
///                 id: "card-title",
///                 value: "{title}",
///                 oninput: move |e| title.set(e.value())
///             }
///         },
///         error: title_error()
///     }
/// }
/// ```
#[macro_export]
macro_rules! form_row {
    (label: $label:expr, id: $id:expr, input: $input:expr $(, error: $error:expr )? ) => {
        rsx! {
            div { class: "app-form-row",
                label { r#for: $id, class: "app-form-label", $label }
                $input
                $(
                    if let Some(msg) = $error {
                        p { class: "app-inline-error app-inline-error--danger", "{msg}" }
                    }
                )?
            }
        }
    };
}

/// Renders a small, inline error message, typically within a form or modal.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     inline_error("Invalid title".to_string())
/// }
/// ```
pub fn inline_error(message: String) -> Element {
    rsx! {
        p { class: "app-inline-error app-inline-error--danger",
            "{message}"
        }
    }
}

/// A labeled section that contains children and an action button.
///
/// Commonly used for grouping options in forms or modals.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     SelectorSection {
///         title: "Children".to_string(),
///         action_label: "Add".to_string(),
///         on_action: move |_| println!("Action clicked"),
///         p { "The list of children" }
///     }
/// }
/// ```
#[component]
pub fn SelectorSection(
    title: String,
    action_label: String,
    on_action: EventHandler<()>,
    #[props(default)] title_icon: Option<Element>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "app-form-section",
            div { class: "app-form-section-header",
                label { class: "app-kicker-inline",
                    if let Some(icon) = title_icon {
                        span { class: "app-icon-slot", {icon} }
                    }
                    "{title}"
                }
                button {
                    class: "app-button-secondary app-form-button-compact app-form-button-compact-xs",
                    onclick: move |_| on_action.call(()),
                    "{action_label}"
                }
            }
            {children}
        }
    }
}

/// A row containing a checkbox and a label.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     CheckboxOptionRow {
///         label_text: "My Option".to_string(),
///         checked: true,
///         on_toggle: move |_| println!("Toggled")
///     }
/// }
/// ```
#[component]
pub fn CheckboxOptionRow(
    label_text: String,
    checked: bool,
    on_toggle: EventHandler<()>,
) -> Element {
    rsx! {
        label { class: "app-checkbox-row",
            input {
                r#type: "checkbox",
                checked: checked,
                onclick: move |_| on_toggle.call(()),
            }
            span { class: "app-checkbox-label-text app-checkbox-label app-checkbox-label-strong", "{label_text}" }
        }
    }
}

/// Returns a user-friendly string representation of a domain error.
///
/// # Examples
///
/// ```
/// use kanban_planner::domain::error::DomainError;
/// use kanban_planner::interface::components::shared_forms::user_message_for_command_error;
///
/// let error = DomainError::InvalidOperation("Failed".to_string());
/// assert_eq!(user_message_for_command_error(&error), "Invalid operation: Failed");
/// ```
pub fn user_message_for_command_error(error: &DomainError) -> String {
    error.to_string()
}

/// Toggles an ID's presence in a vector signal.
///
/// # Examples
///
/// ```ignore
/// let mut signal = use_signal(|| vec![1, 2, 3]);
/// toggle_id(&mut signal, 2);
/// // signal is now [1, 3]
/// ```
pub fn toggle_id<T>(signal: &mut Signal<Vec<T>>, id: T)
where
    T: Copy + Eq + 'static,
{
    let mut values = signal();
    if values.contains(&id) {
        values.retain(|value| *value != id);
    } else {
        values.push(id);
    }
    signal.set(values);
}

/// Utility for building a `Command` for card creation based on optional parent ID.
///
/// # Examples
///
/// ```
/// use kanban_planner::interface::components::shared_forms::build_create_card_command;
///
/// let cmd = build_create_card_command("My Title".to_string(), None, None).unwrap();
/// ```
pub fn build_create_card_command(
    title: String,
    description: Option<String>,
    parent_id: Option<CardId>,
) -> Result<Command, DomainError> {
    match parent_id {
        Some(parent_id) => Ok(Command::CreateChildCard {
            title,
            description,
            parent_id,
        }),
        None => Ok(Command::CreateWorkspaceChildCard {
            title,
            description,
        }),
    }
}

/// Provides a formatted string for a `DueDate`, or an empty string if it's `None`.
///
/// # Examples
///
/// ```
/// use kanban_planner::domain::due_date::DueDate;
/// use kanban_planner::interface::components::shared_forms::due_date_string;
///
/// let due = DueDate::parse("2023-12-31").unwrap();
/// assert_eq!(due_date_string(Some(&due)), "2023-12-31");
/// assert_eq!(due_date_string(None), "");
/// ```
pub fn due_date_string(due_date: Option<&DueDate>) -> String {
    due_date.map(|due| due.to_string()).unwrap_or_default()
}
