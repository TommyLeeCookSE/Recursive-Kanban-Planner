use crate::domain::id::CardId;
use dioxus::prelude::*;

mod card;
mod notes;

pub use card::{CardModal, EditCardModal};
pub use notes::NotesModal;

#[derive(Clone, Debug, PartialEq)]
pub enum ModalType {
    CreateCard { parent_id: Option<CardId> },
    EditCard { id: CardId },
    CardNotes { card_id: CardId },
}

#[component]
pub fn Modal(on_close: EventHandler<()>, title: String, children: Element) -> Element {
    rsx! {
        div {
            class: "app-modal-backdrop-animated",
            onclick: move |_| on_close.call(()),
            div {
                class: "app-modal-shell-animated",
                onclick: |e| e.stop_propagation(),
                div { class: "app-modal-header",
                    h2 { class: "app-modal-title-text app-modal-title", "{title}" }
                    button {
                        class: "app-modal-close-button",
                        title: "Close dialog",
                        onclick: move |_| on_close.call(()),
                        "X"
                    }
                }
                div { class: "app-modal-content", {children} }
            }
        }
    }
}
