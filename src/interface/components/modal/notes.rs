use crate::application::Command;
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::domain::title::MAX_TITLE_LENGTH;
use crate::interface::components::modal::Modal;
use crate::interface::components::shared_forms::{
    inline_error, modal_dispatch_command,
};
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
    let error_message = use_signal(|| None::<String>);

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
        Modal { on_close: move |_| on_close.call(()), title: "Notebook",
            div { class: "app-notes-layout",
                div { class: "app-notes-sidebar",
                    input {
                        class: "app-input",
                        placeholder: "New page title",
                        value: "{new_page_title}",
                        oninput: move |e| new_page_title.set(e.value()),
                        maxlength: MAX_TITLE_LENGTH as i64,
                    }
                    button {
                        class: "app-button-secondary-compact app-button-block",
                        title: "Add a new note page",
                        onclick: move |_| {
                            let title = new_page_title().trim().to_string();
                            modal_dispatch_command(
                                Command::AddNotePage {
                                    card_id,
                                    title,
                                },
                                registry,
                                error_message,
                                move || {
                                    let reg = registry.read();
                                    let latest_note = reg
                                        .get_card(card_id)
                                        .ok()
                                        .and_then(|card| card.notes().last().cloned());
                                    if let Some(note) = latest_note {
                                        selected_note_id.set(Some(note.id()));
                                        title_input.set(note.title().to_string());
                                        body_input.set(note.body().to_string());
                                    }
                                    new_page_title.set("New Page".to_string());
                                },
                            );
                        },
                        "Add Page"
                    }
                    div { class: "app-notes-list",
                        div { class: "app-notes-list-stack",
                            for note in current_notes.iter().cloned() {
                                button {
                                    class: if Some(note.id()) == selected_note_id() { "app-note-list-button--active" } else { "app-note-list-button" },
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
                }

                div { class: "app-notes-editor",
                    if selected_note.is_some() {
                        input {
                            class: "app-input",
                            placeholder: "Page title",
                            value: "{title_input}",
                            oninput: move |e| title_input.set(e.value()),
                            maxlength: MAX_TITLE_LENGTH as i64,
                        }
                        textarea {
                            class: "app-input app-notes-textarea",
                            placeholder: "Write your notes here...",
                            value: "{body_input}",
                            oninput: move |e| body_input.set(e.value()),
                        }
                        div { class: "app-notes-toolbar",
                            button {
                                class: "app-button-secondary-compact",
                                title: "Save this note page",
                                onclick: move |_| {
                                    let Some(note_id) = selected_note_id() else {
                                        return;
                                    };
                                    // Sequence commands: rename then save body
                                    modal_dispatch_command(
                                        Command::RenameNotePage {
                                            card_id,
                                            note_page_id: note_id,
                                            title: title_input().trim().to_string(),
                                        },
                                        registry,
                                        error_message,
                                        move || {
                                            modal_dispatch_command(
                                                Command::SaveNotePageBody {
                                                    card_id,
                                                    note_page_id: note_id,
                                                    body: body_input(),
                                                },
                                                registry,
                                                error_message,
                                                || {},
                                            );
                                        },
                                    );
                                },
                                "Save Page"
                            }
                            button {
                                class: "app-danger-button app-button-compact",
                                title: "Delete this note page",
                                onclick: move |_| {
                                    let Some(note_id) = selected_note_id() else {
                                        return;
                                    };
                                    modal_dispatch_command(
                                        Command::DeleteNotePage {
                                            card_id,
                                            note_page_id: note_id,
                                        },
                                        registry,
                                        error_message,
                                        move || {
                                            let reg = registry.read();
                                            let fallback = reg
                                                .get_card(card_id)
                                                .ok()
                                                .and_then(|card| card.notes().first().cloned());
                                            selected_note_id.set(fallback.as_ref().map(|note| note.id()));
                                            title_input
                                                .set(
                                                    fallback
                                                        .as_ref()
                                                        .map(|note| note.title().to_string())
                                                        .unwrap_or_default(),
                                                );
                                            body_input
                                                .set(
                                                    fallback
                                                        .as_ref()
                                                        .map(|note| note.body().to_string())
                                                        .unwrap_or_default(),
                                                );
                                        },
                                    );
                                },
                                "Delete Page"
                            }
                        }
                    } else {
                        div { class: "app-notes-empty",
                            p { class: "app-empty-message",
                                "No note pages yet. Add one to start writing."
                            }
                        }
                    }
                }
                if let Some(message) = error_message() {
                    {inline_error(message)}
                }
            }
        }
    }
}
