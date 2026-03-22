use crate::application::{Command, execute};
use crate::domain::due_date::DueDate;
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::domain::title::MAX_TITLE_LENGTH;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::components::modal::Modal;
use crate::interface::components::shared_forms::{
    build_create_card_command, inline_error, user_message_for_command_error,
};
use dioxus::prelude::*;
use tracing::{Level, warn};

#[component]
pub fn CardModal(
    on_close: EventHandler<()>,
    parent_id: Option<CardId>,
    registry: Signal<CardRegistry>,
) -> Element {
    let mut input_title = use_signal(String::new);
    let mut input_description = use_signal(String::new);
    let mut error_message = use_signal(|| None::<String>);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: if parent_id.is_some() { "New Card" } else { "New Board" },
            div { class: "app-form-stack",
                input {
                    class: "app-input",
                    placeholder: "Enter title...",
                    value: "{input_title}",
                    oninput: move |e| input_title.set(e.value()),
                    maxlength: MAX_TITLE_LENGTH as i64,
                    autofocus: true,
                }
                input {
                    class: "app-input",
                    placeholder: "Enter short description (optional)...",
                    value: "{input_description}",
                    oninput: move |e| input_description.set(e.value()),
                    maxlength: 80,
                }
                if let Some(message) = error_message() {
                    {inline_error(message)}
                }
                div { class: "app-form-actions",
                    button {
                        class: "app-button-ghost-compact",
                        title: "Cancel card creation",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "app-button-primary-compact disabled:opacity-50",
                        title: "Create this card",
                        disabled: input_title().trim().is_empty(),
                        onclick: move |_| {
                            let trimmed_title = input_title().trim().to_string();
                            let desc_raw = input_description();
                            let trimmed_desc = desc_raw.trim();
                            let description = if trimmed_desc.is_empty() { None } else { Some(trimmed_desc.to_string()) };

                            let command = match build_create_card_command(trimmed_title, description, parent_id) {
                                Ok(command) => command,
                                Err(error_value) => {
                                    let user_message = user_message_for_command_error(&error_value);
                                    warn!(
                                        parent_id = ?parent_id,
                                        error = %error_value,
                                        "Card modal rejected invalid create-card context"
                                    );
                                    error_message.set(Some(user_message.clone()));
                                    record_diagnostic(Level::WARN, "ui-modal", user_message);
                                    return;
                                }
                            };

                            let mut reg = registry.write();
                            match execute(command, &mut reg) {
                                Ok(()) => {
                                    error_message.set(None);
                                    on_close.call(());
                                }
                                Err(error_value) => {
                                    error_message.set(Some(user_message_for_command_error(&error_value)))
                                }
                            }
                        },
                        "Create Card"
                    }
                }
            }
        }
    }
}

#[component]
pub fn EditCardModal(
    on_close: EventHandler<()>,
    id: CardId,
    registry: Signal<CardRegistry>,
) -> Element {
    let card_snapshot = {
        let reg = registry.read();
        match reg.get_card(id) {
            Ok(card) => card.clone(),
            Err(_) => {
                return rsx! {
                    Modal { on_close: move |_| on_close.call(()), title: "Edit Item".to_string(),
                        p { class: "app-error-message", "Card could not be loaded." }
                    }
                };
            }
        }
    };

    let card = card_snapshot;
    let initial_title = card.title().to_string();
    let initial_description = card.description().unwrap_or_default().to_string();
    let initial_due_date = card
        .due_date()
        .map(|due| due.to_string())
        .unwrap_or_default();
    let mut input_title = use_signal(move || initial_title.clone());
    let mut input_description = use_signal(move || initial_description.clone());
    let mut due_date_input = use_signal(move || initial_due_date.clone());
    let mut error_message = use_signal(|| None::<String>);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "Edit Card",
            div { class: "app-form-stack",
                label { class: "app-kicker", "Title" }
                input {
                    class: "app-input",
                    value: "{input_title}",
                    oninput: move |e| input_title.set(e.value()),
                    maxlength: MAX_TITLE_LENGTH as i64,
                    autofocus: true,
                }

                label { class: "app-kicker", "Description" }
                input {
                    class: "app-input",
                    value: "{input_description}",
                    oninput: move |e| input_description.set(e.value()),
                    maxlength: 80,
                    placeholder: "Enter short description...",
                }

                label { class: "app-kicker", "Due Date" }
                input {
                    class: "app-input",
                    r#type: "date",
                    value: "{due_date_input}",
                    oninput: move |e| due_date_input.set(e.value()),
                }

                if let Some(message) = error_message() { {inline_error(message)} }

                div { class: "app-form-actions",
                    button {
                        class: "app-button-ghost-compact",
                        title: "Cancel editing this card",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "app-button-primary-compact disabled:opacity-50",
                        title: "Save card changes",
                        disabled: input_title().trim().is_empty(),
                        onclick: move |_| {
                            let due_date_value = if due_date_input().trim().is_empty() {
                                None
                            } else {
                                match DueDate::parse(due_date_input()) {
                                    Ok(due_date) => Some(due_date),
                                    Err(error_value) => {
                                        error_message.set(Some(user_message_for_command_error(&error_value)));
                                        return;
                                    }
                                }
                            };

                            let mut reg = registry.write();
                            let rename_result = execute(
                                Command::RenameCard {
                                    id,
                                    title: input_title().trim().to_string(),
                                },
                                &mut reg,
                            );
                            let desc_raw = input_description();
                            let desc_trimmed = desc_raw.trim();
                            let description = if desc_trimmed.is_empty() { None } else { Some(desc_trimmed.to_string()) };
                            let desc_result = execute(
                                Command::SetCardDescription { id, description },
                                &mut reg,
                            );
                            let due_result = match due_date_value {
                                Some(due_date) => {
                                    execute(Command::SetDueDate { card_id: id, due_date }, &mut reg)
                                }
                                None => execute(Command::ClearDueDate { card_id: id }, &mut reg),
                            };

                            if let Some(error_value) = rename_result.err().or_else(|| desc_result.err()).or_else(|| due_result.err()) {
                                error_message.set(Some(user_message_for_command_error(&error_value)));
                            } else {
                                error_message.set(None);
                                on_close.call(());
                            }
                        },
                        "Save Changes"
                    }
                }
            }
        }
    }
}
