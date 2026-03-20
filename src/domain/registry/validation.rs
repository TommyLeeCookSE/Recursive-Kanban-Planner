//! Registry validation and integrity checking.
//!
//! This module provides the logic for ensuring that the `CardRegistry` is in a
//! consistent and valid state. It checks structural rules, like parent-child
//! symmetry and workspace card count.
//!
//! For more on how domain validation is handled in this project,
//! see `docs/rust-for-python-devs.md`.

use crate::domain::card::Card;
use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use crate::domain::registry::traversal::validate_parent_chain;
use crate::domain::registry::{CardRegistry, corrupt_state};
use std::collections::HashSet;

/// Validates the internal consistency of the entire card registry.
///
/// This checks that all cards have valid references, there is exactly one
/// workspace root, and no structural invariants are violated.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::domain::registry::CardRegistry;
/// use kanban_planner::domain::registry::validation::validate_registry;
///
/// let registry = CardRegistry::new();
/// validate_registry(&registry).unwrap();
/// ```
pub(super) fn validate_registry(registry: &CardRegistry) -> Result<(), DomainError> {
    let mut referenced_children = HashSet::new();
    validate_card_records(registry, &mut referenced_children)?;
    validate_workspace_card_shape(registry, &referenced_children)?;
    Ok(())
}

/// Validates all individual card records within the registry.
fn validate_card_records(
    registry: &CardRegistry,
    referenced_children: &mut HashSet<CardId>,
) -> Result<(), DomainError> {
    for (card_id, card) in &registry.store {
        validate_card_key_and_title(*card_id, card)?;
        validate_note_pages(card)?;
        validate_parent_reference(registry, *card_id, card)?;
        validate_child_references(registry, *card_id, card, referenced_children)?;
        validate_parent_chain(registry, *card_id)?;
    }

    Ok(())
}

/// Ensures a card's ID matches its storage key and its title is valid.
fn validate_card_key_and_title(card_id: CardId, card: &Card) -> Result<(), DomainError> {
    if card.id() != card_id {
        return Err(corrupt_state(format!(
            "Registry key {card_id} does not match card id {}",
            card.id()
        )));
    }

    if card.title().trim().is_empty() {
        return Err(corrupt_state(format!(
            "Card {card_id} has a blank title in persisted state"
        )));
    }

    Ok(())
}

/// Validates that a card's parent reference is symmetric.
fn validate_parent_reference(
    registry: &CardRegistry,
    card_id: CardId,
    card: &Card,
) -> Result<(), DomainError> {
    if let Some(parent_id) = card.parent_id() {
        let parent = registry.store.get(&parent_id).ok_or_else(|| {
            corrupt_state(format!(
                "Card {card_id} references missing parent {parent_id}"
            ))
        })?;

        if !parent.children_ids().contains(&card_id) {
            return Err(corrupt_state(format!(
                "Parent {parent_id} is missing child reference to {card_id}"
            )));
        }
    }

    Ok(())
}

/// Validates that all children referenced by a card exist and point back to it.
fn validate_child_references(
    registry: &CardRegistry,
    card_id: CardId,
    card: &Card,
    referenced_children: &mut HashSet<CardId>,
) -> Result<(), DomainError> {
    let mut local_children = HashSet::new();
    for child_id in card.children_ids() {
        if *child_id == card_id {
            return Err(corrupt_state(format!(
                "Card {card_id} cannot reference itself as a child"
            )));
        }

        if !local_children.insert(*child_id) {
            return Err(corrupt_state(format!(
                "Card {card_id} contains duplicate child reference {child_id}"
            )));
        }

        if !referenced_children.insert(*child_id) {
            return Err(corrupt_state(format!(
                "Child card {child_id} is referenced by more than one parent"
            )));
        }

        let child = registry.store.get(child_id).ok_or_else(|| {
            corrupt_state(format!(
                "Card {card_id} references missing child {child_id}"
            ))
        })?;

        if child.parent_id() != Some(card_id) {
            return Err(corrupt_state(format!(
                "Child {child_id} does not point back to parent {card_id}"
            )));
        }
    }

    Ok(())
}

/// Ensures the registry has exactly one workspace root and no orphan cards.
fn validate_workspace_card_shape(
    registry: &CardRegistry,
    referenced_children: &HashSet<CardId>,
) -> Result<(), DomainError> {
    let mut top_level_cards = HashSet::new();

    for (card_id, card) in &registry.store {
        match card.parent_id() {
            None => {
                top_level_cards.insert(*card_id);
                if referenced_children.contains(card_id) {
                    return Err(corrupt_state(format!(
                        "Workspace card {card_id} must not be referenced as a child"
                    )));
                }
            }
            Some(_) => {
                if !referenced_children.contains(card_id) {
                    return Err(corrupt_state(format!(
                        "Non-workspace card {card_id} is not referenced by its parent"
                    )));
                }
            }
        }
    }

    if top_level_cards.len() != 1 {
        return Err(corrupt_state(format!(
            "Registry must contain exactly one workspace card, found {}",
            top_level_cards.len()
        )));
    }

    Ok(())
}

/// Validates that all note pages on a card have valid titles and unique IDs.
fn validate_note_pages(card: &Card) -> Result<(), DomainError> {
    let mut ids = HashSet::new();
    for note in card.notes() {
        if note.title().trim().is_empty() {
            return Err(corrupt_state(format!(
                "Card {} contains a note page with a blank title",
                card.id()
            )));
        }

        if !ids.insert(note.id()) {
            return Err(corrupt_state(format!(
                "Card {} contains duplicate note page id {}",
                card.id(),
                note.id()
            )));
        }
    }
    Ok(())
}
