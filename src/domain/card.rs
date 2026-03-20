//! # Card Domain
//!
//! Cards are the primary structural building block of the Kanban Planner.
//! They represent tasks, projects, or any work item that can be nested.
//!
//! Every card may own ordered child cards, notes, and an optional due date.
//!
//! See `docs/rust-for-python-devs.md` for a guide on how this module maps to Python concepts.

use crate::domain::due_date::DueDate;
use crate::domain::error::DomainError;
use crate::domain::id::{CardId, NotePageId};
use crate::domain::note::NotePage;
use crate::domain::title::normalize_non_empty_title;

use serde::{Deserialize, Serialize};

/// Represents a single unit of work or a container for other work items.
///
/// Cards form a tree structure, where each card has a parent (except for the root
/// workspace) and zero or more children.
///
/// # Examples
///
/// ```
/// use kanban_planner::domain::card::Card;
///
/// let card = Card::new_root("My Project".to_string()).unwrap();
/// assert_eq!(card.title(), "My Project");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    id: CardId,
    title: String,
    parent_id: Option<CardId>,
    children_ids: Vec<CardId>,
    #[serde(default)]
    notes: Vec<NotePage>,
    #[serde(default)]
    due_date: Option<DueDate>,
}

impl Card {
    /// Creates a new root card (a workspace).
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::card::Card;
    ///
    /// let card = Card::new_root("Workspace".to_string()).unwrap();
    /// assert!(card.parent_id().is_none());
    /// ```
    pub fn new_root(title: String) -> Result<Self, DomainError> {
        let title = normalize_non_empty_title(title)?;

        Ok(Self {
            id: CardId::new(),
            title,
            parent_id: None,
            children_ids: Vec::new(),
            notes: Vec::new(),
            due_date: None,
        })
    }

    /// Creates a new child card with a specified parent.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::card::Card;
    /// use kanban_planner::domain::id::CardId;
    ///
    /// let parent_id = CardId::new();
    /// let card = Card::new_child("Task".to_string(), parent_id).unwrap();
    /// assert_eq!(card.parent_id(), Some(parent_id));
    /// ```
    pub fn new_child(title: String, parent_id: CardId) -> Result<Self, DomainError> {
        let title = normalize_non_empty_title(title)?;

        Ok(Self {
            id: CardId::new(),
            title,
            parent_id: Some(parent_id),
            children_ids: Vec::new(),
            notes: Vec::new(),
            due_date: None,
        })
    }

    /// Returns the unique identifier of the card.
    pub fn id(&self) -> CardId {
        self.id
    }

    /// Returns the current title of the card.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the ID of the parent card, if any.
    pub fn parent_id(&self) -> Option<CardId> {
        self.parent_id
    }

    /// Returns a slice of the IDs of the child cards.
    pub fn children_ids(&self) -> &[CardId] {
        &self.children_ids
    }

    /// Returns a slice of the note pages attached to this card.
    pub fn notes(&self) -> &[NotePage] {
        &self.notes
    }

    /// Returns the due date of the card, if set.
    pub fn due_date(&self) -> Option<&DueDate> {
        self.due_date.as_ref()
    }

    /// Renames the card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::card::Card;
    ///
    /// let mut card = Card::new_root("Old Title".to_string()).unwrap();
    /// card.rename("New Title".to_string()).unwrap();
    /// assert_eq!(card.title(), "New Title");
    /// ```
    pub fn rename(&mut self, new_title: String) -> Result<(), DomainError> {
        self.title = normalize_non_empty_title(new_title)?;
        Ok(())
    }

    /// Reorders the immediate children of this card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::card::Card;
    /// use kanban_planner::domain::id::CardId;
    ///
    /// let mut card = Card::new_root("Parent".to_string()).unwrap();
    /// // In practice, you'd add real children here.
    /// ```
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

    /// Adds a new note page to the card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::card::Card;
    ///
    /// let mut card = Card::new_root("Task".to_string()).unwrap();
    /// let note_id = card.add_note_page("Project Notes".to_string()).unwrap();
    /// ```
    pub fn add_note_page(&mut self, title: String) -> Result<NotePageId, DomainError> {
        let note = NotePage::new(title)?;
        let id = note.id();
        self.notes.push(note);
        Ok(id)
    }

    /// Renames a note page on this card.
    pub fn rename_note_page(&mut self, id: NotePageId, title: String) -> Result<(), DomainError> {
        let note = self
            .notes
            .iter_mut()
            .find(|note| note.id() == id)
            .ok_or_else(|| DomainError::InvalidOperation(format!("Note page not found: {id}")))?;
        note.rename(title)
    }

    /// Saves the body content of a note page.
    pub fn save_note_page_body(&mut self, id: NotePageId, body: String) -> Result<(), DomainError> {
        let note = self
            .notes
            .iter_mut()
            .find(|note| note.id() == id)
            .ok_or_else(|| DomainError::InvalidOperation(format!("Note page not found: {id}")))?;
        note.set_body(body);
        Ok(())
    }

    /// Deletes a note page from this card.
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

    /// Sets or clears the due date of the card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::card::Card;
    /// use kanban_planner::domain::due_date::DueDate;
    ///
    /// let mut card = Card::new_root("Task".to_string()).unwrap();
    /// let due = DueDate::parse("2023-12-31").unwrap();
    /// card.set_due_date(Some(due));
    /// ```
    pub fn set_due_date(&mut self, due_date: Option<DueDate>) {
        self.due_date = due_date;
    }

    pub(crate) fn add_child(&mut self, child_id: CardId) {
        self.children_ids.push(child_id);
    }

    pub(crate) fn remove_child(&mut self, child_id: CardId) {
        self.children_ids.retain(|id| *id != child_id);
    }

    pub(crate) fn set_parent(&mut self, parent_id: Option<CardId>) {
        self.parent_id = parent_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_root_has_no_parent() {
        let card = Card::new_root("Workspace".to_string()).unwrap();
        assert!(card.parent_id().is_none());
        assert!(card.children_ids().is_empty());
    }

    #[test]
    fn test_new_child_has_parent() {
        let parent = Card::new_root("Parent".to_string()).unwrap();
        let child = Card::new_child("Child".to_string(), parent.id()).unwrap();

        assert_eq!(child.parent_id(), Some(parent.id()));
    }

    #[test]
    fn test_rename_rejects_blank_title() {
        let mut card = Card::new_root("Title".to_string()).unwrap();
        assert!(matches!(
            card.rename("  ".to_string()),
            Err(DomainError::EmptyTitle)
        ));
        assert_eq!(card.title(), "Title");
    }

    #[test]
    fn test_reorder_children_rejects_duplicates() {
        let mut card = Card::new_root("Parent".to_string()).unwrap();
        let first = CardId::new();
        let second = CardId::new();
        card.add_child(first);
        card.add_child(second);

        assert!(matches!(
            card.reorder_children(vec![first, first]),
            Err(DomainError::InvalidOperation(_))
        ));
    }

    #[test]
    fn test_note_page_lifecycle() {
        let mut card = Card::new_root("Workspace".to_string()).unwrap();
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
