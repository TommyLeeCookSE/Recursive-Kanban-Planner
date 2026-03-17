//! # Bucket Domain
//! 
//! This module defines the `Bucket` entity, which represents a logical grouping or column
//! within a Card's board (e.g., "To Do", "In Progress", "Done").
//! 
//! Buckets themselves do not store the references to the child cards they contain. 
//! Instead, child cards store a `bucket_id` reference back to the Bucket they belong to.
//! Ordering of Buckets is determined implicitly by their position within their parent's `Vec<Bucket>`.
//! 
//! For Python developers: 
//! This module uses encapsulation (private fields with public getters/setters) to ensure
//! the domain invariants cannot be bypassed. See `docs/rust-for-python-devs.md` for more on 
//! Rust `impl` and structural design.

use crate::domain::id::BucketId;

/// Represents a column or organizational divider on a Card's board.
/// 
/// The fields are intentionally private to ensure state can only be modified through
/// controlled domain methods, guaranteeing invariants (like non-empty names in the future).
/// 
/// # Examples
/// 
/// ```
/// use kanban_planner::domain::bucket::Bucket;
/// 
/// let mut bucket = Bucket::new("To Do".to_string());
/// assert_eq!(bucket.name(), "To Do");
/// 
/// bucket.rename("In Progress".to_string());
/// assert_eq!(bucket.name(), "In Progress");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bucket {
    id: BucketId,
    name: String,
}

impl Bucket {
    /// Creates a new Bucket with the given name and a freshly generated `BucketId`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use kanban_planner::domain::bucket::Bucket;
    /// 
    /// let bucket = Bucket::new("Done".to_string());
    /// ```
    pub fn new(name: String) -> Self {
        Self {
            id: BucketId::new(),
            name,
        }
    }

    /// Returns a copy of the Bucket's unique identifier.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use kanban_planner::domain::bucket::Bucket;
    /// 
    /// let bucket = Bucket::new("Backlog".to_string());
    /// let id = bucket.id();
    /// ```
    pub fn id(&self) -> BucketId {
        self.id
    }

    /// Returns a reference to the Bucket's current display name.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use kanban_planner::domain::bucket::Bucket;
    /// 
    /// let bucket = Bucket::new("Testing".to_string());
    /// assert_eq!(bucket.name(), "Testing");
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Updates the Bucket's display name. 
    /// The underlying `BucketId` remains stable, preserving all relationships to child cards.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use kanban_planner::domain::bucket::Bucket;
    /// 
    /// let mut bucket = Bucket::new("Old Name".to_string());
    /// bucket.rename("New Name".to_string());
    /// ```
    pub fn rename(&mut self, new_name: String) {
        // In the future, we can add validation here: 
        // e.g., if new_name.trim().is_empty() { return Err(...) }
        self.name = new_name;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_creation() {
        let bucket = Bucket::new("To Do".to_string());
        assert_eq!(bucket.name(), "To Do");
        // We know ID was generated successfully if it compiled and we can call method
        let _id = bucket.id(); 
    }

    #[test]
    fn test_bucket_rename() {
        let mut bucket = Bucket::new("Drafts".to_string());
        let original_id = bucket.id();
        
        bucket.rename("Ready".to_string());
        
        assert_eq!(bucket.name(), "Ready");
        assert_eq!(bucket.id(), original_id, "Renaming must not change the Bucket ID");
    }
}
