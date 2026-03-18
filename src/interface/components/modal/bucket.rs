use crate::application::{Command, execute};
use crate::domain::card::UNASSIGNED_BUCKET_NAME;
use crate::domain::id::{BucketId, CardId};
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::components::modal::Modal;
use crate::interface::components::shared_forms::{inline_error, user_message_for_command_error};
use dioxus::prelude::*;
use tracing::{Level, warn};

#[component]
pub fn BucketModal(
    on_close: EventHandler<()>,
    card_id: CardId,
    registry: Signal<CardRegistry>,
) -> Element {
    let mut input_name = use_signal(String::new);
    let mut error_message = use_signal(|| None::<String>);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "New Column",
            div { class: "flex flex-col gap-4",
                input {
                    class: "app-input",
                    placeholder: "Column Name (e.g., Todo, Doing)",
                    value: "{input_name}",
                    oninput: move |e| input_name.set(e.value()),
                    autofocus: true,
                }
                if let Some(message) = error_message() { {inline_error(message)} }
                div { class: "flex justify-end gap-2",
                    button { class: "app-button-ghost px-4 py-2", onclick: move |_| on_close.call(()), "Cancel" }
                    button {
                        class: "app-button-primary px-6 py-3 disabled:opacity-50",
                        disabled: input_name().trim().is_empty(),
                        onclick: move |_| {
                            let mut reg = registry.write();
                            match execute(
                                Command::AddBucket {
                                    card_id,
                                    name: input_name().trim().to_string(),
                                },
                                &mut reg,
                            ) {
                                Ok(()) => {
                                    error_message.set(None);
                                    on_close.call(());
                                }
                                Err(error_value) => {
                                    error_message.set(Some(user_message_for_command_error(&error_value)))
                                }
                            }
                        },
                        "Add Column"
                    }
                }
            }
        }
    }
}

#[component]
pub fn EditBucketModal(
    on_close: EventHandler<()>,
    card_id: CardId,
    bucket_id: BucketId,
    registry: Signal<CardRegistry>,
) -> Element {
    let snapshot = {
        let reg = registry.read();
        let card = match reg.get_card(card_id) {
            Ok(card) => card,
            Err(_) => {
                return rsx! {
                    Modal { on_close: move |_| on_close.call(()), title: "Edit Column".to_string(),
                        p { class: "app-text-muted", "Board could not be loaded." }
                    }
                };
            }
        };

        let bucket = match card
            .buckets()
            .iter()
            .find(|bucket| bucket.id() == bucket_id)
        {
            Some(bucket) => bucket,
            None => {
                return rsx! {
                    Modal { on_close: move |_| on_close.call(()), title: "Edit Column".to_string(),
                        p { class: "app-text-muted", "Column could not be loaded." }
                    }
                };
            }
        };

        (
            bucket.name().to_string(),
            bucket.name() == UNASSIGNED_BUCKET_NAME,
        )
    };

    let (initial_name, is_unassigned) = snapshot;
    let mut input_name = use_signal(move || initial_name.clone());
    let mut error_message = use_signal(|| None::<String>);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "Edit Column",
            div { class: "flex flex-col gap-4",
                if is_unassigned {
                    p { class: "app-text-muted text-sm", "The Unassigned column cannot be renamed." }
                } else {
                    input {
                        class: "app-input",
                        value: "{input_name}",
                        oninput: move |e| input_name.set(e.value()),
                        autofocus: true,
                    }
                }
                if let Some(message) = error_message() { {inline_error(message)} }
                div { class: "flex justify-end gap-2",
                    button { class: "app-button-ghost px-4 py-2", onclick: move |_| on_close.call(()), "Cancel" }
                    button {
                        class: "app-button-primary px-6 py-3 disabled:opacity-50",
                        disabled: is_unassigned || input_name().trim().is_empty(),
                        onclick: move |_| {
                            if is_unassigned {
                                error_message.set(Some(
                                    "The Unassigned column cannot be renamed.".to_string(),
                                ));
                                return;
                            }

                            let mut reg = registry.write();
                            match execute(
                                Command::RenameBucket {
                                    card_id,
                                    bucket_id,
                                    new_name: input_name().trim().to_string(),
                                },
                                &mut reg,
                            ) {
                                Ok(()) => {
                                    error_message.set(None);
                                    on_close.call(());
                                }
                                Err(error_value) => {
                                    let message = user_message_for_command_error(&error_value);
                                    warn!(
                                        card_id = %card_id,
                                        bucket_id = %bucket_id,
                                        error = %error_value,
                                        "Bucket rename failed"
                                    );
                                    record_diagnostic(
                                        Level::WARN,
                                        "ui-modal",
                                        message.clone(),
                                    );
                                    error_message.set(Some(message));
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
