//! Registry traversal and structural analysis utilities.
//!
//! This module provides functions for traversing the card tree and validating
//! its structure, such as detecting circular references.
//!
//! For more on how Rust handles tree structures and parent-child references
//! compared to Python's object graphs, see `docs/rust-for-python-devs.md`.

use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use crate::domain::registry::{CardRegistry, corrupt_state};
use std::collections::HashSet;

/// Checks if setting a new parent for a card would create a cycle.
///
/// Returns `Err(DomainError::CycleDetected)` if a cycle is found.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::domain::registry::CardRegistry;
/// use kanban_planner::domain::registry::traversal::detect_cycle;
///
/// let registry = CardRegistry::new();
/// let workspace_id = registry.workspace_card_id().unwrap();
/// detect_cycle(&registry, workspace_id, workspace_id).unwrap_err();
/// ```
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

/// Validates that a card's parent chain eventually terminates at a root.
///
/// Returns `Err` if a cycle is detected or if a parent reference is missing.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::domain::registry::CardRegistry;
/// use kanban_planner::domain::registry::traversal::validate_parent_chain;
///
/// let registry = CardRegistry::new();
/// let workspace_id = registry.workspace_card_id().unwrap();
/// validate_parent_chain(&registry, workspace_id).unwrap();
/// ```
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
