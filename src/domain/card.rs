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
use crate::domain::due_date::DueDate;
use crate::domain::error::DomainError;
use crate::domain::id::{BucketId, CardId, NotePageId};
use crate::domain::note::NotePage;

use serde::{Deserialize, Serialize};

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
/// let root = Card::new_root("My Projects".to_string()).unwrap();
/// assert!(root.parent_id().is_none());
/// assert!(!root.buckets().is_empty(), "Root cards must have at least the Unassigned bucket");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    id: CardId,
    title: String,
    parent_id: Option<CardId>,
    children_ids: Vec<CardId>,
    bucket_id: Option<BucketId>,
    buckets: Vec<Bucket>,
    #[serde(default)]
    notes: Vec<NotePage>,
    #[serde(default)]
    due_date: Option<DueDate>,
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
    /// let card = Card::new_root("Work Projects".to_string()).unwrap();
    /// assert!(card.parent_id().is_none());
    /// assert!(card.bucket_id().is_none());
    /// ```
    pub fn new_root(title: String) -> Result<Self, DomainError> {
        if title.trim().is_empty() {
            return Err(DomainError::EmptyTitle);
        }
        Ok(Self {
            id: CardId::new(),
            title,
            parent_id: None,
            children_ids: Vec::new(),
            bucket_id: None,
            buckets: vec![Bucket::new(UNASSIGNED_BUCKET_NAME.to_string())],
            notes: Vec::new(),
            due_date: None,
        })
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
    /// let parent = Card::new_root("Work".to_string()).unwrap();
    /// let bucket_id = parent.buckets()[0].id();
    /// let child = Card::new_child("Task Alpha".to_string(), parent.id(), bucket_id).unwrap();
    ///
    /// assert_eq!(child.parent_id(), Some(parent.id()));
    /// assert_eq!(child.bucket_id(), Some(bucket_id));
    /// ```
    pub fn new_child(
        title: String,
        parent_id: CardId,
        bucket_id: BucketId,
    ) -> Result<Self, DomainError> {
        if title.trim().is_empty() {
            return Err(DomainError::EmptyTitle);
        }
        Ok(Self {
            id: CardId::new(),
            title,
            parent_id: Some(parent_id),
            children_ids: Vec::new(),
            bucket_id: Some(bucket_id),
            buckets: vec![Bucket::new(UNASSIGNED_BUCKET_NAME.to_string())],
            notes: Vec::new(),
            due_date: None,
        })
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

    pub fn notes(&self) -> &[NotePage] {
        &self.notes
    }

    pub fn due_date(&self) -> Option<&DueDate> {
        self.due_date.as_ref()
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
    /// let mut card = Card::new_root("Old".to_string()).unwrap();
    /// assert!(card.rename("New Title".to_string()).is_ok());
    /// assert_eq!(card.title(), "New Title");
    ///
    /// assert!(card.rename("  ".to_string()).is_err(), "Blank titles must be rejected");
    /// ```
    pub fn rename(&mut self, new_title: String) -> Result<(), DomainError> {
        if new_title.trim().is_empty() {
            return Err(DomainError::EmptyTitle);
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
    /// let mut card = Card::new_root("Project".to_string()).unwrap();
    /// assert!(card.add_bucket("In Progress".to_string()).is_ok());
    /// assert!(card.add_bucket("In Progress".to_string()).is_err(), "Duplicate names must fail");
    /// ```
    pub fn add_bucket(&mut self, name: String) -> Result<BucketId, DomainError> {
        if self
            .buckets
            .iter()
            .any(|b| b.name().eq_ignore_ascii_case(&name))
        {
            return Err(DomainError::DuplicateBucketName(name));
        }
        let bucket = Bucket::new(name);
        let id = bucket.id();
        self.buckets.push(bucket);
        Ok(id)
    }

    /// Renames a bucket.
    /// Returns an error if the new name is "Unassigned" (reserved) or if another
    /// bucket already has that name.
    pub fn rename_bucket(&mut self, id: BucketId, new_name: String) -> Result<(), DomainError> {
        if new_name == UNASSIGNED_BUCKET_NAME {
            return Err(DomainError::InvalidOperation(
                "Cannot rename a bucket to 'Unassigned'.".into(),
            ));
        }

        if self
            .buckets
            .iter()
            .any(|b| b.id() != id && b.name().eq_ignore_ascii_case(&new_name))
        {
            return Err(DomainError::DuplicateBucketName(new_name));
        }

        let bucket = self
            .buckets
            .iter_mut()
            .find(|b| b.id() == id)
            .ok_or(DomainError::BucketNotFound(id))?;

        if bucket.name() == UNASSIGNED_BUCKET_NAME {
            return Err(DomainError::InvalidOperation(
                "The 'Unassigned' bucket cannot be renamed.".to_string(),
            ));
        }

        bucket.rename(new_name);
        Ok(())
    }

    /// Removes a bucket from this card's board by its ID.
    /// The "Unassigned" bucket cannot be removed — it is always the fallback.
    /// Returns an error if the bucket is not found or is the Unassigned bucket.
    pub fn remove_bucket(&mut self, bucket_id: BucketId) -> Result<(), DomainError> {
        let pos = self
            .buckets
            .iter()
            .position(|b| b.id() == bucket_id)
            .ok_or(DomainError::BucketNotFound(bucket_id))?;

        if self.buckets[pos].name() == UNASSIGNED_BUCKET_NAME {
            return Err(DomainError::InvalidOperation(
                "The 'Unassigned' bucket cannot be removed.".to_string(),
            ));
        }

        self.buckets.remove(pos);
        Ok(())
    }

    /// Reorders the buckets by providing a new ordered list of `BucketId`s.
    /// All existing bucket IDs must be present — none may be added or dropped.
    pub fn reorder_buckets(&mut self, ordered_ids: Vec<BucketId>) -> Result<(), DomainError> {
        if ordered_ids.len() != self.buckets.len() {
            return Err(DomainError::InvalidOperation(
                "Reorder list length does not match existing buckets".to_string(),
            ));
        }

        let mut reordered = Vec::with_capacity(self.buckets.len());
        let mut seen = std::collections::HashSet::new();

        for id in ordered_ids {
            if !seen.insert(id) {
                return Err(DomainError::DuplicateBucketId(id));
            }
            let bucket = self
                .buckets
                .iter()
                .find(|b| b.id() == id)
                .ok_or(DomainError::BucketNotFound(id))?
                .clone();
            reordered.push(bucket);
        }

        self.buckets = reordered;
        Ok(())
    }

    /// Reorders children by providing a new ordered list of `CardId`s.
    /// All existing child IDs must be present — none may be added or dropped.
    pub fn reorder_children(&mut self, ordered_ids: Vec<CardId>) -> Result<(), DomainError> {
        if ordered_ids.len() != self.children_ids.len() {
            return Err(DomainError::InvalidOperation(
                "Reorder list length does not match existing children".to_string(),
            ));
        }

        let mut seen = std::collections::HashSet::new();
        for id in &ordered_ids {
            if !seen.insert(*id) {
                return Err(DomainError::InvalidOperation(format!(
                    "Duplicate child ID in reorder list: {id}"
                )));
            }
            if !self.children_ids.contains(id) {
                return Err(DomainError::CardNotFound(*id));
            }
        }

        self.children_ids = ordered_ids;
        Ok(())
    }

    pub fn add_note_page(&mut self, title: String) -> Result<NotePageId, DomainError> {
        let note = NotePage::new(title)?;
        let id = note.id();
        self.notes.push(note);
        Ok(id)
    }

    pub fn rename_note_page(&mut self, id: NotePageId, title: String) -> Result<(), DomainError> {
        let note = self
            .notes
            .iter_mut()
            .find(|note| note.id() == id)
            .ok_or_else(|| DomainError::InvalidOperation(format!("Note page not found: {id}")))?;
        note.rename(title)
    }

    pub fn save_note_page_body(&mut self, id: NotePageId, body: String) -> Result<(), DomainError> {
        let note = self
            .notes
            .iter_mut()
            .find(|note| note.id() == id)
            .ok_or_else(|| DomainError::InvalidOperation(format!("Note page not found: {id}")))?;
        note.set_body(body);
        Ok(())
    }

    pub fn delete_note_page(&mut self, id: NotePageId) -> Result<(), DomainError> {
        let original_len = self.notes.len();
        self.notes.retain(|note| note.id() != id);
        if self.notes.len() == original_len {
            return Err(DomainError::InvalidOperation(format!(
                "Note page not found: {id}"
            )));
        }
        Ok(())
    }

    pub fn set_due_date(&mut self, due_date: Option<DueDate>) {
        self.due_date = due_date;
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
        let card = Card::new_root("Root".to_string()).unwrap();
        assert!(card.parent_id().is_none());
        assert!(card.bucket_id().is_none());
    }

    #[test]
    fn test_new_root_has_unassigned_bucket() {
        let card = Card::new_root("Root".to_string()).unwrap();
        assert_eq!(card.buckets().len(), 1);
        assert_eq!(card.buckets()[0].name(), UNASSIGNED_BUCKET_NAME);
    }

    #[test]
    fn test_new_child_has_parent_and_bucket() {
        let parent = Card::new_root("Parent".to_string()).unwrap();
        let bucket_id = parent.buckets()[0].id();
        let child = Card::new_child("Child".to_string(), parent.id(), bucket_id).unwrap();

        assert_eq!(child.parent_id(), Some(parent.id()));
        assert_eq!(child.bucket_id(), Some(bucket_id));
    }

    #[test]
    fn test_rename_rejects_blank_title() {
        let mut card = Card::new_root("Title".to_string()).unwrap();
        assert!(matches!(
            card.rename("  ".to_string()),
            Err(DomainError::EmptyTitle)
        ));
        assert_eq!(
            card.title(),
            "Title",
            "Title must be unchanged after a failed rename"
        );
    }

    #[test]
    fn test_add_duplicate_bucket_fails() {
        let mut card = Card::new_root("Project".to_string()).unwrap();
        assert!(card.add_bucket("In Progress".to_string()).is_ok());
        assert!(matches!(
            card.add_bucket("In Progress".to_string()),
            Err(DomainError::DuplicateBucketName(_))
        ));
    }

    #[test]
    fn test_remove_unassigned_bucket_fails() {
        let mut card = Card::new_root("Project".to_string()).unwrap();
        let unassigned_id = card.buckets()[0].id();
        assert!(matches!(
            card.remove_bucket(unassigned_id),
            Err(DomainError::InvalidOperation(_))
        ));
    }

    #[test]
    fn test_reorder_buckets() {
        let mut card = Card::new_root("Project".to_string()).unwrap();
        let id_a = card.add_bucket("Alpha".to_string()).unwrap();
        let id_b = card.add_bucket("Beta".to_string()).unwrap();
        let unassigned_id = card.buckets()[0].id();

        // Reorder: Beta, Unassigned, Alpha
        assert!(
            card.reorder_buckets(vec![id_b, unassigned_id, id_a])
                .is_ok()
        );
        assert_eq!(card.buckets()[0].name(), "Beta");
        assert_eq!(card.buckets()[1].name(), UNASSIGNED_BUCKET_NAME);
        assert_eq!(card.buckets()[2].name(), "Alpha");
    }

    #[test]
    fn test_reorder_buckets_fails_duplicates_and_unknowns() {
        let mut card = Card::new_root("Project".to_string()).unwrap();
        let id_a = card.add_bucket("Alpha".to_string()).unwrap();
        let _id_b = card.add_bucket("Beta".to_string()).unwrap();
        let unassigned_id = card.buckets()[0].id();

        // Fails Duplicate
        assert!(matches!(
            card.reorder_buckets(vec![id_a, id_a, unassigned_id]),
            Err(DomainError::DuplicateBucketId(_))
        ));

        // Fails Unknown
        assert!(matches!(
            card.reorder_buckets(vec![id_a, BucketId::new(), unassigned_id]),
            Err(DomainError::BucketNotFound(_))
        ));
    }

    #[test]
    fn test_new_card_rejects_blank_title() {
        assert!(matches!(
            Card::new_root("   ".to_string()),
            Err(DomainError::EmptyTitle)
        ));
    }

    #[test]
    fn test_note_page_lifecycle() {
        let mut card = Card::new_root("Root".to_string()).unwrap();
        let note_id = card.add_note_page("Ideas".to_string()).unwrap();
        card.save_note_page_body(note_id, "hello".to_string())
            .unwrap();
        card.rename_note_page(note_id, "Refined".to_string())
            .unwrap();

        assert_eq!(card.notes()[0].title(), "Refined");
        assert_eq!(card.notes()[0].body(), "hello");

        card.delete_note_page(note_id).unwrap();
        assert!(card.notes().is_empty());
    }
}
