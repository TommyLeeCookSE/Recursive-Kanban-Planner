//! Utilities for normalizing and validating titles.
//!
//! This module provides shared logic for ensuring that cards and note pages
//! have valid, non-empty titles.
//!
//! For a discussion on Rust's `String` versus Python's `str`,
//! see `docs/rust-for-python-devs.md`.

use crate::domain::error::DomainError;

/// Validates and trims a title, ensuring it is not empty.
///
/// Returns the trimmed `String` if valid, or a `DomainError::EmptyTitle` if
/// the input is empty or contains only whitespace.
///
/// # Examples
///
/// ```
/// use kanban_planner::domain::title::normalize_non_empty_title;
///
/// let title = normalize_non_empty_title("  My Task  ").unwrap();
/// assert_eq!(title, "My Task");
/// ```
pub fn normalize_non_empty_title(title: impl Into<String>) -> Result<String, DomainError> {
    let title = title.into();
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(DomainError::EmptyTitle);
    }

    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_non_empty_title_rejects_blank_values() {
        assert!(matches!(
            normalize_non_empty_title("   "),
            Err(DomainError::EmptyTitle)
        ));
    }

    #[test]
    fn normalize_non_empty_title_trims_whitespace() {
        assert_eq!(
            normalize_non_empty_title("  Title  ").unwrap(),
            "Title".to_string()
        );
    }
}
