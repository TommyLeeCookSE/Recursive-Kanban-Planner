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

/// A view representing a single board and its immediate children.
///
/// # Examples
///
/// ```
/// use kanban_planner::application::{BoardView, build_board_view};
/// use kanban_planner::domain::registry::CardRegistry;
///
/// let registry = CardRegistry::new();
/// let root_id = registry.workspace_card_id().unwrap();
/// let view = build_board_view(root_id, &registry).unwrap();
/// assert_eq!(view.card.title(), "My Workspace");
/// ```
pub struct BoardView<'a> {
    /// The primary card acting as the board.
    pub card: &'a Card,
    /// The list of immediate child cards in their display order.
    pub children: Vec<&'a Card>,
}

/// A lightweight preview of a card, including its immediate children.
///
/// # Examples
///
/// ```
/// use kanban_planner::application::{CardPreviewView, build_card_preview_view};
/// use kanban_planner::domain::registry::CardRegistry;
///
/// let registry = CardRegistry::new();
/// let root_id = registry.workspace_card_id().unwrap();
/// let preview = build_card_preview_view(root_id, &registry).unwrap();
/// assert_eq!(preview.card.id(), root_id);
/// ```
#[derive(Debug)]
pub struct CardPreviewView<'a> {
    /// The card being previewed.
    pub card: &'a Card,
    /// The immediate children of the card.
    pub children: Vec<&'a Card>,
}

fn build_card_children_view(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<(&Card, Vec<&Card>), DomainError> {
    Ok((registry.get_card(card_id)?, registry.get_children(card_id)?))
}

/// Constructs a `BoardView` for a given card.
///
/// # Examples
///
/// ```
/// use kanban_planner::application::build_board_view;
/// use kanban_planner::domain::registry::CardRegistry;
///
/// let registry = CardRegistry::new();
/// let root_id = registry.workspace_card_id().unwrap();
/// let view = build_board_view(root_id, &registry).unwrap();
/// ```
pub fn build_board_view(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<BoardView<'_>, DomainError> {
    trace!(%card_id, "Building board view");
    let (card, children) = build_card_children_view(card_id, registry)?;
    Ok(BoardView { card, children })
}

/// Constructs a `CardPreviewView` for a given card.
///
/// # Examples
///
/// ```
/// use kanban_planner::application::build_card_preview_view;
/// use kanban_planner::domain::registry::CardRegistry;
///
/// let registry = CardRegistry::new();
/// let root_id = registry.workspace_card_id().unwrap();
/// let preview = build_card_preview_view(root_id, &registry).unwrap();
/// ```
pub fn build_card_preview_view(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<CardPreviewView<'_>, DomainError> {
    trace!(%card_id, "Building card preview view");
    let (card, children) = build_card_children_view(card_id, registry)?;
    Ok(CardPreviewView { card, children })
}
