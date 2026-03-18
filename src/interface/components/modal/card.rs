use crate::application::{CardRuleEvent, Command, PopupNotification, execute};
use crate::domain::due_date::DueDate;
use crate::domain::id::{BucketId, CardId};
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::actions::{dispatch_card_rule_event, report_result};
use crate::interface::components::modal::Modal;
use crate::interface::components::modal::ModalType;
use crate::interface::components::shared_forms::{
    build_create_card_command, inline_error, toggle_id, user_message_for_command_error,
};
use dioxus::prelude::*;
use tracing::{Level, warn};

#[component]
pub fn CardModal(
    on_close: EventHandler<()>,
    parent_id: Option<CardId>,
    bucket_id: Option<BucketId>,
    registry: Signal<CardRegistry>,
) -> Element {
    let mut input_title = use_signal(String::new);
    let mut error_message = use_signal(|| None::<String>);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: if parent_id.is_some() { "New Child Card" } else { "New Root Board" },
            div { class: "flex flex-col gap-4",
                input {
                    class: "app-input",
                    placeholder: "Enter title...",
                    value: "{input_title}",
                    oninput: move |e| input_title.set(e.value()),
                    autofocus: true,
                }
                if let Some(message) = error_message() {
                    {inline_error(message)}
                }
                div { class: "flex justify-end gap-2",
                    button { class: "app-button-ghost px-4 py-2", onclick: move |_| on_close.call(()), "Cancel" }
                    button {
                        class: "app-button-primary px-6 py-3 disabled:opacity-50",
                        disabled: input_title().trim().is_empty(),
                        onclick: move |_| {
                            let trimmed_title = input_title().trim().to_string();
                            let command = match build_create_card_command(trimmed_title, parent_id, bucket_id) {
                                Ok(command) => command,
                                Err(error_value) => {
                                    let user_message = user_message_for_command_error(&error_value);
                                    warn!(
                                        parent_id = ?parent_id,
                                        bucket_id = ?bucket_id,
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
                        "Create Item"
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
    let popup_queue = use_context::<Signal<Vec<PopupNotification>>>();
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();
    let warning_message = use_context::<Signal<Option<String>>>();

    let card_snapshot = {
        let reg = registry.read();
        let card = match reg.get_card(id) {
            Ok(card) => card.clone(),
            Err(_) => {
                return rsx! {
                    Modal { on_close: move |_| on_close.call(()), title: "Edit Item".to_string(),
                        p { class: "app-text-muted", "Card could not be loaded." }
                    }
                };
            }
        };
        (
            card,
            reg.label_definitions().to_vec(),
            reg.rule_definitions().to_vec(),
        )
    };

    let (card, label_definitions, rule_definitions) = card_snapshot;
    let initial_title = card.title().to_string();
    let initial_due_date = card
        .due_date()
        .map(|due| due.to_string())
        .unwrap_or_default();
    let initial_labels = card.label_ids().to_vec();
    let initial_rules = card.rule_ids().to_vec();
    let mut input_title = use_signal(move || initial_title.clone());
    let mut due_date_input = use_signal(move || initial_due_date.clone());
    let mut selected_labels = use_signal(move || initial_labels.clone());
    let mut selected_rules = use_signal(move || initial_rules.clone());
    let mut error_message = use_signal(|| None::<String>);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "Edit Item",
            div { class: "flex flex-col gap-4",
                label { class: "app-kicker", "Title" }
                input {
                    class: "app-input",
                    value: "{input_title}",
                    oninput: move |e| input_title.set(e.value()),
                    autofocus: true,
                }

                label { class: "app-kicker", "Due Date" }
                input {
                    class: "app-input",
                    r#type: "date",
                    value: "{due_date_input}",
                    oninput: move |e| due_date_input.set(e.value()),
                }

                div { class: "flex items-center justify-between gap-4",
                    label { class: "app-kicker", "Labels" }
                    button {
                        class: "app-button-secondary px-4 py-2 text-xs",
                        onclick: move |_| active_modal.set(Some(ModalType::ManageLabels {})),
                        "Manage Labels"
                    }
                }
                if label_definitions.is_empty() {
                    p { class: "app-text-muted text-sm", "No labels created yet." }
                } else {
                    div { class: "space-y-3",
                        for label in label_definitions.iter().cloned() {
                            label { class: "flex items-center gap-3 rounded-xl border px-4 py-3", style: "border-color: var(--app-border);",
                                input {
                                    r#type: "checkbox",
                                    checked: selected_labels().contains(&label.id()),
                                    onclick: move |_| toggle_id(&mut selected_labels, label.id()),
                                }
                                span { class: "app-text-primary text-sm font-medium", "{label.name()}" }
                            }
                        }
                    }
                }

                div { class: "flex items-center justify-between gap-4",
                    label { class: "app-kicker", "Rules" }
                    button {
                        class: "app-button-secondary px-4 py-2 text-xs",
                        onclick: move |_| active_modal.set(Some(ModalType::ManageRules {})),
                        "Manage Rules"
                    }
                }
                if rule_definitions.is_empty() {
                    p { class: "app-text-muted text-sm", "No rules created yet." }
                } else {
                    div { class: "space-y-3",
                        for rule in rule_definitions.iter().cloned() {
                            label { class: "flex items-center gap-3 rounded-xl border px-4 py-3", style: "border-color: var(--app-border);",
                                input {
                                    r#type: "checkbox",
                                    checked: selected_rules().contains(&rule.id()),
                                    onclick: move |_| toggle_id(&mut selected_rules, rule.id()),
                                }
                                span { class: "app-text-primary text-sm font-medium", "{rule.name()}" }
                            }
                        }
                    }
                }

                if let Some(message) = error_message() { {inline_error(message)} }

                div { class: "flex flex-wrap justify-between gap-3",
                    button {
                        class: "app-button-secondary px-4 py-2",
                        onclick: move |_| {
                            let result = dispatch_card_rule_event(
                                id,
                                CardRuleEvent::NoteOpened,
                                registry,
                                popup_queue,
                                "ui-modal",
                            );
                            let _ = report_result(
                                result,
                                warning_message,
                                "ui-modal",
                                "dispatch note-opened rule",
                            );
                            active_modal.set(Some(ModalType::CardNotes { card_id: id }));
                        },
                        "Open Notes"
                    }
                    div { class: "flex gap-2",
                        button { class: "app-button-ghost px-4 py-2", onclick: move |_| on_close.call(()), "Cancel" }
                        button {
                            class: "app-button-primary px-6 py-3 disabled:opacity-50",
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
                                    Command::RenameCard { id, title: input_title().trim().to_string() },
                                    &mut reg,
                                );
                                let due_result = match due_date_value {
                                    Some(due_date) => execute(Command::SetDueDate { card_id: id, due_date }, &mut reg),
                                    None => execute(Command::ClearDueDate { card_id: id }, &mut reg),
                                };
                                let labels_result = execute(
                                    Command::SetCardLabels { card_id: id, label_ids: selected_labels() },
                                    &mut reg,
                                );
                                let rules_result = execute(
                                    Command::SetCardRules { card_id: id, rule_ids: selected_rules() },
                                    &mut reg,
                                );

                                if let Some(error_value) = rename_result
                                    .err()
                                    .or_else(|| due_result.err())
                                    .or_else(|| labels_result.err())
                                    .or_else(|| rules_result.err())
                                {
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
}
