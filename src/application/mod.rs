use crate::domain::bucket::Bucket;
use crate::domain::card::Card;
use crate::domain::due_date::DueDate;
use crate::domain::error::DomainError;
use crate::domain::id::{BucketId, CardId, LabelId, NotePageId, RuleId};
use crate::domain::label::LabelColor;
use crate::domain::registry::{CardRegistry, DeleteStrategy};
use crate::domain::rule::{RuleAction, RuleDefinition, RuleTrigger};
use crate::infrastructure::logging::record_diagnostic;
use std::collections::HashMap;
use tracing::{Level, error, info};

/// Application layer commands that can be executed against the domain.
///
/// # Examples
///
/// ```rust
/// use kanban_planner::application::Command;
///
/// let command = Command::CreateRootCard {
///     title: "Roadmap".to_string(),
/// };
/// assert!(matches!(command, Command::CreateRootCard { .. }));
/// ```
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
    CreateLabelDefinition {
        name: String,
        color: LabelColor,
    },
    DeleteLabelDefinition {
        label_id: LabelId,
    },
    SetCardLabels {
        card_id: CardId,
        label_ids: Vec<LabelId>,
    },
    CreateRuleDefinition {
        name: String,
        trigger: RuleTrigger,
        action: RuleAction,
    },
    DeleteRuleDefinition {
        rule_id: RuleId,
    },
    SetCardRules {
        card_id: CardId,
        rule_ids: Vec<RuleId>,
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
    ReorderRootCards {
        ordered_ids: Vec<CardId>,
    },
    /// Drops a card into a specific bucket at a given index on its parent board.
    DropCardAtPosition {
        board_id: CardId,
        card_id: CardId,
        target_bucket_id: BucketId,
        target_index: usize,
    },
}

/// Executes a command against the card registry.
///
/// The application layer owns command start, success, and failure logging.
/// Callers should surface user-facing state as needed, but should not duplicate
/// command failure logs around this function.
///
/// # Examples
///
/// ```rust
/// use kanban_planner::application::{Command, execute};
/// use kanban_planner::domain::registry::CardRegistry;
///
/// let mut registry = CardRegistry::new();
/// execute(
///     Command::CreateRootCard {
///         title: "Workspace".to_string(),
///     },
///     &mut registry,
/// )?;
/// assert_eq!(registry.get_root_cards().len(), 1);
/// # Ok::<(), kanban_planner::domain::error::DomainError>(())
/// ```
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
        Command::CreateLabelDefinition { name, color } => {
            registry.create_label_definition(name, color)?;
            Ok(())
        }
        Command::DeleteLabelDefinition { label_id } => registry.delete_label_definition(label_id),
        Command::SetCardLabels { card_id, label_ids } => {
            registry.set_card_labels(card_id, label_ids)
        }
        Command::CreateRuleDefinition {
            name,
            trigger,
            action,
        } => {
            registry.create_rule_definition(name, trigger, action)?;
            Ok(())
        }
        Command::DeleteRuleDefinition { rule_id } => registry.delete_rule_definition(rule_id),
        Command::SetCardRules { card_id, rule_ids } => registry.set_card_rules(card_id, rule_ids),
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
        Command::ReorderRootCards { ordered_ids } => registry.reorder_root_cards(ordered_ids),
        Command::DropCardAtPosition {
            board_id,
            card_id,
            target_bucket_id,
            target_index,
        } => apply_card_drop_internal(registry, board_id, card_id, target_bucket_id, target_index),
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
///
/// # Examples
///
/// ```ignore
/// // BoardView is returned by build_board_view(...) and then consumed by the UI.
/// ```
pub struct BoardView<'a> {
    pub card: &'a Card,
    pub columns: Vec<ColumnView<'a>>,
}

/// A single column in a [`BoardView`].
///
/// # Examples
///
/// ```ignore
/// // ColumnView is produced as part of a BoardView projection.
/// ```
pub struct ColumnView<'a> {
    pub bucket: &'a Bucket,
    pub cards: Vec<&'a Card>,
}

/// A single bucket group in an inline card preview.
#[derive(Debug)]
pub struct CardPreviewSection<'a> {
    pub bucket: &'a Bucket,
    pub cards: Vec<&'a Card>,
}

/// A read-only preview of a card's immediate children grouped by bucket.
#[derive(Debug)]
pub struct CardPreviewView<'a> {
    pub card: &'a Card,
    pub sections: Vec<CardPreviewSection<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopupNotification {
    pub title: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggeredRuleOutcome {
    pub rule: RuleDefinition,
    pub popup: PopupNotification,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardRuleEvent {
    NoteOpened,
    NoteClosed,
    MovedToBucket(BucketId),
}

/// Builds a [`BoardView`] for a given card.
///
/// If the card has an "Unassigned" bucket that is empty, it is omitted from the view
/// to reduce visual clutter on highly organized boards.
///
/// # Examples
///
/// ```rust
/// use kanban_planner::application::build_board_view;
/// use kanban_planner::domain::registry::CardRegistry;
///
/// let mut registry = CardRegistry::new();
/// let board_id = registry.create_root_card("Workspace".to_string())?;
/// let view = build_board_view(board_id, &registry)?;
///
/// assert_eq!(view.card.id(), board_id);
/// # Ok::<(), kanban_planner::domain::error::DomainError>(())
/// ```
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

/// Builds an inline preview of a card's immediate children grouped by bucket.
///
/// Empty buckets are omitted from the preview to keep the UI compact.
pub fn build_card_preview_view(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<CardPreviewView, DomainError> {
    info!(%card_id, "Building card preview view");
    let card = registry.get_card(card_id)?;
    let projection = registry.board_projection(card_id)?;

    let mut sections = Vec::new();
    for bucket in card.buckets() {
        let cards = projection.get(&bucket.id()).cloned().unwrap_or_default();
        if cards.is_empty() {
            continue;
        }

        sections.push(CardPreviewSection { bucket, cards });
    }

    Ok(CardPreviewView { card, sections })
}

pub fn evaluate_card_rules(
    card_id: CardId,
    event: CardRuleEvent,
    registry: &CardRegistry,
) -> Result<Vec<TriggeredRuleOutcome>, DomainError> {
    let card = registry.get_card(card_id)?;
    let assigned_rule_ids: std::collections::HashSet<RuleId> =
        card.rule_ids().iter().copied().collect();

    let mut matches = Vec::new();
    for rule in registry.rule_definitions() {
        if !assigned_rule_ids.contains(&rule.id()) {
            continue;
        }

        if !rule_matches_event(rule.trigger(), &event) {
            continue;
        }

        let popup = match rule.action() {
            RuleAction::ShowPopup { title, message } => PopupNotification {
                title: title.clone(),
                message: message.clone(),
            },
        };

        matches.push(TriggeredRuleOutcome {
            rule: rule.clone(),
            popup,
        });
    }

    Ok(matches)
}

fn rule_matches_event(trigger: &RuleTrigger, event: &CardRuleEvent) -> bool {
    match (trigger, event) {
        (RuleTrigger::NoteOpened, CardRuleEvent::NoteOpened) => true,
        (RuleTrigger::NoteClosed, CardRuleEvent::NoteClosed) => true,
        (RuleTrigger::MovedToBucket(expected), CardRuleEvent::MovedToBucket(actual)) => {
            expected == actual
        }
        _ => false,
    }
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
        Command::CreateLabelDefinition { .. } => {
            info!(
                command = "CreateLabelDefinition",
                "Executing application command"
            );
        }
        Command::DeleteLabelDefinition { label_id } => {
            info!(command = "DeleteLabelDefinition", %label_id, "Executing application command");
        }
        Command::SetCardLabels { card_id, label_ids } => {
            info!(command = "SetCardLabels", %card_id, label_count = label_ids.len(), "Executing application command");
        }
        Command::CreateRuleDefinition { .. } => {
            info!(
                command = "CreateRuleDefinition",
                "Executing application command"
            );
        }
        Command::DeleteRuleDefinition { rule_id } => {
            info!(command = "DeleteRuleDefinition", %rule_id, "Executing application command");
        }
        Command::SetCardRules { card_id, rule_ids } => {
            info!(command = "SetCardRules", %card_id, rule_count = rule_ids.len(), "Executing application command");
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
        Command::ReorderRootCards { ordered_ids } => {
            info!(
                command = "ReorderRootCards",
                root_count = ordered_ids.len(),
                "Executing application command"
            );
        }
        Command::DropCardAtPosition { .. } => {
            info!(
                command = "DropCardAtPosition",
                "Executing application command"
            );
        }
    }
}

fn apply_card_drop_internal(
    registry: &mut CardRegistry,
    board_id: CardId,
    card_id: CardId,
    target_bucket_id: BucketId,
    target_index: usize,
) -> Result<(), DomainError> {
    let board = registry.get_card(board_id)?;
    let bucket_order: Vec<BucketId> = board.buckets().iter().map(|bucket| bucket.id()).collect();
    let child_order = board.children_ids().to_vec();

    let mut cards_by_bucket: HashMap<BucketId, Vec<CardId>> = bucket_order
        .iter()
        .copied()
        .map(|bucket_id| (bucket_id, Vec::new()))
        .collect();
    let mut current_bucket_id = None;

    for child_id in child_order {
        let child = registry.get_card(child_id)?;
        let bucket_id = child.bucket_id().ok_or_else(|| {
            DomainError::InvalidOperation(format!(
                "Child card {child_id} is missing its bucket assignment"
            ))
        })?;

        if child_id == card_id {
            current_bucket_id = Some(bucket_id);
            continue;
        }

        cards_by_bucket.entry(bucket_id).or_default().push(child_id);
    }

    let _current_bucket_id = current_bucket_id.ok_or(DomainError::CardNotFound(card_id))?;
    let target_cards = cards_by_bucket
        .get_mut(&target_bucket_id)
        .ok_or(DomainError::BucketNotFound(target_bucket_id))?;
    let insertion_index = target_index.min(target_cards.len());
    target_cards.insert(insertion_index, card_id);

    // If bucket changed, update it first
    if current_bucket_id != Some(target_bucket_id) {
        registry.move_card_to_bucket(card_id, target_bucket_id)?;
    }

    let mut reordered_children = Vec::new();
    for bucket_id in bucket_order {
        if let Some(cards) = cards_by_bucket.get(&bucket_id) {
            reordered_children.extend(cards.iter().copied());
        }
    }

    registry.reorder_children(board_id, reordered_children)
}

fn command_name(command: &Command) -> &'static str {
    match command {
        Command::CreateRootCard { .. } => "CreateRootCard",
        Command::CreateChildCard { .. } => "CreateChildCard",
        Command::RenameCard { .. } => "RenameCard",
        Command::AddNotePage { .. } => "AddNotePage",
        Command::RenameNotePage { .. } => "RenameNotePage",
        Command::SaveNotePageBody { .. } => "SaveNotePageBody",
        Command::DeleteNotePage { .. } => "DeleteNotePage",
        Command::SetDueDate { .. } => "SetDueDate",
        Command::ClearDueDate { .. } => "ClearDueDate",
        Command::CreateLabelDefinition { .. } => "CreateLabelDefinition",
        Command::DeleteLabelDefinition { .. } => "DeleteLabelDefinition",
        Command::SetCardLabels { .. } => "SetCardLabels",
        Command::CreateRuleDefinition { .. } => "CreateRuleDefinition",
        Command::DeleteRuleDefinition { .. } => "DeleteRuleDefinition",
        Command::SetCardRules { .. } => "SetCardRules",
        Command::DeleteCard { .. } => "DeleteCard",
        Command::MoveCardToBucket { .. } => "MoveCardToBucket",
        Command::ReparentCard { .. } => "ReparentCard",
        Command::AddBucket { .. } => "AddBucket",
        Command::RenameBucket { .. } => "RenameBucket",
        Command::RemoveBucket { .. } => "RemoveBucket",
        Command::ReorderBuckets { .. } => "ReorderBuckets",
        Command::ReorderChildren { .. } => "ReorderChildren",
        Command::ReorderRootCards { .. } => "ReorderRootCards",
        Command::DropCardAtPosition { .. } => "DropCardAtPosition",
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

    #[test]
    fn test_card_preview_groups_children_by_bucket() {
        let mut reg = CardRegistry::new();
        let root_id = reg.create_root_card("Root".into()).unwrap();
        let extra_bucket_id = reg.add_bucket(root_id, "Doing".into()).unwrap();
        let third_bucket_id = reg.add_bucket(root_id, "Done".into()).unwrap();
        let unassigned_id = reg.get_card(root_id).unwrap().buckets()[0].id();

        reg.create_child_card("Alpha".into(), root_id, extra_bucket_id)
            .unwrap();
        reg.create_child_card("Beta".into(), root_id, unassigned_id)
            .unwrap();

        let preview = build_card_preview_view(root_id, &reg).unwrap();

        assert_eq!(preview.card.id(), root_id);
        assert_eq!(preview.sections.len(), 2);
        assert_eq!(preview.sections[0].bucket.id(), unassigned_id);
        assert_eq!(preview.sections[0].cards[0].title(), "Beta");
        assert_eq!(preview.sections[1].bucket.id(), extra_bucket_id);
        assert_eq!(preview.sections[1].cards[0].title(), "Alpha");
        assert!(
            preview
                .sections
                .iter()
                .all(|section| section.bucket.id() != third_bucket_id),
            "Empty buckets should be omitted from card previews"
        );
    }

    #[test]
    fn evaluate_card_rules_matches_bucket_trigger() {
        let mut reg = CardRegistry::new();
        let card_id = reg.create_root_card("Root".into()).unwrap();
        let bucket_id = reg.get_card(card_id).unwrap().buckets()[0].id();
        let rule_id = reg
            .create_rule_definition(
                "Bucket popup".into(),
                RuleTrigger::MovedToBucket(bucket_id),
                RuleAction::ShowPopup {
                    title: "Moved".into(),
                    message: "Card changed bucket".into(),
                },
            )
            .unwrap();
        reg.set_card_rules(card_id, vec![rule_id]).unwrap();

        let matches =
            evaluate_card_rules(card_id, CardRuleEvent::MovedToBucket(bucket_id), &reg).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].popup.title, "Moved");
    }

    #[test]
    fn test_execute_drop_card_at_position() {
        let mut reg = CardRegistry::new();
        let board_id = reg.create_root_card("Board".into()).unwrap();
        let b1_id = reg.get_card(board_id).unwrap().buckets()[0].id(); // Unassigned
        let b2_id = reg.add_bucket(board_id, "Bucket 2".into()).unwrap();

        let c1 = reg.create_child_card("C1".into(), board_id, b1_id).unwrap();
        let c2 = reg.create_child_card("C2".into(), board_id, b1_id).unwrap();
        let c3 = reg.create_child_card("C3".into(), board_id, b2_id).unwrap();

        // Initial order: C1, C2 (b1), C3 (b2)
        assert_eq!(
            reg.get_card(board_id).unwrap().children_ids(),
            &[c1, c2, c3]
        );

        // Drop C1 into b2 at index 0 (before C3)
        execute(
            Command::DropCardAtPosition {
                board_id,
                card_id: c1,
                target_bucket_id: b2_id,
                target_index: 0,
            },
            &mut reg,
        )
        .unwrap();

        // New order should be: C2 (b1), C1 (b2), C3 (b2)
        // Wait, the logic preserves bucket order from the board.
        // Board has [b1, b2].
        // b1 has [C2].
        // b2 has [C1, C3].
        // So global order should be [C2, C1, C3].
        assert_eq!(
            reg.get_card(board_id).unwrap().children_ids(),
            &[c2, c1, c3]
        );
        assert_eq!(reg.get_card(c1).unwrap().bucket_id(), Some(b2_id));
    }
}
