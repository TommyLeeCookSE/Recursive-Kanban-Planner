//! # Identifier Domain
//!
//! This module defines the strict, type-safe generic identifiers for the Kanban Planner.
//!
//! By wrapping the internal `Ulid` generation in specific tuple structs (`CardId`, `BucketId`),
//! we utilize the Rust Newtype pattern. This prevents logic bugs where IDs of different entities
//! are accidentally swapped during function calls.
//!
//! For Python developer translations regarding the `#[derive]` macro and `impl` structures used here,
//! see `docs/rust-for-python-devs.md`.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use ulid::Ulid;

/// Unique identifier for a Card in the planner.
///
/// Wraps a ULID to ensure type safety and prevent accidentally mixing up with other IDs.
///
/// # Examples
///
/// ```
/// use kanban_planner::domain::id::CardId;
///
/// let id = CardId::new();
/// println!("Generated Card ID: {}", id);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CardId(Ulid);

impl CardId {
    /// Generates a new unique CardId using the current time.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::id::CardId;
    ///
    /// let id = CardId::new();
    /// ```
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for CardId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for CardId {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}

/// Unique identifier for a Bucket (column) in a Card's board.
///
/// Wraps a ULID to ensure type safety.
///
/// # Examples
///
/// ```
/// use kanban_planner::domain::id::BucketId;
///
/// let bucket_id = BucketId::new();
/// println!("Generated Bucket ID: {}", bucket_id);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BucketId(Ulid);

impl BucketId {
    /// Generates a new unique BucketId using the current time.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::id::BucketId;
    ///
    /// let bucket_id = BucketId::new();
    /// ```
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for BucketId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for BucketId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for BucketId {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}

/// Unique identifier for a note page stored on a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NotePageId(Ulid);

impl NotePageId {
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for NotePageId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NotePageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for NotePageId {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}

/// Unique identifier for a label definition in the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LabelId(Ulid);

impl LabelId {
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for LabelId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for LabelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for LabelId {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}

/// Unique identifier for a rule definition in the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RuleId(Ulid);

impl RuleId {
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for RuleId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for RuleId {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_id_generation_and_display() {
        let id1 = CardId::new();
        let id2 = CardId::new();

        // Ensure they are unique
        assert_ne!(id1, id2);

        // Ensure display format matches the inner ULID string format
        assert_eq!(id1.to_string(), id1.0.to_string());
    }

    #[test]
    fn test_bucket_id_generation_and_display() {
        let id = BucketId::new();
        assert_eq!(id.to_string(), id.0.to_string());
    }

    #[test]
    fn test_note_page_id_generation_and_display() {
        let id = NotePageId::new();
        assert_eq!(id.to_string(), id.0.to_string());
    }

    #[test]
    fn test_label_id_generation_and_display() {
        let id = LabelId::new();
        assert_eq!(id.to_string(), id.0.to_string());
    }

    #[test]
    fn test_rule_id_generation_and_display() {
        let id = RuleId::new();
        assert_eq!(id.to_string(), id.0.to_string());
    }
}
