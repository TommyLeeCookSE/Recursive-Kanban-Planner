use crate::application::{Command, execute};
use crate::domain::registry::CardRegistry;
use crate::domain::rule::RuleAction;
use crate::interface::components::modal::Modal;
use crate::interface::components::shared_forms::{
    build_rule_trigger, collect_bucket_choices, describe_rule_action, describe_rule_trigger,
    inline_error, user_message_for_command_error,
};
use dioxus::prelude::*;

#[component]
pub fn ManageRulesModal(on_close: EventHandler<()>, registry: Signal<CardRegistry>) -> Element {
    let mut name_input = use_signal(String::new);
    let mut trigger_kind = use_signal(|| "NoteOpened".to_string());
    let mut popup_title = use_signal(String::new);
    let mut popup_message = use_signal(String::new);
    let mut selected_bucket_id = use_signal(String::new);
    let mut error_message = use_signal(|| None::<String>);
    let rules = registry.read().rule_definitions().to_vec();
    let bucket_choices = collect_bucket_choices(&registry.read());

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "Manage Rules",
            div { class: "flex flex-col gap-4",
                input { class: "app-input", placeholder: "Rule name", value: "{name_input}", oninput: move |e| name_input.set(e.value()) }
                select {
                    class: "app-input",
                    value: "{trigger_kind}",
                    oninput: move |e| trigger_kind.set(e.value()),
                    option { value: "NoteOpened", "Note Opened" }
                    option { value: "NoteClosed", "Note Closed" }
                    option { value: "MovedToBucket", "Moved To Bucket" }
                }
                if trigger_kind() == "MovedToBucket" {
                    select {
                        class: "app-input",
                        value: "{selected_bucket_id}",
                        oninput: move |e| selected_bucket_id.set(e.value()),
                        option { value: "", "Select a bucket" }
                        for (id, label) in bucket_choices.iter().cloned() {
                            option { value: "{id}", "{label}" }
                        }
                    }
                }
                input { class: "app-input", placeholder: "Popup title", value: "{popup_title}", oninput: move |e| popup_title.set(e.value()) }
                textarea { class: "app-input min-h-[10rem]", placeholder: "Popup message", value: "{popup_message}", oninput: move |e| popup_message.set(e.value()) }
                button {
                    class: "app-button-primary px-6 py-3",
                    onclick: move |_| {
                        let trigger = match build_rule_trigger(&trigger_kind(), &selected_bucket_id()) {
                            Ok(trigger) => trigger,
                            Err(error_value) => {
                                error_message.set(Some(user_message_for_command_error(&error_value)));
                                return;
                            }
                        };
                        let mut reg = registry.write();
                        match execute(
                            Command::CreateRuleDefinition {
                                name: name_input().trim().to_string(),
                                trigger,
                                action: RuleAction::ShowPopup {
                                    title: popup_title().trim().to_string(),
                                    message: popup_message().trim().to_string(),
                                },
                            },
                            &mut reg,
                        ) {
                            Ok(()) => {
                                name_input.set(String::new());
                                popup_title.set(String::new());
                                popup_message.set(String::new());
                                selected_bucket_id.set(String::new());
                                trigger_kind.set("NoteOpened".to_string());
                                error_message.set(None);
                            }
                            Err(error_value) => error_message.set(Some(user_message_for_command_error(&error_value))),
                        }
                    },
                    "Create Rule"
                }
                if let Some(message) = error_message() { {inline_error(message)} }
                div { class: "space-y-3",
                    for rule in rules.iter().cloned() {
                        div { class: "rounded-xl border px-4 py-3", style: "border-color: var(--app-border);",
                            div { class: "mb-2 flex items-center justify-between gap-3",
                                h3 { class: "app-text-primary font-semibold", "{rule.name()}" }
                                button {
                                    class: "app-danger-button px-3 py-2",
                                    onclick: move |_| {
                                        let mut reg = registry.write();
                                        if execute(Command::DeleteRuleDefinition { rule_id: rule.id() }, &mut reg).is_err() {}
                                    },
                                    "Delete"
                                }
                            }
                            p { class: "app-text-muted text-sm", "{describe_rule_trigger(rule.trigger())}" }
                            p { class: "app-text-soft mt-1 text-sm", "{describe_rule_action(rule.action())}" }
                        }
                    }
                }
            }
        }
    }
}
