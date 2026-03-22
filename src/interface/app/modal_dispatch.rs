use crate::domain::registry::CardRegistry;
use crate::interface::components::modal::{
    CardModal, EditCardModal, ModalType, NotesModal, SearchModal,
};
use dioxus::prelude::*;

pub(super) fn render_modal_overlay(
    modal: ModalType,
    mut active_modal: Signal<Option<ModalType>>,
    registry: Signal<CardRegistry>,
) -> Element {
    match modal {
        ModalType::CreateCard { parent_id } => rsx! {
            CardModal {
                on_close: move |_| active_modal.set(None),
                parent_id,
                registry,
            }
        },
        ModalType::EditCard { id } => rsx! {
            EditCardModal {
                on_close: move |_| active_modal.set(None),
                id,
                registry,
            }
        },
        ModalType::CardNotes { card_id } => rsx! {
            NotesModal {
                on_close: move |_| active_modal.set(None),
                card_id,
                registry,
            }
        },
        ModalType::Search => rsx! {
            SearchModal {
                on_close: move |_| active_modal.set(None),
                registry,
            }
        },
    }
}
