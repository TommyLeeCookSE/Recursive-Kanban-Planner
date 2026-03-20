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

#[derive(Clone, Copy)]
struct CommandDescriptor {
    name: &'static str,
    apply: fn(Command, &mut CardRegistry) -> Result<(), DomainError>,
}

/// A request to mutate the application state.
///
/// # Examples
///
/// ```
/// use kanban_planner::application::Command;
///
/// let cmd = Command::CreateWorkspaceChildCard {
///     title: "New Board".into(),
/// };
/// assert_eq!(cmd.name(), "CreateWorkspaceChildCard");
/// ```
#[derive(Debug)]
pub enum Command {
    /// Create a new card under the workspace root.
    CreateWorkspaceChildCard {
        /// The title of the new card.
        title: String,
    },
    /// Create a new card under a specific parent.
    CreateChildCard {
        /// The title of the new card.
        title: String,
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
    fn descriptor(&self) -> CommandDescriptor {
        match self {
            Command::CreateWorkspaceChildCard { .. } => CommandDescriptor {
                name: "CreateWorkspaceChildCard",
                apply: apply_create_workspace_child_card,
            },
            Command::CreateChildCard { .. } => CommandDescriptor {
                name: "CreateChildCard",
                apply: apply_create_child_card,
            },
            Command::RenameCard { .. } => CommandDescriptor {
                name: "RenameCard",
                apply: apply_rename_card,
            },
            Command::AddNotePage { .. } => CommandDescriptor {
                name: "AddNotePage",
                apply: apply_add_note_page,
            },
            Command::RenameNotePage { .. } => CommandDescriptor {
                name: "RenameNotePage",
                apply: apply_rename_note_page,
            },
            Command::SaveNotePageBody { .. } => CommandDescriptor {
                name: "SaveNotePageBody",
                apply: apply_save_note_page_body,
            },
            Command::DeleteNotePage { .. } => CommandDescriptor {
                name: "DeleteNotePage",
                apply: apply_delete_note_page,
            },
            Command::SetDueDate { .. } => CommandDescriptor {
                name: "SetDueDate",
                apply: apply_set_due_date,
            },
            Command::ClearDueDate { .. } => CommandDescriptor {
                name: "ClearDueDate",
                apply: apply_clear_due_date,
            },
            Command::DeleteCard { .. } => CommandDescriptor {
                name: "DeleteCard",
                apply: apply_delete_card,
            },
            Command::ReparentCard { .. } => CommandDescriptor {
                name: "ReparentCard",
                apply: apply_reparent_card,
            },
            Command::ReorderChildren { .. } => CommandDescriptor {
                name: "ReorderChildren",
                apply: apply_reorder_children,
            },
            Command::DropChildAtPosition { .. } => CommandDescriptor {
                name: "DropChildAtPosition",
                apply: apply_drop_child_at_position,
            },
        }
    }

    /// Returns the string name of the command variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::application::Command;
    ///
    /// let cmd = Command::ClearDueDate { card_id: Default::default() };
    /// assert_eq!(cmd.name(), "ClearDueDate");
    /// ```
    pub fn name(&self) -> &'static str {
        self.descriptor().name
    }

    /// Logs the start of command execution using `tracing`.
    pub fn log_start(&self) {
        let descriptor = self.descriptor();
        info!(
            command = descriptor.name,
            details = ?self,
            "Executing application command"
        );
    }

    /// Applies the command mutation to the provided registry.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::application::Command;
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let mut registry = CardRegistry::new();
    /// let cmd = Command::CreateWorkspaceChildCard { title: "Test".into() };
    /// cmd.apply(&mut registry).unwrap();
    /// ```
    pub fn apply(self, registry: &mut CardRegistry) -> Result<(), DomainError> {
        let descriptor = self.descriptor();
        (descriptor.apply)(self, registry)
    }
}

fn apply_create_workspace_child_card(
    command: Command,
    registry: &mut CardRegistry,
) -> Result<(), DomainError> {
    let Command::CreateWorkspaceChildCard { title } = command else {
        unreachable!("descriptor/apply mismatch for CreateWorkspaceChildCard");
    };
    registry.create_workspace_child_card(title)?;
    Ok(())
}

fn apply_create_child_card(
    command: Command,
    registry: &mut CardRegistry,
) -> Result<(), DomainError> {
    let Command::CreateChildCard { title, parent_id } = command else {
        unreachable!("descriptor/apply mismatch for CreateChildCard");
    };
    registry.create_child_card(title, parent_id)?;
    Ok(())
}

fn apply_rename_card(command: Command, registry: &mut CardRegistry) -> Result<(), DomainError> {
    let Command::RenameCard { id, title } = command else {
        unreachable!("descriptor/apply mismatch for RenameCard");
    };
    registry.rename_card(id, title)
}

fn apply_add_note_page(command: Command, registry: &mut CardRegistry) -> Result<(), DomainError> {
    let Command::AddNotePage { card_id, title } = command else {
        unreachable!("descriptor/apply mismatch for AddNotePage");
    };
    registry.add_note_page(card_id, title)?;
    Ok(())
}

fn apply_rename_note_page(
    command: Command,
    registry: &mut CardRegistry,
) -> Result<(), DomainError> {
    let Command::RenameNotePage {
        card_id,
        note_page_id,
        title,
    } = command
    else {
        unreachable!("descriptor/apply mismatch for RenameNotePage");
    };
    registry.rename_note_page(card_id, note_page_id, title)
}

fn apply_save_note_page_body(
    command: Command,
    registry: &mut CardRegistry,
) -> Result<(), DomainError> {
    let Command::SaveNotePageBody {
        card_id,
        note_page_id,
        body,
    } = command
    else {
        unreachable!("descriptor/apply mismatch for SaveNotePageBody");
    };
    registry.save_note_page_body(card_id, note_page_id, body)
}

fn apply_delete_note_page(
    command: Command,
    registry: &mut CardRegistry,
) -> Result<(), DomainError> {
    let Command::DeleteNotePage {
        card_id,
        note_page_id,
    } = command
    else {
        unreachable!("descriptor/apply mismatch for DeleteNotePage");
    };
    registry.delete_note_page(card_id, note_page_id)
}

fn apply_set_due_date(command: Command, registry: &mut CardRegistry) -> Result<(), DomainError> {
    let Command::SetDueDate { card_id, due_date } = command else {
        unreachable!("descriptor/apply mismatch for SetDueDate");
    };
    registry.set_due_date(card_id, due_date)
}

fn apply_clear_due_date(command: Command, registry: &mut CardRegistry) -> Result<(), DomainError> {
    let Command::ClearDueDate { card_id } = command else {
        unreachable!("descriptor/apply mismatch for ClearDueDate");
    };
    registry.clear_due_date(card_id)
}

fn apply_delete_card(command: Command, registry: &mut CardRegistry) -> Result<(), DomainError> {
    let Command::DeleteCard { id, strategy } = command else {
        unreachable!("descriptor/apply mismatch for DeleteCard");
    };
    registry.delete_card(id, strategy)
}

fn apply_reparent_card(command: Command, registry: &mut CardRegistry) -> Result<(), DomainError> {
    let Command::ReparentCard {
        card_id,
        new_parent_id,
    } = command
    else {
        unreachable!("descriptor/apply mismatch for ReparentCard");
    };
    registry.reparent_card(card_id, new_parent_id)
}

fn apply_reorder_children(
    command: Command,
    registry: &mut CardRegistry,
) -> Result<(), DomainError> {
    let Command::ReorderChildren {
        parent_id,
        ordered_ids,
    } = command
    else {
        unreachable!("descriptor/apply mismatch for ReorderChildren");
    };
    registry.reorder_children(parent_id, ordered_ids)
}

fn apply_drop_child_at_position(
    command: Command,
    registry: &mut CardRegistry,
) -> Result<(), DomainError> {
    let Command::DropChildAtPosition {
        parent_id,
        card_id,
        target_index,
    } = command
    else {
        unreachable!("descriptor/apply mismatch for DropChildAtPosition");
    };
    registry.drop_child_at_position(parent_id, card_id, target_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_name_comes_from_descriptor() {
        let command = Command::DropChildAtPosition {
            parent_id: CardId::default(),
            card_id: CardId::default(),
            target_index: 2,
        };

        assert_eq!(command.name(), "DropChildAtPosition");
    }

    #[test]
    fn command_apply_uses_descriptor_function() {
        let mut registry = CardRegistry::new();
        let command = Command::CreateWorkspaceChildCard {
            title: "Top Level Board".into(),
        };

        command.apply(&mut registry).unwrap();

        assert_eq!(registry.workspace_child_count(), 1);
    }
}
