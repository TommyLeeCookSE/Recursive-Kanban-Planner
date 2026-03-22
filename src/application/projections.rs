//! Read-only views of the domain model for UI display.
//!
//! This module provides "projections" – data structures that combine multiple
//! domain objects into a single package optimized for rendering. This prevents
//! the UI from needing to perform complex traversals or multiple registry
//! lookups directly.
//!
//! For an explanation of how these projections compare to Python's "View Objects"
//! or "Serializers", see `docs/rust-for-python-devs.md`.

use crate::domain::card::Card;
use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use tracing::trace;

/// A unified view representing a single card and its immediate children.
///
/// This projection is used for both full-screen board views and small card previews.
///
/// # Examples
///
/// ```
/// use kanban_planner::application::{CardView, build_card_view};
/// use kanban_planner::domain::registry::CardRegistry;
///
/// let registry = CardRegistry::new();
/// let root_id = registry.workspace_card_id().unwrap();
/// let view = build_card_view(root_id, &registry).unwrap();
/// assert_eq!(view.card.title(), "My Workspace");
/// ```
pub struct CardView<'a> {
    /// The primary card acting as the board.
    pub card: &'a Card,
    /// The list of immediate child cards in their display order.
    pub children: Vec<&'a Card>,
}

fn build_card_children_view(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<(&Card, Vec<&Card>), DomainError> {
    Ok((registry.get_card(card_id)?, registry.get_children(card_id)?))
}

/// Constructs a `CardView` for a given card.
///
/// This is the primary entry point for projecting a card and its children.
///
/// # Examples
///
/// ```
/// use kanban_planner::application::build_card_view;
/// use kanban_planner::domain::registry::CardRegistry;
///
/// let registry = CardRegistry::new();
/// let root_id = registry.workspace_card_id().unwrap();
/// let view = build_card_view(root_id, &registry).unwrap();
/// ```
pub fn build_card_view(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<CardView<'_>, DomainError> {
    trace!(%card_id, "Building card view");
    let (card, children) = build_card_children_view(card_id, registry)?;
    Ok(CardView { card, children })
}

/// Compatibility alias for `build_card_view` when used for a board screen.
pub fn build_board_view(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<CardView<'_>, DomainError> {
    build_card_view(card_id, registry)
}

/// Compatibility alias for `build_card_view` when used for a card preview.
pub fn build_card_preview_view(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<CardView<'_>, DomainError> {
    build_card_view(card_id, registry)
}
