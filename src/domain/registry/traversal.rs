use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use crate::domain::registry::{CardRegistry, corrupt_state};
use std::collections::HashSet;

pub(super) fn detect_cycle(
    registry: &CardRegistry,
    card_id: CardId,
    proposed_parent_id: CardId,
) -> Result<(), DomainError> {
    let mut current_ancestor_id = Some(proposed_parent_id);
    while let Some(ancestor_id) = current_ancestor_id {
        if ancestor_id == card_id {
            return Err(DomainError::CycleDetected);
        }

        current_ancestor_id = registry.get_card(ancestor_id)?.parent_id();
    }

    Ok(())
}

pub(super) fn validate_parent_chain(
    registry: &CardRegistry,
    card_id: CardId,
) -> Result<(), DomainError> {
    let mut current_parent = registry
        .store
        .get(&card_id)
        .ok_or_else(|| corrupt_state(format!("Card {card_id} is missing from the registry")))?
        .parent_id();
    let mut seen = HashSet::new();

    while let Some(parent_id) = current_parent {
        if !seen.insert(parent_id) {
            return Err(corrupt_state(format!(
                "Card {card_id} participates in a parent cycle involving {parent_id}"
            )));
        }

        let parent = registry.store.get(&parent_id).ok_or_else(|| {
            corrupt_state(format!(
                "Card {card_id} references missing ancestor {parent_id}"
            ))
        })?;

        current_parent = parent.parent_id();
    }

    Ok(())
}
