use crate::application::{Command, execute};
use crate::domain::error::DomainError;
use crate::domain::id::{BucketId, CardId};
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use dioxus::prelude::*;
use tracing::{Level, warn};

/// Defines the type of modal to be displayed and the data it requires.
///
/// # Examples
///
/// ```rust
/// use kanban_planner::interface::components::modal::ModalType;
///
/// let modal = ModalType::CreateCard {
///     parent_id: None,
///     bucket_id: None,
/// };
/// assert!(matches!(modal, ModalType::CreateCard { .. }));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum ModalType {
    /// Modal for creating a new card (root or child).
    CreateCard {
        /// The parent card ID, or None for root level.
        parent_id: Option<CardId>,
        /// The bucket ID where the new card will be placed.
        bucket_id: Option<BucketId>,
    },
    /// Modal for renaming an existing card.
    RenameCard {
        /// ID of the card to rename.
        id: CardId,
        /// Current title to pre-populate the field.
        current_title: String,
    },
    /// Modal for adding a new bucket (column) to a board.
    CreateBucket {
        /// ID of the card whose board receives the new bucket.
        card_id: CardId,
    },
}

/// A generic blurred modal wrapper that handles the overlay and center positioning.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     Modal {
///         on_close: move |_| {},
///         title: "New Card".to_string(),
///         div { "Modal body" }
///     }
/// }
/// ```
#[component]
pub fn Modal(on_close: EventHandler<()>, title: String, children: Element) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm animate-in fade-in duration-200",
            onclick: move |_| on_close.call(()),

            div {
                class: "w-full max-w-md overflow-hidden rounded-3xl border border-white/20 bg-white/95 shadow-2xl backdrop-blur-md dark:border-gray-700/70 dark:bg-gray-900/95 animate-in zoom-in-95 duration-200",
                onclick: |e| e.stop_propagation(),

                div { class: "px-6 py-4 flex items-center justify-between border-b border-gray-200 dark:border-gray-800",
                    h2 { class: "text-lg font-bold text-gray-900 dark:text-white", "{title}" }
                    button {
                        class: "p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors",
                        onclick: move |_| on_close.call(()),
                        "X"
                    }
                }

                div { class: "p-6",
                    {children}
                }
            }
        }
    }
}

/// Modal component for creating a new task or board.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     CardModal {
///         on_close: move |_| {},
///         parent_id: None,
///         bucket_id: None,
///         registry,
///     }
/// }
/// ```
#[component]
pub fn CardModal(
    on_close: EventHandler<()>,
    parent_id: Option<CardId>,
    bucket_id: Option<BucketId>,
    registry: Signal<CardRegistry>,
) -> Element {
    let mut input_title = use_signal(String::new);
    let mut error_message = use_signal(|| None::<String>);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: if parent_id.is_some() { "New Child Card" } else { "New Root Board" },
            div { class: "flex flex-col gap-4",
                input {
                    class: "px-4 py-2 border rounded bg-white dark:bg-gray-700 dark:border-gray-600 focus:outline-none focus:ring-2 focus:ring-sunfire text-gray-900 dark:text-gray-100",
                    placeholder: "Enter title...",
                    value: "{input_title}",
                    oninput: move |e| input_title.set(e.value()),
                    autofocus: true,
                }
                if let Some(message) = error_message() {
                    p { class: "rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-900/50 dark:bg-red-950/40 dark:text-red-200",
                        "{message}"
                    }
                }
                div { class: "flex justify-end gap-2",
                    button {
                        class: "px-4 py-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "px-6 py-2 bg-sunfire hover:bg-sunfire-dark text-white font-bold rounded shadow transition-all disabled:opacity-50",
                        disabled: input_title().trim().is_empty(),
                        onclick: move |_| {
                            let trimmed_title = input_title().trim().to_string();
                            let command = match build_create_card_command(trimmed_title, parent_id, bucket_id) {
                                Ok(command) => command,
                                Err(error_value) => {
                                    let user_message = user_message_for_command_error(&error_value);
                                    warn!(
                                        parent_id = ?parent_id,
                                        bucket_id = ?bucket_id,
                                        error = %error_value,
                                        "Card modal rejected invalid create-card context"
                                    );
                                    error_message.set(Some(user_message.clone()));
                                    record_diagnostic(Level::WARN, "ui-modal", user_message);
                                    return;
                                }
                            };

                            let mut reg = registry.write();
                            match execute(command, &mut reg) {
                                Ok(()) => {
                                    error_message.set(None);
                                    on_close.call(());
                                }
                                Err(error_value) => {
                                    error_message.set(Some(user_message_for_command_error(&error_value)));
                                }
                            }
                        },
                        "Create Item"
                    }
                }
            }
        }
    }
}

/// Modal component for changing the title of a card.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     RenameCardModal {
///         on_close: move |_| {},
///         id: card_id,
///         current_title: "Current title".to_string(),
///         registry,
///     }
/// }
/// ```
#[component]
pub fn RenameCardModal(
    on_close: EventHandler<()>,
    id: CardId,
    current_title: String,
    registry: Signal<CardRegistry>,
) -> Element {
    let mut input_title = use_signal(move || current_title.clone());
    let mut error_message = use_signal(|| None::<String>);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "Rename Item",
            div { class: "flex flex-col gap-4",
                input {
                    class: "px-4 py-2 border rounded bg-white dark:bg-gray-700 dark:border-gray-600 focus:outline-none focus:ring-2 focus:ring-sunfire text-gray-900 dark:text-gray-100",
                    placeholder: "Enter a new title...",
                    value: "{input_title}",
                    oninput: move |e| input_title.set(e.value()),
                    autofocus: true,
                }
                if let Some(message) = error_message() {
                    p { class: "rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-900/50 dark:bg-red-950/40 dark:text-red-200",
                        "{message}"
                    }
                }
                div { class: "flex justify-end gap-2",
                    button {
                        class: "px-4 py-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "px-6 py-2 bg-sunfire hover:bg-sunfire-dark text-white font-bold rounded shadow transition-all disabled:opacity-50",
                        disabled: input_title().trim().is_empty(),
                        onclick: move |_| {
                            let mut reg = registry.write();
                            match execute(
                                Command::RenameCard {
                                    id,
                                    title: input_title().trim().to_string(),
                                },
                                &mut reg,
                            ) {
                                Ok(()) => {
                                    error_message.set(None);
                                    on_close.call(());
                                }
                                Err(error_value) => {
                                    error_message.set(Some(user_message_for_command_error(&error_value)));
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

/// Modal component for adding a new column to a board.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     BucketModal {
///         on_close: move |_| {},
///         card_id: board_id,
///         registry,
///     }
/// }
/// ```
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
                    class: "px-4 py-2 border rounded bg-white dark:bg-gray-700 dark:border-gray-600 focus:outline-none focus:ring-2 focus:ring-sunfire text-gray-900 dark:text-gray-100",
                    placeholder: "Column Name (e.g., Todo, Doing)",
                    value: "{input_name}",
                    oninput: move |e| input_name.set(e.value()),
                    autofocus: true,
                }
                if let Some(message) = error_message() {
                    p { class: "rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-900/50 dark:bg-red-950/40 dark:text-red-200",
                        "{message}"
                    }
                }
                div { class: "flex justify-end gap-2",
                    button {
                        class: "px-4 py-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "px-6 py-2 bg-sunfire hover:bg-sunfire-dark text-white font-bold rounded shadow transition-all disabled:opacity-50",
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
                                    error_message.set(Some(user_message_for_command_error(&error_value)));
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

fn build_create_card_command(
    title: String,
    parent_id: Option<CardId>,
    bucket_id: Option<BucketId>,
) -> Result<Command, DomainError> {
    match parent_id {
        Some(parent_id) => {
            let bucket_id = bucket_id.ok_or_else(|| {
                DomainError::InvalidOperation(
                    "Unable to create a child card because no destination column was selected."
                        .to_string(),
                )
            })?;

            Ok(Command::CreateChildCard {
                title,
                parent_id,
                bucket_id,
            })
        }
        None => Ok(Command::CreateRootCard { title }),
    }
}

fn user_message_for_command_error(error: &DomainError) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::id::{BucketId, CardId};

    #[test]
    fn create_child_card_requires_bucket_id() {
        let parent_id = CardId::default();
        let error = build_create_card_command("Child".to_string(), Some(parent_id), None)
            .expect_err("missing child bucket should be rejected");

        assert!(
            matches!(error, DomainError::InvalidOperation(message) if message.contains("destination column"))
        );
    }

    #[test]
    fn create_child_card_command_preserves_bucket_id() {
        let parent_id = CardId::default();
        let bucket_id = BucketId::default();

        let command =
            build_create_card_command("Child".to_string(), Some(parent_id), Some(bucket_id))
                .expect("valid child command should be created");

        match command {
            Command::CreateChildCard {
                parent_id: actual_parent_id,
                bucket_id: actual_bucket_id,
                ..
            } => {
                assert_eq!(actual_parent_id, parent_id);
                assert_eq!(actual_bucket_id, bucket_id);
            }
            _ => panic!("expected child card command"),
        }
    }
}
