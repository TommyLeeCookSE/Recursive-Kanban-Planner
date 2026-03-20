//! Domain-specific error types.
//!
//! This module defines the `DomainError` enum, which represents all possible
//! failure states within the domain logic. This project prefers explicit `Result`
//! returns over panics to ensure the UI can gracefully handle and report errors.
//!
//! For a comparison of Rust's `Result` pattern versus Python's exceptions,
//! see `docs/rust-for-python-devs.md`.

use crate::domain::id::CardId;

/// Errors that can occur during domain operations.
///
/// # Examples
///
/// ```
/// use kanban_planner::domain::error::DomainError;
/// use kanban_planner::domain::id::CardId;
///
/// let error = DomainError::CardNotFound(CardId::new());
/// assert!(error.to_string().contains("Card not found"));
/// ```
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DomainError {
    /// A card with the specified ID does not exist in the registry.
    #[error("Card not found: {0}")]
    CardNotFound(CardId),

    /// A card or note page title was empty or contained only whitespace.
    #[error("Card title cannot be empty or blank")]
    EmptyTitle,

    /// An attempt was made to delete a card without specifying how to handle its children.
    #[error("Cannot delete a card that still has children; choose a DeleteStrategy")]
    CardHasChildren,

    /// A move operation would result in a card becoming its own ancestor.
    #[error("Reparenting would create a cycle in the card tree")]
    CycleDetected,

    /// A catch-all for operations that violate domain invariants in unexpected ways.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Errors encountered when attempting to load data from older versions of the app.
    #[error("Incompatible legacy data: {0}")]
    IncompatibleLegacyData(String),
}
