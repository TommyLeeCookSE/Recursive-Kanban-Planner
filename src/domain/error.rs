use crate::domain::id::CardId;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("Card not found: {0}")]
    CardNotFound(CardId),

    #[error("Card title cannot be empty or blank")]
    EmptyTitle,

    #[error("Cannot delete a card that still has children; choose a DeleteStrategy")]
    CardHasChildren,

    #[error("Reparenting would create a cycle in the card tree")]
    CycleDetected,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Incompatible legacy data: {0}")]
    IncompatibleLegacyData(String),
}
