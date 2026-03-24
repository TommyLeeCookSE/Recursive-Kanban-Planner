use crate::domain::registry::CardRegistry;
use crate::interface::components::modal::{
    CardModal, EditCardModal, Modal, ModalType, NotesModal, SearchModal,
};
use crate::interface::components::shared_forms::due_date_string;
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
        ModalType::EditCard { id } => {
            let reg = registry.read();
            let card_data = reg.get_card(id).ok().map(|card| {
                (
                    card.title().to_string(),
                    card.description().unwrap_or_default().to_string(),
                    due_date_string(card.due_date()),
                )
            });

            if let Some((title, description, due_date)) = card_data {
                rsx! {
                    EditCardModal {
                        on_close: move |_| active_modal.set(None),
                        id,
                        initial_title: title,
                        initial_description: description,
                        initial_due_date: due_date,
                        registry,
                    }
                }
            } else {
                rsx! {
                    Modal {
                        on_close: move |_| active_modal.set(None),
                        title: "Error".to_string(),
                        p { class: "app-error-message", "Card not found" }
                    }
                }
            }
        }
        ModalType::CardNotes { card_id } => rsx! {
            NotesModal {
                on_close: move |_| active_modal.set(None),
                card_id,
                registry,
            }
        },
        ModalType::Search => rsx! {
            SearchModal { on_close: move |_| active_modal.set(None), registry }
        },
    }
}
