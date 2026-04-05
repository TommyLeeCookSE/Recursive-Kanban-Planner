use crate::application::Command;
use crate::domain::due_date::DueDate;
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::domain::title::MAX_TITLE_LENGTH;
use crate::form_row;
use crate::interface::components::modal::{Modal, ModalActions};
use crate::interface::components::shared_forms::{inline_error, modal_dispatch_command};
use dioxus::prelude::*;

#[component]
pub fn CardModal(
    on_close: EventHandler<()>,
    parent_id: Option<CardId>,
    registry: Signal<CardRegistry>,
) -> Element {
    let mut input_title = use_signal(String::new);
    let mut input_description = use_signal(String::new);
    let error_message = use_signal(|| None::<String>);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: if parent_id.is_some() { "New Card" } else { "New Board" },
            div { class: "app-form-stack",
                {form_row! {
                    label: "Title",
                    id: "card-title",
                    input: rsx! {
                        input {
                            id: "card-title",
                            class: "app-input",
                            placeholder: "Enter title...",
                            value: "{input_title}",
                            oninput: move |e| input_title.set(e.value()),
                            maxlength: MAX_TITLE_LENGTH as i64,
                            autofocus: true,
                        }
                    }
                }}
                {form_row! {
                    label: "Description",
                    id: "card-description",
                    input: rsx! {
                        input {
                            id: "card-description",
                            class: "app-input",
                            placeholder: "Enter description (optional)...",
                            value: "{input_description}",
                            oninput: move |e| input_description.set(e.value()),
                        }
                    }
                }}
                if let Some(message) = error_message() {
                    {inline_error(message)}
                }
                ModalActions {
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
                            let title = input_title().trim().to_string();
                            let description = if input_description().trim().is_empty() {
                                None
                            } else {
                                Some(input_description().trim().to_string())
                            };
                            modal_dispatch_command(
                                Command::CreateCard {
                                    title,
                                    description,
                                    parent_id,
                                },
                                registry,
                                error_message,
                                move || on_close.call(()),
                            );
                        },
                        "Create"
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
    initial_title: String,
    initial_description: String,
    initial_due_date: String,
    registry: Signal<CardRegistry>,
) -> Element {
    let mut input_title = use_signal({
        let it = initial_title.clone();
        move || it.clone()
    });
    let mut input_description = use_signal({
        let id = initial_description.clone();
        move || id.clone()
    });
    let mut due_date_input = use_signal({
        let idd = initial_due_date.clone();
        move || idd.clone()
    });
    let error_message = use_signal(|| None::<String>);

    rsx! {
        Modal { on_close: move |_| on_close.call(()), title: "Edit Card",
            div { class: "app-form-stack",
                {form_row! {
                    label: "Title",
                    id: "edit-title",
                    input: rsx! {
                        input {
                            id: "edit-title",
                            class: "app-input",
                            value: "{input_title}",
                            oninput: move |e| input_title.set(e.value()),
                            maxlength: MAX_TITLE_LENGTH as i64,
                            autofocus: true,
                        }
                    }
                }}

                {form_row! {
                    label: "Description",
                    id: "edit-description",
                    input: rsx! {
                        input {
                            id: "edit-description",
                            class: "app-input",
                            value: "{input_description}",
                            oninput: move |e| input_description.set(e.value()),
                            maxlength: 80,
                            placeholder: "Enter short description...",
                        }
                    }
                }}

                {form_row! {
                    label: "Due Date",
                    id: "edit-due-date",
                    input: rsx! {
                        input {
                            id: "edit-due-date",
                            class: "app-input",
                            r#type: "date",
                            value: "{due_date_input}",
                            oninput: move |e| due_date_input.set(e.value()),
                        }
                    }
                }}

                if let Some(message) = error_message() {
                    {inline_error(message)}
                }

                ModalActions {
                    button {
                        class: "app-button-ghost-compact",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "app-button-primary-compact",
                        disabled: input_title().trim().is_empty(),
                        onclick: {
                            let initial_title = initial_title.clone();
                            let initial_description = initial_description.clone();
                            let initial_due_date = initial_due_date.clone();
                            move |_| {
                                let title = if input_title() != initial_title {
                                    Some(input_title().trim().to_string())
                                } else {
                                    None
                                };
                                let description = if input_description() != initial_description {
                                    Some(if input_description().trim().is_empty() {
                                        None
                                      } else {
                                        Some(input_description().trim().to_string())
                                      })
                                } else {
                                    None
                                };
                                let due_date = if due_date_input() != initial_due_date {
                                    Some(if due_date_input().trim().is_empty() {
                                        None
                                    } else {
                                        DueDate::parse(due_date_input()).ok()
                                    })
                                } else {
                                    None
                                };
                                if title.is_some() || description.is_some() || due_date.is_some() {
                                    modal_dispatch_command(
                                        Command::UpdateCardDetails {
                                            id,
                                            title,
                                            description,
                                            due_date,
                                        },
                                        registry,
                                        error_message,
                                        move || on_close.call(()),
                                    );
                                } else {
                                    on_close.call(());
                                }
                            }
                        },
                        "Save Changes"
                    }
                }
            }
        }
    }
}
