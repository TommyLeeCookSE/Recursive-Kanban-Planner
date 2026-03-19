use crate::application::{Command, execute};
use crate::domain::id::CardId;
use crate::domain::label::LabelColor;
use crate::domain::registry::CardRegistry;
use crate::interface::components::modal::{Modal, ModalType};
use crate::interface::components::shared_forms::{
    CheckboxOptionRow, SelectorSection, inline_error, parse_label_color, toggle_id,
    user_message_for_command_error,
};
use crate::interface::components::visuals::{render_label_chip, render_label_icon};
use dioxus::prelude::*;

#[component]
pub fn CardLabelsModal(
    on_close: EventHandler<()>,
    card_id: CardId,
    registry: Signal<CardRegistry>,
) -> Element {
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();
    let card_snapshot = {
        let reg = registry.read();
        let card = match reg.get_card(card_id) {
            Ok(card) => card.clone(),
            Err(_) => {
                return rsx! {
                    Modal { on_close: move |_| on_close.call(()), title: "Edit Labels".to_string(),
                        p { class: "app-text-muted", "Card could not be loaded." }
                    }
                };
            }
        };
        (card, reg.label_definitions().to_vec())
    };

    let (card, label_definitions) = card_snapshot;
    let initial_labels = card.label_ids().to_vec();
    let mut selected_labels = use_signal(move || initial_labels.clone());
    let mut error_message = use_signal(|| None::<String>);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "Edit Labels",
            div { class: "flex flex-col gap-4",
                SelectorSection {
                    title: "Assigned Labels".to_string(),
                    action_label: "Manage Labels".to_string(),
                    on_action: move |_| active_modal.set(Some(ModalType::ManageLabels {})),
                    title_icon: Some(render_label_icon()),
                    if label_definitions.is_empty() {
                        p { class: "app-text-muted text-sm", "No labels created yet." }
                    } else {
                        div { class: "space-y-3",
                            for label in label_definitions.iter().cloned() {
                                CheckboxOptionRow {
                                    label_text: label.name().to_string(),
                                    checked: selected_labels().contains(&label.id()),
                                    on_toggle: move |_| toggle_id(&mut selected_labels, label.id()),
                                }
                            }
                        }
                    }
                }

                if let Some(message) = error_message() { {inline_error(message)} }

                div { class: "flex justify-end gap-2",
                    button { class: "app-button-ghost px-4 py-2", onclick: move |_| on_close.call(()), "Cancel" }
                    button {
                        class: "app-button-primary px-6 py-3",
                        onclick: move |_| {
                            let mut reg = registry.write();
                            match execute(Command::SetCardLabels { card_id, label_ids: selected_labels() }, &mut reg) {
                                Ok(()) => {
                                    error_message.set(None);
                                    on_close.call(());
                                }
                                Err(error_value) => error_message.set(Some(user_message_for_command_error(&error_value))),
                            }
                        },
                        "Save Labels"
                    }
                }
            }
        }
    }
}

#[component]
pub fn ManageLabelsModal(on_close: EventHandler<()>, registry: Signal<CardRegistry>) -> Element {
    let mut name_input = use_signal(String::new);
    let mut color_input = use_signal(|| LabelColor::Ember.as_str().to_string());
    let mut error_message = use_signal(|| None::<String>);
    let labels = registry.read().label_definitions().to_vec();

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "Manage Labels",
            div { class: "flex flex-col gap-4",
                input { class: "app-input", placeholder: "Label name", value: "{name_input}", oninput: move |e| name_input.set(e.value()) }
                select {
                    class: "app-input",
                    value: "{color_input}",
                    oninput: move |e| color_input.set(e.value()),
                    for color in LabelColor::ALL {
                        option { value: "{color.as_str()}", "{color.as_str()}" }
                    }
                }
                button {
                    class: "app-button-primary px-6 py-3",
                    onclick: move |_| {
                        let color = parse_label_color(&color_input());
                        let mut reg = registry.write();
                        match execute(Command::CreateLabelDefinition { name: name_input().trim().to_string(), color }, &mut reg) {
                            Ok(()) => {
                                name_input.set(String::new());
                                error_message.set(None);
                            }
                            Err(error_value) => error_message.set(Some(user_message_for_command_error(&error_value))),
                        }
                    },
                    "Create Label"
                }
                if let Some(message) = error_message() { {inline_error(message)} }
                div { class: "space-y-3",
                    for label in labels.iter().cloned() {
                        div { class: "flex items-center justify-between gap-3 rounded-xl border px-4 py-3", style: "border-color: var(--app-border);",
                            div { class: "flex items-center gap-3",
                                {render_label_chip(label.name().to_string(), label.color())}
                            }
                            button {
                                class: "app-danger-button px-3 py-2",
                                onclick: move |_| {
                                    let mut reg = registry.write();
                                    if execute(Command::DeleteLabelDefinition { label_id: label.id() }, &mut reg).is_err() {}
                                },
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}
