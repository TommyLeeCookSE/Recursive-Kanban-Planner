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

/// A lightweight representation of a card in the topology graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopologyNode {
    pub id: CardId,
    pub parent_id: Option<CardId>,
    pub title: String,
}

/// A projection representing the structural relationships in the registry.
#[derive(Clone, Debug)]
pub struct GraphTopologyView {
    pub nodes: Vec<TopologyNode>,
    /// All directed edges (parent -> child)
    pub edges: Vec<(CardId, CardId)>,
    pub center_id: CardId,
}

/// Constructs a full `GraphTopologyView` centered on a specific card.
pub fn build_graph_topology(
    center_id: CardId,
    registry: &CardRegistry,
) -> Result<GraphTopologyView, DomainError> {
    // Validate center exists
    registry.get_card(center_id)?;

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for card in registry.all_cards() {
        nodes.push(TopologyNode {
            id: card.id(),
            parent_id: card.parent_id(),
            title: card.title().to_string(),
        });

        for child_id in card.children_ids() {
            edges.push((card.id(), *child_id));
        }
    }

    Ok(GraphTopologyView {
        nodes,
        edges,
        center_id,
    })
}
