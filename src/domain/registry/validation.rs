use crate::domain::card::Card;
use crate::domain::error::DomainError;
use crate::domain::registry::traversal::validate_parent_chain;
use crate::domain::registry::{CardRegistry, corrupt_state};
use std::collections::HashSet;

pub(super) fn validate_registry(registry: &CardRegistry) -> Result<(), DomainError> {
    let mut referenced_children = HashSet::new();
    let mut top_level_cards = HashSet::new();

    for (card_id, card) in &registry.store {
        if card.id() != *card_id {
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

        validate_note_pages(card)?;

        if let Some(parent_id) = card.parent_id() {
            let parent = registry.store.get(&parent_id).ok_or_else(|| {
                corrupt_state(format!(
                    "Card {card_id} references missing parent {parent_id}"
                ))
            })?;

            if !parent.children_ids().contains(card_id) {
                return Err(corrupt_state(format!(
                    "Parent {parent_id} is missing child reference to {card_id}"
                )));
            }
        }

        let mut local_children = HashSet::new();
        for child_id in card.children_ids() {
            if *child_id == *card_id {
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

            if child.parent_id() != Some(*card_id) {
                return Err(corrupt_state(format!(
                    "Child {child_id} does not point back to parent {card_id}"
                )));
            }
        }

        validate_parent_chain(registry, *card_id)?;
    }

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
