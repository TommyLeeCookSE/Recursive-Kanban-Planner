use crate::application::{Command, execute};
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::interface::components::modal::Modal;
use crate::interface::components::shared_forms::{inline_error, user_message_for_command_error};
use dioxus::prelude::*;

#[component]
pub fn NotesModal(
    on_close: EventHandler<()>,
    card_id: CardId,
    registry: Signal<CardRegistry>,
) -> Element {
    let notes = {
        let reg = registry.read();
        match reg.get_card(card_id) {
            Ok(card) => card.notes().to_vec(),
            Err(_) => Vec::new(),
        }
    };

    let initial_selected = notes.first().map(|note| note.id());
    let mut selected_note_id = use_signal(move || initial_selected);
    let mut title_input = use_signal(|| {
        notes
            .first()
            .map(|note| note.title().to_string())
            .unwrap_or_default()
    });
    let mut body_input = use_signal(|| {
        notes
            .first()
            .map(|note| note.body().to_string())
            .unwrap_or_default()
    });
    let mut new_page_title = use_signal(|| "New Page".to_string());
    let mut error_message = use_signal(|| None::<String>);

    let current_notes = {
        let reg = registry.read();
        reg.get_card(card_id)
            .map(|card| card.notes().to_vec())
            .unwrap_or_default()
    };

    let selected_note = current_notes
        .iter()
        .find(|note| Some(note.id()) == selected_note_id());

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "Notebook",
            div { class: "flex max-h-[75vh] flex-col gap-4 lg:flex-row",
                div { class: "w-full space-y-3 lg:w-56",
                    input {
                        class: "app-input",
                        placeholder: "New page title",
                        value: "{new_page_title}",
                        oninput: move |e| new_page_title.set(e.value()),
                    }
                    button {
                        class: "app-button-secondary w-full px-4 py-2 text-sm",
                        title: "Add a new note page",
                        onclick: move |_| {
                            let title = new_page_title().trim().to_string();
                            let mut reg = registry.write();
                            match execute(Command::AddNotePage { card_id, title }, &mut reg) {
                                Ok(()) => {
                                    let latest_note = reg.get_card(card_id).ok().and_then(|card| card.notes().last().cloned());
                                    if let Some(note) = latest_note {
                                        selected_note_id.set(Some(note.id()));
                                        title_input.set(note.title().to_string());
                                        body_input.set(note.body().to_string());
                                    }
                                    new_page_title.set("New Page".to_string());
                                    error_message.set(None);
                                }
                                Err(error_value) => error_message.set(Some(user_message_for_command_error(&error_value))),
                            }
                        },
                        "Add Page"
                    }
                    div { class: "max-h-64 space-y-2 overflow-y-auto pr-1",
                        for note in current_notes.iter().cloned() {
                            button {
                                class: if Some(note.id()) == selected_note_id() { "app-button-primary w-full px-4 py-2 text-left text-sm" } else { "app-button-secondary w-full px-4 py-2 text-left text-sm" },
                                title: "Open note page {note.title()}",
                                onclick: move |_| {
                                    selected_note_id.set(Some(note.id()));
                                    title_input.set(note.title().to_string());
                                    body_input.set(note.body().to_string());
                                },
                                "{note.title()}"
                            }
                        }
                    }
                }

                div { class: "flex min-h-[24rem] flex-1 flex-col gap-3",
                    if selected_note.is_some() {
                        input {
                            class: "app-input",
                            placeholder: "Page title",
                            value: "{title_input}",
                            oninput: move |e| title_input.set(e.value()),
                        }
                        textarea {
                            class: "app-input min-h-[18rem]",
                            placeholder: "Write your notes here...",
                            value: "{body_input}",
                            oninput: move |e| body_input.set(e.value()),
                        }
                        div { class: "flex flex-wrap justify-end gap-2",
                            button {
                                class: "app-button-secondary px-4 py-2",
                                title: "Save this note page",
                                onclick: move |_| {
                                    let Some(note_id) = selected_note_id() else { return; };
                                    let mut reg = registry.write();
                                    let rename_result = execute(
                                        Command::RenameNotePage { card_id, note_page_id: note_id, title: title_input().trim().to_string() },
                                        &mut reg,
                                    );
                                    let save_result = execute(
                                        Command::SaveNotePageBody { card_id, note_page_id: note_id, body: body_input() },
                                        &mut reg,
                                    );
                                    if let Some(error_value) = rename_result.err().or_else(|| save_result.err()) {
                                        error_message.set(Some(user_message_for_command_error(&error_value)));
                                    } else {
                                        error_message.set(None);
                                    }
                                },
                                "Save Page"
                            }
                            button {
                                class: "app-danger-button px-4 py-2",
                                title: "Delete this note page",
                                onclick: move |_| {
                                    let Some(note_id) = selected_note_id() else { return; };
                                    let mut reg = registry.write();
                                    match execute(Command::DeleteNotePage { card_id, note_page_id: note_id }, &mut reg) {
                                        Ok(()) => {
                                            let fallback = reg.get_card(card_id).ok().and_then(|card| card.notes().first().cloned());
                                            selected_note_id.set(fallback.as_ref().map(|note| note.id()));
                                            title_input.set(fallback.as_ref().map(|note| note.title().to_string()).unwrap_or_default());
                                            body_input.set(fallback.as_ref().map(|note| note.body().to_string()).unwrap_or_default());
                                            error_message.set(None);
                                        }
                                        Err(error_value) => error_message.set(Some(user_message_for_command_error(&error_value))),
                                    }
                                },
                                "Delete Page"
                            }
                        }
                    } else {
                        div { class: "app-empty-state flex min-h-[18rem] items-center justify-center rounded-[1.5rem] p-6 text-center",
                            p { class: "app-text-muted", "No note pages yet. Add one to start writing." }
                        }
                    }
                }
                if let Some(message) = error_message() { {inline_error(message)} }
            }
        }
    }
}
