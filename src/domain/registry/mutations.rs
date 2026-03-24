//! Registry mutation operations.
//!
//! This module provides the internal implementation for all state changes
//! in the `CardRegistry`. These functions are designed to be called by the
//! `CardRegistry` methods to ensure that domain invariants are maintained.
//!
//! For more on how this project organizes logic into separate modules for
//! mutations, see `docs/rust-for-python-devs.md`.

use super::{CardRegistry, DeleteStrategy, traversal, workspace};
use crate::domain::card::Card;
use crate::domain::due_date::DueDate;
use crate::domain::error::DomainError;
use crate::domain::id::{CardId, NotePageId};

/// Creates a new card as a child of the workspace root.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::domain::registry::CardRegistry;
/// use kanban_planner::domain::registry::mutations::create_workspace_child_card;
///
/// let mut registry = CardRegistry::new();
/// let id = create_workspace_child_card(&mut registry, "Project".into()).unwrap();
/// ```
pub(super) fn create_workspace_child_card(
    registry: &mut CardRegistry,
    title: String,
    description: Option<String>,
) -> Result<CardId, DomainError> {
    let workspace_id = workspace::workspace_card_id(registry)?;
    create_child_card(registry, title, description, workspace_id)
}

/// Creates a new card as a child of the specified parent.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::domain::registry::CardRegistry;
/// use kanban_planner::domain::registry::mutations::create_child_card;
///
/// let mut registry = CardRegistry::new();
/// let workspace_id = registry.workspace_card_id().unwrap();
/// let id = create_child_card(&mut registry, "Project".into(), None, workspace_id).unwrap();
/// ```
pub(super) fn create_child_card(
    registry: &mut CardRegistry,
    title: String,
    description: Option<String>,
    parent_id: CardId,
) -> Result<CardId, DomainError> {
    registry.get_card(parent_id)?;

    let child = Card::new(title, description, Some(parent_id))?;
    let child_id = child.id();

    registry.get_card_mut(parent_id)?.add_child(child_id);
    registry.store.insert(child_id, child);
    Ok(child_id)
}

/// Creates a new card, either in the workspace or as a child.
pub(super) fn create_card(
    registry: &mut CardRegistry,
    title: String,
    description: Option<String>,
    parent_id: Option<CardId>,
) -> Result<CardId, DomainError> {
    match parent_id {
        Some(pid) => create_child_card(registry, title, description, pid),
        None => create_workspace_child_card(registry, title, description),
    }
}

/// Updates various details of an existing card in a single atomic operation.
pub(super) fn update_card_details(
    registry: &mut CardRegistry,
    id: CardId,
    title: Option<String>,
    description: Option<Option<String>>,
    due_date: Option<Option<DueDate>>,
) -> Result<(), DomainError> {
    let card = registry.get_card_mut(id)?;
    if let Some(title) = title {
        card.rename(title)?;
    }
    if let Some(description) = description {
        card.set_description(description)?;
    }
    if let Some(due) = due_date {
        card.set_due_date(due);
    }
    Ok(())
}

/// Reorders the children of a parent card.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::domain::registry::CardRegistry;
/// use kanban_planner::domain::registry::mutations::reorder_children;
///
/// let mut registry = CardRegistry::new();
/// let id = registry.workspace_card_id().unwrap();
/// reorder_children(&mut registry, id, vec![]).unwrap();
/// ```
pub(super) fn reorder_children(
    registry: &mut CardRegistry,
    parent_id: CardId,
    ordered_ids: Vec<CardId>,
) -> Result<(), DomainError> {
    registry
        .get_card_mut(parent_id)?
        .reorder_children(ordered_ids)
}

/// Moves a child card to a specific position in the parent's child list.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::domain::registry::CardRegistry;
/// use kanban_planner::domain::registry::mutations::drop_child_at_position;
///
/// let mut registry = CardRegistry::new();
/// let parent_id = registry.workspace_card_id().unwrap();
/// // ... create child ...
/// // drop_child_at_position(&mut registry, parent_id, child_id, 0).unwrap();
/// ```
pub(super) fn drop_child_at_position(
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

    reorder_children(registry, parent_id, reordered_children)
}

/// Renames an existing card.
pub(super) fn rename_card(
    registry: &mut CardRegistry,
    id: CardId,
    title: String,
) -> Result<(), DomainError> {
    registry.get_card_mut(id)?.rename(title)
}

/// Updates the description of an existing card.
pub(super) fn set_card_description(
    registry: &mut CardRegistry,
    id: CardId,
    description: Option<String>,
) -> Result<(), DomainError> {
    registry.get_card_mut(id)?.set_description(description)
}

/// Adds a new note page to a card.
pub(super) fn add_note_page(
    registry: &mut CardRegistry,
    card_id: CardId,
    title: String,
) -> Result<NotePageId, DomainError> {
    registry.get_card_mut(card_id)?.add_note_page(title)
}

/// Renames a note page on a card.
pub(super) fn rename_note_page(
    registry: &mut CardRegistry,
    card_id: CardId,
    note_page_id: NotePageId,
    title: String,
) -> Result<(), DomainError> {
    registry
        .get_card_mut(card_id)?
        .rename_note_page(note_page_id, title)
}

/// Saves the body content of a note page.
pub(super) fn save_note_page_body(
    registry: &mut CardRegistry,
    card_id: CardId,
    note_page_id: NotePageId,
    body: String,
) -> Result<(), DomainError> {
    registry
        .get_card_mut(card_id)?
        .save_note_page_body(note_page_id, body)
}

/// Deletes a note page from a card.
pub(super) fn delete_note_page(
    registry: &mut CardRegistry,
    card_id: CardId,
    note_page_id: NotePageId,
) -> Result<(), DomainError> {
    registry
        .get_card_mut(card_id)?
        .delete_note_page(note_page_id)
}

/// Sets the due date for a card.
pub(super) fn set_due_date(
    registry: &mut CardRegistry,
    card_id: CardId,
    due_date: DueDate,
) -> Result<(), DomainError> {
    registry.get_card_mut(card_id)?.set_due_date(Some(due_date));
    Ok(())
}

/// Clears the due date for a card.
pub(super) fn clear_due_date(
    registry: &mut CardRegistry,
    card_id: CardId,
) -> Result<(), DomainError> {
    registry.get_card_mut(card_id)?.set_due_date(None);
    Ok(())
}

/// Changes the parent of a card.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::domain::registry::CardRegistry;
/// use kanban_planner::domain::registry::mutations::reparent_card;
///
/// let mut registry = CardRegistry::new();
/// let workspace_id = registry.workspace_card_id().unwrap();
/// // ... create cards ...
/// // reparent_card(&mut registry, child_id, new_parent_id).unwrap();
/// ```
pub(super) fn reparent_card(
    registry: &mut CardRegistry,
    card_id: CardId,
    new_parent_id: CardId,
) -> Result<(), DomainError> {
    if card_id == registry.workspace_card_id()? {
        return Err(DomainError::InvalidOperation(
            "The workspace root cannot be reparented".to_string(),
        ));
    }

    if card_id == new_parent_id {
        return Err(DomainError::CycleDetected);
    }

    traversal::detect_cycle(registry, card_id, new_parent_id)?;

    let old_parent_id = registry.get_card(card_id)?.parent_id();
    if old_parent_id == Some(new_parent_id) {
        return Ok(());
    }

    registry.get_card(new_parent_id)?;
    registry.get_card_mut(new_parent_id)?.add_child(card_id);

    if let Some(old_parent_id) = old_parent_id {
        registry.get_card_mut(old_parent_id)?.remove_child(card_id);
    }

    registry
        .get_card_mut(card_id)?
        .set_parent(Some(new_parent_id));
    Ok(())
}

/// Deletes a card from the registry using the specified strategy.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::domain::registry::{CardRegistry, DeleteStrategy};
/// use kanban_planner::domain::registry::mutations::delete_card;
///
/// let mut registry = CardRegistry::new();
/// let id = registry.workspace_card_id().unwrap();
/// // ... create child ...
/// // delete_card(&mut registry, child_id, DeleteStrategy::CascadeDelete).unwrap();
/// ```
pub(super) fn delete_card(
    registry: &mut CardRegistry,
    card_id: CardId,
    strategy: DeleteStrategy,
) -> Result<(), DomainError> {
    let workspace_id = workspace::workspace_card_id(registry)?;
    if card_id == workspace_id {
        return Err(DomainError::InvalidOperation(
            "The workspace card cannot be deleted.".into(),
        ));
    }

    let card = registry.get_card(card_id)?;
    let children = card.children_ids().to_vec();
    let parent_id = card.parent_id();

    if !children.is_empty() {
        match strategy {
            DeleteStrategy::Reject => return Err(DomainError::CardHasChildren),
            DeleteStrategy::CascadeDelete => {
                for child_id in children {
                    delete_card(registry, child_id, DeleteStrategy::CascadeDelete)?;
                }
            }
            DeleteStrategy::ReparentToGrandparent => {
                let grandparent_id = parent_id.ok_or_else(|| {
                    DomainError::InvalidOperation(
                        "Cannot reparent to grandparent: card has no parent.".into(),
                    )
                })?;
                for child_id in children {
                    reparent_card(registry, child_id, grandparent_id)?;
                }
            }
        }
    }

    if let Some(parent_id) = parent_id {
        registry.get_card_mut(parent_id)?.remove_child(card_id);
    }

    registry.store.remove(&card_id);
    Ok(())
}
