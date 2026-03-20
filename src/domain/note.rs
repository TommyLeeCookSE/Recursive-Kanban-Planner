//! Rich text notes associated with cards.
//!
//! Cards can contain multiple `NotePage`s, each with its own title and body.
//! This allows users to attach detailed documentation or logs to a planning item.
//!
//! For more on how Rust's `String` and ownership compare to Python's object
//! references, see `docs/rust-for-python-devs.md`.

use crate::domain::error::DomainError;
use crate::domain::id::NotePageId;
use crate::domain::title::normalize_non_empty_title;
use serde::{Deserialize, Serialize};

/// A named document attached to a card.
///
/// # Examples
///
/// ```
/// use kanban_planner::domain::note::NotePage;
///
/// let note = NotePage::new("Journal".to_string()).unwrap();
/// assert_eq!(note.title(), "Journal");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotePage {
    id: NotePageId,
    title: String,
    body: String,
}

impl NotePage {
    /// Creates a new `NotePage` with a non-empty title and an empty body.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::note::NotePage;
    ///
    /// let note = NotePage::new("Notes".into()).unwrap();
    /// ```
    pub fn new(title: String) -> Result<Self, DomainError> {
        let title = normalize_non_empty_title(title)?;

        Ok(Self {
            id: NotePageId::new(),
            title,
            body: String::new(),
        })
    }

    /// Returns the unique ID of the note page.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::note::NotePage;
    ///
    /// let note = NotePage::new("A".into()).unwrap();
    /// let id = note.id();
    /// ```
    pub fn id(&self) -> NotePageId {
        self.id
    }

    /// Returns a reference to the note page's title.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::note::NotePage;
    ///
    /// let note = NotePage::new("My Note".into()).unwrap();
    /// assert_eq!(note.title(), "My Note");
    /// ```
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns a reference to the note page's body content.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::note::NotePage;
    ///
    /// let note = NotePage::new("A".into()).unwrap();
    /// assert_eq!(note.body(), "");
    /// ```
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Updates the title of the note page.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::note::NotePage;
    ///
    /// let mut note = NotePage::new("Old".into()).unwrap();
    /// note.rename("New".into()).unwrap();
    /// ```
    pub fn rename(&mut self, title: String) -> Result<(), DomainError> {
        self.title = normalize_non_empty_title(title)?;
        Ok(())
    }

    /// Replaces the body content of the note page.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::note::NotePage;
    ///
    /// let mut note = NotePage::new("A".into()).unwrap();
    /// note.set_body("Some content".into());
    /// ```
    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_page_rejects_blank_title() {
        assert!(matches!(
            NotePage::new("  ".to_string()),
            Err(DomainError::EmptyTitle)
        ));
    }
}
