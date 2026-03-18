use crate::domain::bucket::Bucket;
use crate::domain::card::Card;
use crate::domain::error::DomainError;
use crate::domain::id::{BucketId, CardId};
use crate::domain::registry::{CardRegistry, DeleteStrategy};
use crate::infrastructure::logging::record_diagnostic;
use tracing::{Level, error, info};

/// Application layer commands that can be executed against the domain.
#[derive(Debug)]
pub enum Command {
    CreateRootCard {
        title: String,
    },
    CreateChildCard {
        title: String,
        parent_id: CardId,
        bucket_id: BucketId,
    },
    RenameCard {
        id: CardId,
        title: String,
    },
    DeleteCard {
        id: CardId,
        strategy: DeleteStrategy,
    },
    MoveCardToBucket {
        card_id: CardId,
        bucket_id: BucketId,
    },
    ReparentCard {
        card_id: CardId,
        new_parent_id: CardId,
    },
    AddBucket {
        card_id: CardId,
        name: String,
    },
    RenameBucket {
        card_id: CardId,
        bucket_id: BucketId,
        new_name: String,
    },
    RemoveBucket {
        card_id: CardId,
        bucket_id: BucketId,
    },
    ReorderBuckets {
        card_id: CardId,
        ordered_ids: Vec<BucketId>,
    },
    ReorderChildren {
        card_id: CardId,
        ordered_ids: Vec<CardId>,
    },
}

/// Executes a command against the card registry.
pub fn execute(command: Command, registry: &mut CardRegistry) -> Result<(), DomainError> {
    log_command_start(&command);
    let command_label = command_name(&command);

    let result = match command {
        Command::CreateRootCard { title } => {
            registry.create_root_card(title)?;
            Ok(())
        }
        Command::CreateChildCard {
            title,
            parent_id,
            bucket_id,
        } => {
            registry.create_child_card(title, parent_id, bucket_id)?;
            Ok(())
        }
        Command::RenameCard { id, title } => registry.rename_card(id, title),
        Command::DeleteCard { id, strategy } => registry.delete_card(id, strategy),
        Command::MoveCardToBucket { card_id, bucket_id } => {
            registry.move_card_to_bucket(card_id, bucket_id)
        }
        Command::ReparentCard {
            card_id,
            new_parent_id,
        } => registry.reparent_card(card_id, new_parent_id),
        Command::AddBucket { card_id, name } => {
            registry.add_bucket(card_id, name)?;
            Ok(())
        }
        Command::RenameBucket {
            card_id,
            bucket_id,
            new_name,
        } => registry.rename_bucket(card_id, bucket_id, new_name),
        Command::RemoveBucket { card_id, bucket_id } => registry.remove_bucket(card_id, bucket_id),
        Command::ReorderBuckets {
            card_id,
            ordered_ids,
        } => registry.reorder_buckets(card_id, ordered_ids),
        Command::ReorderChildren {
            card_id,
            ordered_ids,
        } => registry.reorder_children(card_id, ordered_ids),
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

/// A read-only projection of a single card's board, used for UI rendering.
pub struct BoardView<'a> {
    pub card: &'a Card,
    pub columns: Vec<ColumnView<'a>>,
}

/// A single column in a [`BoardView`].
pub struct ColumnView<'a> {
    pub bucket: &'a Bucket,
    pub cards: Vec<&'a Card>,
}

/// Builds a [`BoardView`] for a given card.
///
/// If the card has an "Unassigned" bucket that is empty, it is omitted from the view
/// to reduce visual clutter on highly organized boards.
pub fn build_board_view(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<BoardView, DomainError> {
    info!(%card_id, "Building board view");
    let card = registry.get_card(card_id)?;
    let projection = registry.board_projection(card_id)?;

    let mut columns = Vec::new();
    let bucket_count = card.buckets().len();

    for bucket in card.buckets() {
        let cards = projection.get(&bucket.id()).cloned().unwrap_or_default();

        // Skip "Unassigned" column if it's empty AND not the only column
        if bucket.name() == crate::domain::card::UNASSIGNED_BUCKET_NAME
            && cards.is_empty()
            && bucket_count > 1
        {
            continue;
        }

        columns.push(ColumnView { bucket, cards });
    }

    Ok(BoardView { card, columns })
}

fn log_command_start(command: &Command) {
    match command {
        Command::CreateRootCard { .. } => {
            info!(command = "CreateRootCard", "Executing application command");
        }
        Command::CreateChildCard {
            parent_id,
            bucket_id,
            ..
        } => {
            info!(
                command = "CreateChildCard",
                %parent_id,
                %bucket_id,
                "Executing application command"
            );
        }
        Command::RenameCard { id, .. } => {
            info!(command = "RenameCard", card_id = %id, "Executing application command");
        }
        Command::DeleteCard { id, strategy } => {
            info!(
                command = "DeleteCard",
                card_id = %id,
                strategy = ?strategy,
                "Executing application command"
            );
        }
        Command::MoveCardToBucket { card_id, bucket_id } => {
            info!(
                command = "MoveCardToBucket",
                %card_id,
                %bucket_id,
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
        Command::AddBucket { card_id, .. } => {
            info!(command = "AddBucket", %card_id, "Executing application command");
        }
        Command::RenameBucket {
            card_id, bucket_id, ..
        } => {
            info!(
                command = "RenameBucket",
                %card_id,
                %bucket_id,
                "Executing application command"
            );
        }
        Command::RemoveBucket { card_id, bucket_id } => {
            info!(
                command = "RemoveBucket",
                %card_id,
                %bucket_id,
                "Executing application command"
            );
        }
        Command::ReorderBuckets {
            card_id,
            ordered_ids,
        } => {
            info!(
                command = "ReorderBuckets",
                %card_id,
                bucket_count = ordered_ids.len(),
                "Executing application command"
            );
        }
        Command::ReorderChildren {
            card_id,
            ordered_ids,
        } => {
            info!(
                command = "ReorderChildren",
                %card_id,
                child_count = ordered_ids.len(),
                "Executing application command"
            );
        }
    }
}

fn command_name(command: &Command) -> &'static str {
    match command {
        Command::CreateRootCard { .. } => "CreateRootCard",
        Command::CreateChildCard { .. } => "CreateChildCard",
        Command::RenameCard { .. } => "RenameCard",
        Command::DeleteCard { .. } => "DeleteCard",
        Command::MoveCardToBucket { .. } => "MoveCardToBucket",
        Command::ReparentCard { .. } => "ReparentCard",
        Command::AddBucket { .. } => "AddBucket",
        Command::RenameBucket { .. } => "RenameBucket",
        Command::RemoveBucket { .. } => "RemoveBucket",
        Command::ReorderBuckets { .. } => "ReorderBuckets",
        Command::ReorderChildren { .. } => "ReorderChildren",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::card::UNASSIGNED_BUCKET_NAME;

    #[test]
    fn test_execute_create_root() {
        let mut reg = CardRegistry::new();
        execute(
            Command::CreateRootCard {
                title: "Root".into(),
            },
            &mut reg,
        )
        .unwrap();
        assert_eq!(reg.get_root_cards().len(), 1);
    }

    #[test]
    fn test_board_view_shows_unassigned_if_only_bucket() {
        let mut reg = CardRegistry::new();
        let root_id = reg.create_root_card("Root".into()).unwrap();

        // Root has ONLY Unassigned. It should show up even if empty.
        let view = build_board_view(root_id, &reg).unwrap();
        assert_eq!(view.columns.len(), 1);
        assert_eq!(view.columns[0].bucket.name(), UNASSIGNED_BUCKET_NAME);

        // Now add another bucket. NOW Unassigned should hide (since it's empty).
        reg.add_bucket(root_id, "Todo".into()).unwrap();
        let view = build_board_view(root_id, &reg).unwrap();
        assert_eq!(view.columns.len(), 1);
        assert_eq!(view.columns[0].bucket.name(), "Todo");

        // Add a card to Unassigned. Now both should show.
        let root = reg.get_card(root_id).unwrap();
        let unassigned_id = root.buckets()[0].id(); // Unassigned is at index 0 by default.
        reg.create_child_card("Child".into(), root_id, unassigned_id)
            .unwrap();

        let view = build_board_view(root_id, &reg).unwrap();
        assert_eq!(view.columns.len(), 2);
    }
}
