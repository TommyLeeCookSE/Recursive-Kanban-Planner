use crate::domain::card::Card;
use crate::domain::due_date::DueDate;
use crate::domain::error::DomainError;
use crate::domain::id::{CardId, NotePageId};
use crate::domain::registry::{CardRegistry, DeleteStrategy};
use crate::infrastructure::logging::record_diagnostic;
use tracing::{Level, error, info, trace};

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

pub fn execute(command: Command, registry: &mut CardRegistry) -> Result<(), DomainError> {
    log_command_start(&command);
    let command_label = command_name(&command);

    let result = match command {
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
        } => apply_child_drop_internal(registry, parent_id, card_id, target_index),
    };

    match &result {
        Ok(()) => info!(command = command_label, "Application command completed"),
        Err(error_value) => {
            error!(
                command = command_label,
                error = %error_value,
                "Application command failed"
            );
            record_diagnostic(
                Level::ERROR,
                "application",
                format!("Application command '{command_label}' failed: {error_value}"),
            );
        }
    }

    result
}

pub struct BoardView<'a> {
    pub card: &'a Card,
    pub children: Vec<&'a Card>,
}

#[derive(Debug)]
pub struct CardPreviewView<'a> {
    pub card: &'a Card,
    pub children: Vec<&'a Card>,
}

pub fn build_board_view(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<BoardView<'_>, DomainError> {
    trace!(%card_id, "Building board view");
    Ok(BoardView {
        card: registry.get_card(card_id)?,
        children: registry.get_children(card_id)?,
    })
}

pub fn build_card_preview_view(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<CardPreviewView<'_>, DomainError> {
    trace!(%card_id, "Building card preview view");
    Ok(CardPreviewView {
        card: registry.get_card(card_id)?,
        children: registry.get_children(card_id)?,
    })
}

fn log_command_start(command: &Command) {
    match command {
        Command::CreateWorkspaceChildCard { .. } => {
            info!(
                command = "CreateWorkspaceChildCard",
                "Executing application command"
            );
        }
        Command::CreateChildCard { parent_id, .. } => {
            info!(
                command = "CreateChildCard",
                %parent_id,
                "Executing application command"
            );
        }
        Command::RenameCard { id, .. } => {
            info!(command = "RenameCard", card_id = %id, "Executing application command");
        }
        Command::AddNotePage { card_id, .. } => {
            info!(command = "AddNotePage", %card_id, "Executing application command");
        }
        Command::RenameNotePage {
            card_id,
            note_page_id,
            ..
        } => {
            info!(command = "RenameNotePage", %card_id, %note_page_id, "Executing application command");
        }
        Command::SaveNotePageBody {
            card_id,
            note_page_id,
            ..
        } => {
            info!(command = "SaveNotePageBody", %card_id, %note_page_id, "Executing application command");
        }
        Command::DeleteNotePage {
            card_id,
            note_page_id,
        } => {
            info!(command = "DeleteNotePage", %card_id, %note_page_id, "Executing application command");
        }
        Command::SetDueDate { card_id, due_date } => {
            info!(command = "SetDueDate", %card_id, %due_date, "Executing application command");
        }
        Command::ClearDueDate { card_id } => {
            info!(command = "ClearDueDate", %card_id, "Executing application command");
        }
        Command::DeleteCard { id, strategy } => {
            info!(
                command = "DeleteCard",
                card_id = %id,
                strategy = ?strategy,
                "Executing application command"
            );
        }
        Command::ReparentCard {
            card_id,
            new_parent_id,
        } => {
            info!(
                command = "ReparentCard",
                %card_id,
                %new_parent_id,
                "Executing application command"
            );
        }
        Command::ReorderChildren {
            parent_id,
            ordered_ids,
        } => {
            info!(
                command = "ReorderChildren",
                %parent_id,
                child_count = ordered_ids.len(),
                "Executing application command"
            );
        }
        Command::DropChildAtPosition { .. } => {
            info!(
                command = "DropChildAtPosition",
                "Executing application command"
            );
        }
    }
}

fn apply_child_drop_internal(
    registry: &mut CardRegistry,
    parent_id: CardId,
    card_id: CardId,
    target_index: usize,
) -> Result<(), DomainError> {
    let parent = registry.get_card(parent_id)?;
    if !parent.children_ids().contains(&card_id) {
        return Err(DomainError::InvalidOperation(format!(
            "Card {card_id} is not a child of parent {parent_id}"
        )));
    }

    let mut reordered_children: Vec<CardId> = parent
        .children_ids()
        .iter()
        .copied()
        .filter(|child_id| *child_id != card_id)
        .collect();
    let insertion_index = target_index.min(reordered_children.len());
    reordered_children.insert(insertion_index, card_id);

    registry.reorder_children(parent_id, reordered_children)
}

fn command_name(command: &Command) -> &'static str {
    match command {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_create_workspace_child() {
        let mut registry = CardRegistry::new();
        execute(
            Command::CreateWorkspaceChildCard {
                title: "Top Level Board".into(),
            },
            &mut registry,
        )
        .unwrap();

        assert_eq!(registry.workspace_child_count(), 1);
    }

    #[test]
    fn test_board_view_returns_ordered_children() {
        let mut registry = CardRegistry::new();
        let workspace_id = registry.workspace_card_id().unwrap();
        let first_id = registry
            .create_child_card("First".into(), workspace_id)
            .unwrap();
        let second_id = registry
            .create_child_card("Second".into(), workspace_id)
            .unwrap();

        let view = build_board_view(workspace_id, &registry).unwrap();
        let ids: Vec<CardId> = view.children.iter().map(|card| card.id()).collect();
        assert_eq!(ids, vec![first_id, second_id]);
    }

    #[test]
    fn test_card_preview_returns_immediate_children_only() {
        let mut registry = CardRegistry::new();
        let workspace_id = registry.workspace_card_id().unwrap();
        let project_id = registry
            .create_child_card("Project".into(), workspace_id)
            .unwrap();
        let task_id = registry
            .create_child_card("Task".into(), project_id)
            .unwrap();

        let preview = build_card_preview_view(project_id, &registry).unwrap();
        assert_eq!(preview.card.id(), project_id);
        assert_eq!(preview.children.len(), 1);
        assert_eq!(preview.children[0].id(), task_id);
    }

    #[test]
    fn test_execute_drop_child_at_position() {
        let mut registry = CardRegistry::new();
        let workspace_id = registry.workspace_card_id().unwrap();
        let first = registry
            .create_child_card("First".into(), workspace_id)
            .unwrap();
        let second = registry
            .create_child_card("Second".into(), workspace_id)
            .unwrap();
        let third = registry
            .create_child_card("Third".into(), workspace_id)
            .unwrap();

        execute(
            Command::DropChildAtPosition {
                parent_id: workspace_id,
                card_id: third,
                target_index: 0,
            },
            &mut registry,
        )
        .unwrap();

        let ids: Vec<CardId> = registry
            .get_children(workspace_id)
            .unwrap()
            .iter()
            .map(|card| card.id())
            .collect();
        assert_eq!(ids, vec![third, first, second]);
    }
}
