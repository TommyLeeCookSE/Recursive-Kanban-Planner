//! Definitions for all mutation operations in the system.
//!
//! This module uses the `Command` pattern to encapsulate state-changing requests.
//! Each variant of the `Command` enum represents a specific user intent, which
//! can be logged, validated, and applied to the `CardRegistry`.
//!
//! For a comparison of Rust's Enums versus Python's class-based commands or
//! dispatch tables, see `docs/rust-for-python-devs.md`.

use crate::domain::due_date::DueDate;
use crate::domain::error::DomainError;
use crate::domain::id::{CardId, NotePageId};
use crate::domain::registry::{CardRegistry, DeleteStrategy};
use tracing::info;

/// A request to mutate the application state.
///
/// # Examples
///
/// ```
/// use kanban_planner::application::Command;
///
/// let cmd = Command::CreateWorkspaceChildCard {
///     title: "New Board".into(),
///     description: None,
/// };
/// assert_eq!(cmd.name(), "CreateWorkspaceChildCard");
/// ```
#[derive(Debug)]
pub enum Command {
    /// Create a new card under the workspace root.
    CreateWorkspaceChildCard {
        /// The title of the new card.
        title: String,
        /// The optional description of the new card.
        description: Option<String>,
    },
    /// Create a new card under a specific parent.
    CreateChildCard {
        /// The title of the new card.
        title: String,
        /// The optional description of the new card.
        description: Option<String>,
        /// The ID of the parent card.
        parent_id: CardId,
    },
    /// Rename an existing card.
    RenameCard {
        /// The ID of the card to rename.
        id: CardId,
        /// The new title.
        title: String,
    },
    /// Update the description of an existing card.
    SetCardDescription {
        /// The ID of the card.
        id: CardId,
        /// The new description.
        description: Option<String>,
    },
    /// Add a new note page to a card.
    AddNotePage {
        /// The ID of the card.
        card_id: CardId,
        /// The title of the note page.
        title: String,
    },
    /// Rename a note page.
    RenameNotePage {
        /// The ID of the card containing the note.
        card_id: CardId,
        /// The ID of the note page to rename.
        note_page_id: NotePageId,
        /// The new title.
        title: String,
    },
    /// Update the body content of a note page.
    SaveNotePageBody {
        /// The ID of the card containing the note.
        card_id: CardId,
        /// The ID of the note page.
        note_page_id: NotePageId,
        /// The new body content.
        body: String,
    },
    /// Remove a note page from a card.
    DeleteNotePage {
        /// The ID of the card.
        card_id: CardId,
        /// The ID of the note page to delete.
        note_page_id: NotePageId,
    },
    /// Set the due date for a card.
    SetDueDate {
        /// The ID of the card.
        card_id: CardId,
        /// The new due date.
        due_date: DueDate,
    },
    /// Remove the due date from a card.
    ClearDueDate {
        /// The ID of the card.
        card_id: CardId,
    },
    /// Delete a card from the registry.
    DeleteCard {
        /// The ID of the card to delete.
        id: CardId,
        /// The strategy for handling any children of this card.
        strategy: DeleteStrategy,
    },
    /// Move a card to a different parent.
    ReparentCard {
        /// The ID of the card to move.
        card_id: CardId,
        /// The ID of the new parent.
        new_parent_id: CardId,
    },
    /// Explicitly set the order of children for a parent.
    ReorderChildren {
        /// The ID of the parent card.
        parent_id: CardId,
        /// The complete list of child IDs in their new order.
        ordered_ids: Vec<CardId>,
    },
    /// Move a child card to a specific index within its current parent.
    DropChildAtPosition {
        /// The ID of the parent card.
        parent_id: CardId,
        /// The ID of the child card being moved.
        card_id: CardId,
        /// The target index for insertion.
        target_index: usize,
    },
}

impl Command {
    /// Returns the string name of the command variant.
    pub fn name(&self) -> &'static str {
        match self {
            Command::CreateWorkspaceChildCard { .. } => "CreateWorkspaceChildCard",
            Command::CreateChildCard { .. } => "CreateChildCard",
            Command::RenameCard { .. } => "RenameCard",
            Command::SetCardDescription { .. } => "SetCardDescription",
            Command::AddNotePage { .. } => "AddNotePage",
            Command::RenameNotePage { .. } => "RenameNotePage",
            Command::SaveNotePageBody { .. } => "SaveNotePageBody",
            Command::DeleteNotePage { .. } => "DeleteNotePage",
            Command::SetDueDate { .. } => "SetDueDate",
            Command::ClearDueDate { .. } => "ClearDueDate",
            Command::DeleteCard { .. } => "DeleteCard",
            Command::ReparentCard { .. } => "ReparentCard",
            Command::ReorderChildren { .. } => "ReorderChildren",
            Command::DropChildAtPosition { .. } => "DropChildAtPosition",
        }
    }

    /// Logs the start of command execution using `tracing`.
    pub fn log_start(&self) {
        info!(
            command = self.name(),
            details = ?self,
            "Executing application command"
        );
    }

    /// Applies the command mutation to the provided registry.
    pub fn apply(self, registry: &mut CardRegistry) -> Result<(), DomainError> {
        match self {
            Command::CreateWorkspaceChildCard { title, description } => {
                registry.create_workspace_child_card(title, description)?;
            }
            Command::CreateChildCard { title, description, parent_id } => {
                registry.create_child_card(title, description, parent_id)?;
            }
            Command::RenameCard { id, title } => {
                registry.rename_card(id, title)?;
            }
            Command::SetCardDescription { id, description } => {
                registry.set_card_description(id, description)?;
            }
            Command::AddNotePage { card_id, title } => {
                registry.add_note_page(card_id, title)?;
            }
            Command::RenameNotePage {
                card_id,
                note_page_id,
                title,
            } => {
                registry.rename_note_page(card_id, note_page_id, title)?;
            }
            Command::SaveNotePageBody {
                card_id,
                note_page_id,
                body,
            } => {
                registry.save_note_page_body(card_id, note_page_id, body)?;
            }
            Command::DeleteNotePage {
                card_id,
                note_page_id,
            } => {
                registry.delete_note_page(card_id, note_page_id)?;
            }
            Command::SetDueDate { card_id, due_date } => {
                registry.set_due_date(card_id, due_date)?;
            }
            Command::ClearDueDate { card_id } => {
                registry.clear_due_date(card_id)?;
            }
            Command::DeleteCard { id, strategy } => {
                registry.delete_card(id, strategy)?;
            }
            Command::ReparentCard {
                card_id,
                new_parent_id,
            } => {
                registry.reparent_card(card_id, new_parent_id)?;
            }
            Command::ReorderChildren {
                parent_id,
                ordered_ids,
            } => {
                registry.reorder_children(parent_id, ordered_ids)?;
            }
            Command::DropChildAtPosition {
                parent_id,
                card_id,
                target_index,
            } => {
                registry.drop_child_at_position(parent_id, card_id, target_index)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_name_is_correct() {
        let command = Command::DropChildAtPosition {
            parent_id: CardId::default(),
            card_id: CardId::default(),
            target_index: 2,
        };

        assert_eq!(command.name(), "DropChildAtPosition");
    }

    #[test]
    fn command_apply_works() {
        let mut registry = CardRegistry::new();
        let command = Command::CreateWorkspaceChildCard {
            title: "Top Level Board".into(),
            description: None,
        };

        command.apply(&mut registry).unwrap();

        assert_eq!(registry.workspace_child_count(), 1);
    }
}
