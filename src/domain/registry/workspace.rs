//! Workspace-specific registry operations.
//!
//! This module provides utilities for accessing the root workspace card
//! and its properties.
//!
//! For more on why exactly one workspace card exists in the registry,
//! see `docs/rust-for-python-devs.md`.

use super::{CardRegistry, corrupt_state};
use crate::domain::card::Card;
use crate::domain::error::DomainError;
use crate::domain::id::CardId;

/// Returns a reference to the root workspace card.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::domain::registry::CardRegistry;
/// use kanban_planner::domain::registry::workspace::workspace_card;
///
/// let registry = CardRegistry::new();
/// let workspace = workspace_card(&registry).unwrap();
/// ```
pub(super) fn workspace_card(registry: &CardRegistry) -> Result<&Card, DomainError> {
    let mut top_level_cards = registry
        .store
        .values()
        .filter(|card| card.parent_id().is_none());
    let workspace = top_level_cards.next().ok_or_else(|| {
        DomainError::InvalidOperation("Workspace card is missing from the registry".into())
    })?;

    if top_level_cards.next().is_some() {
        return Err(corrupt_state(
            "Registry contains multiple workspace cards".to_string(),
        ));
    }

    Ok(workspace)
}

/// Returns the ID of the root workspace card.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::domain::registry::CardRegistry;
/// use kanban_planner::domain::registry::workspace::workspace_card_id;
///
/// let registry = CardRegistry::new();
/// let id = workspace_card_id(&registry).unwrap();
/// ```
pub(super) fn workspace_card_id(registry: &CardRegistry) -> Result<CardId, DomainError> {
    Ok(workspace_card(registry)?.id())
}

/// Returns the number of immediate children in the workspace.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::domain::registry::CardRegistry;
/// use kanban_planner::domain::registry::workspace::workspace_child_count;
///
/// let registry = CardRegistry::new();
/// let count = workspace_child_count(&registry);
/// ```
pub(super) fn workspace_child_count(registry: &CardRegistry) -> usize {
    workspace_card(registry)
        .map(|workspace| workspace.children_ids().len())
        .unwrap_or(0)
}
