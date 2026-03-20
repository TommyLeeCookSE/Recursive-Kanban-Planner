use crate::application::Command;
use crate::domain::due_date::DueDate;
use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use dioxus::prelude::*;

pub fn inline_error(message: String) -> Element {
    rsx! {
        p { class: "app-inline-error app-inline-error--danger",
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
