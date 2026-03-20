//! Modal components and types for the user interface.
//!
//! This module provides the base `Modal` component and the `ModalType` enum
//! used to dispatch different modal views across the application.
//!
//! For more on Rust's module system and documentation, see `docs/rust-for-python-devs.md`.

use crate::domain::id::CardId;
use dioxus::prelude::*;

mod card;
mod notes;

pub use card::{CardModal, EditCardModal};
pub use notes::NotesModal;

/// Represents the different types of modals available in the application.
///
/// # Examples
///
/// ```ignore
/// use crate::interface::components::modal::ModalType;
/// use crate::domain::id::CardId;
///
/// let modal = ModalType::CreateCard { parent_id: None };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum ModalType {
    /// Modal for creating a new card, optionally under a parent.
    CreateCard { parent_id: Option<CardId> },
    /// Modal for editing an existing card.
    EditCard { id: CardId },
    /// Modal for viewing and editing notes associated with a card.
    CardNotes { card_id: CardId },
}

/// A generic modal shell component.
///
/// Provides the backdrop, standard header with title and close button,
/// and a content area for children.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     Modal {
///         on_close: move |_| println!("Closed"),
///         title: "My Modal".to_string(),
///         p { "Modal content goes here" }
///     }
/// }
/// ```
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
