//! Shared data models for the user interface.
//!
//! This module defines simplified views of domain data that are optimized
//! for rendering in UI components.
//!
//! For more on how Rust's data structures compare to Python's classes,
//! see `docs/rust-for-python-devs.md`.

use crate::application::CardPreviewView;
use crate::domain::card::Card;
use crate::domain::id::CardId;

/// A simplified view of a card's data for rendering in the UI.
///
/// # Examples
///
/// ```ignore
/// use crate::interface::components::visuals::CardDisplayData;
/// use crate::domain::id::CardId;
///
/// let data = CardDisplayData {
///     id: CardId::new(),
///     title: "Fix bug".to_string(),
///     due_date: Some("2023-12-31".to_string()),
///     is_overdue: false,
///     preview_items: vec!["Task 1".to_string()],
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardDisplayData {
    /// Unique identifier for the card.
    pub id: CardId,
    /// The card's title.
    pub title: String,
    /// Optional formatted due date string.
    pub due_date: Option<String>,
    /// Whether the card's due date has passed.
    pub is_overdue: bool,
    /// Titles of immediate child cards for a quick preview.
    pub preview_items: Vec<String>,
}

/// Transforms a domain `Card` into `CardDisplayData`.
///
/// # Examples
///
/// ```ignore
/// let display_data = build_card_display(&card, Some(&preview_view));
/// ```
pub fn build_card_display(card: &Card, preview_view: Option<&CardPreviewView>) -> CardDisplayData {
    let preview_items = preview_view
        .map(|view| {
            view.children
                .iter()
                .map(|child| child.title().to_string())
                .collect()
        })
        .unwrap_or_default();

    CardDisplayData {
        id: card.id(),
        title: card.title().to_string(),
        due_date: card.due_date().map(|due| due.to_string()),
        is_overdue: card.due_date().map(|due| due.is_overdue()).unwrap_or(false),
        preview_items,
    }
}
