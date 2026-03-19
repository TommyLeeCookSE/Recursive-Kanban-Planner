use crate::domain::id::{BucketId, CardId};
use dioxus::prelude::*;

mod bucket;
mod card;
mod notes;

pub use bucket::{BucketModal, EditBucketModal};
pub use card::{CardModal, EditCardModal};
pub use notes::NotesModal;

#[derive(Clone, Debug, PartialEq)]
pub enum ModalType {
    CreateCard {
        parent_id: Option<CardId>,
        bucket_id: Option<BucketId>,
    },
    EditCard {
        id: CardId,
    },
    CreateBucket {
        card_id: CardId,
    },
    EditBucket {
        card_id: CardId,
        bucket_id: BucketId,
    },
    CardNotes {
        card_id: CardId,
    },
}

#[component]
pub fn Modal(on_close: EventHandler<()>, title: String, children: Element) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4 backdrop-blur-sm animate-in fade-in duration-200",
            onclick: move |_| on_close.call(()),
            div {
                class: "app-modal-surface w-full max-w-md overflow-hidden rounded-[2rem] animate-in zoom-in-95 duration-200",
                onclick: |e| e.stop_propagation(),
                div { class: "flex items-center justify-between border-b px-6 py-4", style: "border-color: var(--app-border);",
                    h2 { class: "app-text-primary text-lg font-bold", "{title}" }
                    button {
                        class: "app-button-ghost p-2",
                        title: "Close dialog",
                        onclick: move |_| on_close.call(()),
                        "X"
                    }
                }
                div { class: "p-6", {children} }
            }
        }
    }
}
