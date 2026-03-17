//! # Card Domain
//!
//! This module defines the `Card` entity — the single, unified building block of the entire system.
//! A card can represent a task, a project, a team, or any logical grouping.
//!
//! ## The Tree
//! Cards form a strict tree. Each card has at most one parent (`parent_id: Option<CardId>`).
//! Root cards have `parent_id: None` and are never assigned to a bucket.
//! All non-root cards have `parent_id: Some(_)` and are **always** assigned to a bucket in
//! their parent's board (at minimum, the automatically created "Unassigned" bucket).
//!
//! ## Mutation Rules
//! All fields are private. State can only change through the controlled methods on this struct
//! or through the `CardRegistry` for multi-card operations (e.g., reparenting).
//!
//! See `docs/rust-for-python-devs.md` for explanations of Rust patterns used here.

use crate::domain::bucket::Bucket;
use crate::domain::id::{BucketId, CardId};

/// The name automatically given to the default bucket on every card's board.
pub const UNASSIGNED_BUCKET_NAME: &str = "Unassigned";

/// The fundamental entity of the Recursive Kanban Planner.
///
/// Every node in the system — task, project, team, workspace — is a `Card`.
/// Cards form a tree via `parent_id`, and each card's `buckets` define the columns
/// of its own Kanban board for organizing its children.
///
/// # Examples
///
/// ```
/// use kanban_planner::domain::card::Card;
///
/// let root = Card::new_root("My Projects".to_string());
/// assert!(root.parent_id().is_none());
/// assert!(!root.buckets().is_empty(), "Root cards must have at least the Unassigned bucket");
/// ```
#[derive(Debug, Clone)]
pub struct Card {
    id: CardId,
    title: String,
    parent_id: Option<CardId>,
    children_ids: Vec<CardId>,
    bucket_id: Option<BucketId>,
    buckets: Vec<Bucket>,
}

impl Card {
    /// Creates a new root-level Card with no parent and no bucket assignment.
    ///
    /// Root cards automatically receive an "Unassigned" bucket so they can immediately
    /// organize their children.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::card::Card;
    ///
    /// let card = Card::new_root("Work Projects".to_string());
    /// assert!(card.parent_id().is_none());
    /// assert!(card.bucket_id().is_none());
    /// ```
    pub fn new_root(title: String) -> Self {
        Self {
            id: CardId::new(),
            title,
            parent_id: None,
            children_ids: Vec::new(),
            bucket_id: None,
            buckets: vec![Bucket::new(UNASSIGNED_BUCKET_NAME.to_string())],
        }
    }

    /// Creates a new child Card assigned to a specific parent and bucket.
    ///
    /// Child cards must always belong to a parent and be placed in one of the parent's
    /// buckets. The `bucket_id` must reference a valid bucket in the parent's `buckets` list.
    /// Validation of the `bucket_id` against the parent's actual buckets is performed by
    /// the `CardRegistry`, which has access to the full card graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::card::Card;
    /// use kanban_planner::domain::id::{BucketId, CardId};
    ///
    /// let parent = Card::new_root("Work".to_string());
    /// let bucket_id = parent.buckets()[0].id();
    /// let child = Card::new_child("Task Alpha".to_string(), parent.id(), bucket_id);
    ///
    /// assert_eq!(child.parent_id(), Some(parent.id()));
    /// assert_eq!(child.bucket_id(), Some(bucket_id));
    /// ```
    pub fn new_child(title: String, parent_id: CardId, bucket_id: BucketId) -> Self {
        Self {
            id: CardId::new(),
            title,
            parent_id: Some(parent_id),
            children_ids: Vec::new(),
            bucket_id: Some(bucket_id),
            buckets: vec![Bucket::new(UNASSIGNED_BUCKET_NAME.to_string())],
        }
    }

    // -------------------------------------------------------------------------
    // Getters
    // -------------------------------------------------------------------------

    /// Returns the card's unique identifier.
    pub fn id(&self) -> CardId {
        self.id
    }

    /// Returns the card's current display title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the parent card's ID, or `None` if this is a root card.
    pub fn parent_id(&self) -> Option<CardId> {
        self.parent_id
    }

    /// Returns a slice of the ordered child card IDs.
    pub fn children_ids(&self) -> &[CardId] {
        &self.children_ids
    }

    /// Returns the bucket ID this card is assigned to in its parent's board,
    /// or `None` if this is a root card.
    pub fn bucket_id(&self) -> Option<BucketId> {
        self.bucket_id
    }

    /// Returns a slice of the ordered buckets defined for this card's board.
    pub fn buckets(&self) -> &[Bucket] {
        &self.buckets
    }

    // -------------------------------------------------------------------------
    // Title
    // -------------------------------------------------------------------------

    /// Renames the card. Returns an error if the new title is empty or only whitespace.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::card::Card;
    ///
    /// let mut card = Card::new_root("Old".to_string());
    /// assert!(card.rename("New Title".to_string()).is_ok());
    /// assert_eq!(card.title(), "New Title");
    ///
    /// assert!(card.rename("  ".to_string()).is_err(), "Blank titles must be rejected");
    /// ```
    pub fn rename(&mut self, new_title: String) -> Result<(), String> {
        if new_title.trim().is_empty() {
            return Err("Card title cannot be empty or blank.".to_string());
        }
        self.title = new_title;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Bucket assignment (which bucket this card lives in on its parent's board)
    // -------------------------------------------------------------------------

    /// Moves this card into a different bucket on its parent's board.
    ///
    /// This operation only updates the card's own `bucket_id`. The caller (Registry or
    /// Application layer) is responsible for verifying the `BucketId` exists in the parent.
    pub fn assign_to_bucket(&mut self, bucket_id: BucketId) {
        self.bucket_id = Some(bucket_id);
    }

    // -------------------------------------------------------------------------
    // Bucket management (the columns on this card's own board)
    // -------------------------------------------------------------------------

    /// Adds a new bucket (column) to this card's board.
    /// Returns an error if a bucket with the same name already exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::card::Card;
    ///
    /// let mut card = Card::new_root("Project".to_string());
    /// assert!(card.add_bucket("In Progress".to_string()).is_ok());
    /// assert!(card.add_bucket("In Progress".to_string()).is_err(), "Duplicate names must fail");
    /// ```
    pub fn add_bucket(&mut self, name: String) -> Result<BucketId, String> {
        if self.buckets.iter().any(|b| b.name().eq_ignore_ascii_case(&name)) {
            return Err(format!("A bucket named '{}' already exists on this card.", name));
        }
        let bucket = Bucket::new(name);
        let id = bucket.id();
        self.buckets.push(bucket);
        Ok(id)
    }

    /// Removes a bucket from this card's board by its ID.
    /// The "Unassigned" bucket cannot be removed — it is always the fallback.
    /// Returns an error if the bucket is not found or is the Unassigned bucket.
    pub fn remove_bucket(&mut self, bucket_id: BucketId) -> Result<(), String> {
        let pos = self
            .buckets
            .iter()
            .position(|b| b.id() == bucket_id)
            .ok_or_else(|| "Bucket not found on this card.".to_string())?;

        if self.buckets[pos].name() == UNASSIGNED_BUCKET_NAME {
            return Err("The 'Unassigned' bucket cannot be removed.".to_string());
        }

        self.buckets.remove(pos);
        Ok(())
    }

    /// Reorders the buckets by providing a new ordered list of `BucketId`s.
    /// All existing bucket IDs must be present — none may be added or dropped.
    pub fn reorder_buckets(&mut self, ordered_ids: Vec<BucketId>) -> Result<(), String> {
        if ordered_ids.len() != self.buckets.len() {
            return Err("Reorder list must contain exactly the same buckets.".to_string());
        }

        let mut reordered = Vec::with_capacity(self.buckets.len());
        for id in &ordered_ids {
            let bucket = self
                .buckets
                .iter()
                .find(|b| b.id() == *id)
                .ok_or_else(|| format!("Unknown bucket ID in reorder list."))?
                .clone();
            reordered.push(bucket);
        }

        self.buckets = reordered;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Child management (called by CardRegistry during structural mutations)
    // -------------------------------------------------------------------------

    /// Appends a child card ID to this card's ordered children list.
    /// This is called by the `CardRegistry` when a new child card is created or reparented.
    pub(crate) fn add_child(&mut self, child_id: CardId) {
        self.children_ids.push(child_id);
    }

    /// Removes a child card ID from this card's children list.
    /// This is called by the `CardRegistry` during deletion or reparenting.
    pub(crate) fn remove_child(&mut self, child_id: CardId) {
        self.children_ids.retain(|id| *id != child_id);
    }

    /// Sets the parent ID directly. Only callable within the `domain` module,
    /// ensuring reparenting always goes through the `CardRegistry`.
    pub(crate) fn set_parent(&mut self, parent_id: Option<CardId>) {
        self.parent_id = parent_id;
    }

    /// Sets the bucket ID directly. Used by the `CardRegistry` during reparenting
    /// when the child must be reassigned to the new parent's Unassigned bucket.
    pub(crate) fn set_bucket(&mut self, bucket_id: Option<BucketId>) {
        self.bucket_id = bucket_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_root_has_no_parent_or_bucket() {
        let card = Card::new_root("Root".to_string());
        assert!(card.parent_id().is_none());
        assert!(card.bucket_id().is_none());
    }

    #[test]
    fn test_new_root_has_unassigned_bucket() {
        let card = Card::new_root("Root".to_string());
        assert_eq!(card.buckets().len(), 1);
        assert_eq!(card.buckets()[0].name(), UNASSIGNED_BUCKET_NAME);
    }

    #[test]
    fn test_new_child_has_parent_and_bucket() {
        let parent = Card::new_root("Parent".to_string());
        let bucket_id = parent.buckets()[0].id();
        let child = Card::new_child("Child".to_string(), parent.id(), bucket_id);

        assert_eq!(child.parent_id(), Some(parent.id()));
        assert_eq!(child.bucket_id(), Some(bucket_id));
    }

    #[test]
    fn test_rename_rejects_blank_title() {
        let mut card = Card::new_root("Title".to_string());
        assert!(card.rename("  ".to_string()).is_err());
        assert_eq!(card.title(), "Title", "Title must be unchanged after a failed rename");
    }

    #[test]
    fn test_add_duplicate_bucket_fails() {
        let mut card = Card::new_root("Project".to_string());
        assert!(card.add_bucket("In Progress".to_string()).is_ok());
        assert!(card.add_bucket("In Progress".to_string()).is_err());
    }

    #[test]
    fn test_remove_unassigned_bucket_fails() {
        let mut card = Card::new_root("Project".to_string());
        let unassigned_id = card.buckets()[0].id();
        assert!(card.remove_bucket(unassigned_id).is_err());
    }

    #[test]
    fn test_reorder_buckets() {
        let mut card = Card::new_root("Project".to_string());
        let id_a = card.add_bucket("Alpha".to_string()).unwrap();
        let id_b = card.add_bucket("Beta".to_string()).unwrap();
        let unassigned_id = card.buckets()[0].id();

        // Reorder: Beta, Unassigned, Alpha
        assert!(card.reorder_buckets(vec![id_b, unassigned_id, id_a]).is_ok());
        assert_eq!(card.buckets()[0].name(), "Beta");
        assert_eq!(card.buckets()[1].name(), UNASSIGNED_BUCKET_NAME);
        assert_eq!(card.buckets()[2].name(), "Alpha");
    }
}
