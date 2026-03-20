use crate::domain::card::Card;
use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use tracing::trace;

pub struct BoardView<'a> {
    pub card: &'a Card,
    pub children: Vec<&'a Card>,
}

#[derive(Debug)]
pub struct CardPreviewView<'a> {
    pub card: &'a Card,
    pub children: Vec<&'a Card>,
}

fn build_card_children_view(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<(&Card, Vec<&Card>), DomainError> {
    Ok((registry.get_card(card_id)?, registry.get_children(card_id)?))
}

pub fn build_board_view(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<BoardView<'_>, DomainError> {
    trace!(%card_id, "Building board view");
    let (card, children) = build_card_children_view(card_id, registry)?;
    Ok(BoardView { card, children })
}

pub fn build_card_preview_view(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<CardPreviewView<'_>, DomainError> {
    trace!(%card_id, "Building card preview view");
    let (card, children) = build_card_children_view(card_id, registry)?;
    Ok(CardPreviewView { card, children })
}
