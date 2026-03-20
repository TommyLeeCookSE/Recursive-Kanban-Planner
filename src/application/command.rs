use crate::domain::due_date::DueDate;
use crate::domain::error::DomainError;
use crate::domain::id::{CardId, NotePageId};
use crate::domain::registry::{CardRegistry, DeleteStrategy};
use tracing::info;

#[derive(Clone, Copy)]
struct CommandMetadata {
    name: &'static str,
}

#[derive(Debug)]
pub enum Command {
    CreateWorkspaceChildCard {
        title: String,
    },
    CreateChildCard {
        title: String,
        parent_id: CardId,
    },
    RenameCard {
        id: CardId,
        title: String,
    },
    AddNotePage {
        card_id: CardId,
        title: String,
    },
    RenameNotePage {
        card_id: CardId,
        note_page_id: NotePageId,
        title: String,
    },
    SaveNotePageBody {
        card_id: CardId,
        note_page_id: NotePageId,
        body: String,
    },
    DeleteNotePage {
        card_id: CardId,
        note_page_id: NotePageId,
    },
    SetDueDate {
        card_id: CardId,
        due_date: DueDate,
    },
    ClearDueDate {
        card_id: CardId,
    },
    DeleteCard {
        id: CardId,
        strategy: DeleteStrategy,
    },
    ReparentCard {
        card_id: CardId,
        new_parent_id: CardId,
    },
    ReorderChildren {
        parent_id: CardId,
        ordered_ids: Vec<CardId>,
    },
    DropChildAtPosition {
        parent_id: CardId,
        card_id: CardId,
        target_index: usize,
    },
}

impl Command {
    fn metadata(&self) -> CommandMetadata {
        let name = match self {
            Command::CreateWorkspaceChildCard { .. } => "CreateWorkspaceChildCard",
            Command::CreateChildCard { .. } => "CreateChildCard",
            Command::RenameCard { .. } => "RenameCard",
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
        };

        CommandMetadata { name }
    }

    pub fn name(&self) -> &'static str {
        self.metadata().name
    }

    pub fn log_start(&self) {
        let metadata = self.metadata();
        info!(
            command = metadata.name,
            details = ?self,
            "Executing application command"
        );
    }

    pub fn apply(self, registry: &mut CardRegistry) -> Result<(), DomainError> {
        match self {
            Command::CreateWorkspaceChildCard { title } => {
                registry.create_workspace_child_card(title)?;
                Ok(())
            }
            Command::CreateChildCard { title, parent_id } => {
                registry.create_child_card(title, parent_id)?;
                Ok(())
            }
            Command::RenameCard { id, title } => registry.rename_card(id, title),
            Command::AddNotePage { card_id, title } => {
                registry.add_note_page(card_id, title)?;
                Ok(())
            }
            Command::RenameNotePage {
                card_id,
                note_page_id,
                title,
            } => registry.rename_note_page(card_id, note_page_id, title),
            Command::SaveNotePageBody {
                card_id,
                note_page_id,
                body,
            } => registry.save_note_page_body(card_id, note_page_id, body),
            Command::DeleteNotePage {
                card_id,
                note_page_id,
            } => registry.delete_note_page(card_id, note_page_id),
            Command::SetDueDate { card_id, due_date } => registry.set_due_date(card_id, due_date),
            Command::ClearDueDate { card_id } => registry.clear_due_date(card_id),
            Command::DeleteCard { id, strategy } => registry.delete_card(id, strategy),
            Command::ReparentCard {
                card_id,
                new_parent_id,
            } => registry.reparent_card(card_id, new_parent_id),
            Command::ReorderChildren {
                parent_id,
                ordered_ids,
            } => registry.reorder_children(parent_id, ordered_ids),
            Command::DropChildAtPosition {
                parent_id,
                card_id,
                target_index,
            } => registry.drop_child_at_position(parent_id, card_id, target_index),
        }
    }
}
