use crate::domain::id::{BucketId, CardId};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("Card not found: {0}")]
    CardNotFound(CardId),

    #[error("Bucket not found: {0}")]
    BucketNotFound(BucketId),

    #[error("Duplicate bucket ID during reorder: {0}")]
    DuplicateBucketId(BucketId),

    #[error("Card title cannot be empty or blank")]
    EmptyTitle,

    #[error("A bucket named '{0}' already exists on this card")]
    DuplicateBucketName(String),

    #[error("Cannot delete a non-empty bucket; reassign or delete its cards first")]
    BucketNotEmpty,

    #[error("Cannot delete a card that still has children; choose a DeleteStrategy")]
    CardHasChildren,

    #[error("Reparenting would create a cycle in the card tree")]
    CycleDetected,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}
