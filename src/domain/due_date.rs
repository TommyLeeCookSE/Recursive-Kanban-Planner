//! Logic for handling card due dates.
//!
//! This module provides the `DueDate` type, which ensures that dates are stored in a
//! consistent ISO 8601 format (YYYY-MM-DD). It handles parsing, formatting, and
//! overdue calculations.
//!
//! For more on how Rust handles strings and date types compared to Python's `datetime`,
//! see `docs/rust-for-python-devs.md`.

use crate::domain::error::DomainError;
use chrono::{DateTime, Local, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A validated date in YYYY-MM-DD format.
///
/// # Examples
///
/// ```
/// use kanban_planner::domain::due_date::DueDate;
///
/// let due = DueDate::parse("2025-12-31").unwrap();
/// assert_eq!(due.as_str(), "2025-12-31");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DueDate(String);

impl DueDate {
    /// Parses a date string into a `DueDate`.
    ///
    /// The string must be in YYYY-MM-DD format.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::due_date::DueDate;
    ///
    /// let due = DueDate::parse("2025-01-01").unwrap();
    /// ```
    pub fn parse(raw: impl Into<String>) -> Result<Self, DomainError> {
        let raw = raw.into();
        let parsed = NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d").map_err(|_| {
            DomainError::InvalidOperation(
                "Due date must be a valid date in YYYY-MM-DD format".to_string(),
            )
        })?;
        Ok(Self(parsed.format("%Y-%m-%d").to_string()))
    }

    /// Creates a `DueDate` from a UTC datetime.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::due_date::DueDate;
    /// use chrono::Utc;
    ///
    /// let due = DueDate::from_utc_datetime(Utc::now());
    /// ```
    pub fn from_utc_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt.date_naive().format("%Y-%m-%d").to_string())
    }

    /// Returns the date as a string slice in YYYY-MM-DD format.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::due_date::DueDate;
    ///
    /// let due = DueDate::parse("2025-01-01").unwrap();
    /// assert_eq!(due.as_str(), "2025-01-01");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Checks if the due date is before the current local date.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::due_date::DueDate;
    ///
    /// let due = DueDate::parse("2000-01-01").unwrap();
    /// assert!(due.is_overdue());
    /// ```
    pub fn is_overdue(&self) -> bool {
        self.is_overdue_on(current_local_date())
    }

    /// Checks if the due date is before a specific reference date string.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::due_date::DueDate;
    ///
    /// let due = DueDate::parse("2025-01-01").unwrap();
    /// assert!(due.is_overdue_on("2025-01-02"));
    /// assert!(!due.is_overdue_on("2025-01-01"));
    /// ```
    pub fn is_overdue_on(&self, today: impl AsRef<str>) -> bool {
        self.0.as_str() < today.as_ref()
    }
}

impl fmt::Display for DueDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Returns the current local date as a YYYY-MM-DD string.
///
/// # Examples
///
/// ```
/// use kanban_planner::domain::due_date::current_local_date;
///
/// let today = current_local_date();
/// ```
pub fn current_local_date() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_date_rejects_invalid_strings() {
        assert!(matches!(
            DueDate::parse("2026-02-30"),
            Err(DomainError::InvalidOperation(_))
        ));
    }

    #[test]
    fn due_date_normalizes_valid_input() {
        let due_date = DueDate::parse("2026-03-18").unwrap();
        assert_eq!(due_date.as_str(), "2026-03-18");
    }

    #[test]
    fn due_date_overdue_check_is_deterministic() {
        let due_date = DueDate::parse("2026-03-18").unwrap();
        assert!(due_date.is_overdue_on("2026-03-19"));
        assert!(!due_date.is_overdue_on("2026-03-18"));
        assert!(!due_date.is_overdue_on("2026-03-17"));
    }
}
