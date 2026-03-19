use crate::application::Command;
use crate::domain::due_date::DueDate;
use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use dioxus::prelude::*;

pub fn inline_error(message: String) -> Element {
    rsx! {
        p { class: "rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-900/50 dark:bg-red-950/40 dark:text-red-200",
            "{message}"
        }
    }
}

#[component]
pub fn SelectorSection(
    title: String,
    action_label: String,
    on_action: EventHandler<()>,
    #[props(default)] title_icon: Option<Element>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "flex flex-col gap-4",
            div { class: "flex items-center justify-between gap-4",
                label { class: "app-kicker flex items-center gap-2",
                    if let Some(icon) = title_icon {
                        span { class: "shrink-0", {icon} }
                    }
                    "{title}"
                }
                button {
                    class: "app-button-secondary px-4 py-2 text-xs",
                    onclick: move |_| on_action.call(()),
                    "{action_label}"
                }
            }
            {children}
        }
    }
}

#[component]
pub fn CheckboxOptionRow(
    label_text: String,
    checked: bool,
    on_toggle: EventHandler<()>,
) -> Element {
    rsx! {
        label { class: "flex items-center gap-3 rounded-xl border px-4 py-3", style: "border-color: var(--app-border);",
            input {
                r#type: "checkbox",
                checked: checked,
                onclick: move |_| on_toggle.call(()),
            }
            span { class: "app-text-primary text-sm font-medium", "{label_text}" }
        }
    }
}

pub fn user_message_for_command_error(error: &DomainError) -> String {
    error.to_string()
}

pub fn confirm_destructive_action(message: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|window| window.confirm_with_message(message).ok())
            .unwrap_or(false)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = message;
        true
    }
}

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

pub fn build_create_card_command(
    title: String,
    parent_id: Option<CardId>,
) -> Result<Command, DomainError> {
    match parent_id {
        Some(parent_id) => Ok(Command::CreateChildCard { title, parent_id }),
        None => Ok(Command::CreateWorkspaceChildCard { title }),
    }
}

pub fn due_date_string(due_date: Option<&DueDate>) -> String {
    due_date.map(|due| due.to_string()).unwrap_or_default()
}
