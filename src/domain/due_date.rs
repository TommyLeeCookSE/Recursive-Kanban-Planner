use crate::domain::error::DomainError;
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DueDate(String);

impl DueDate {
    pub fn parse(raw: impl Into<String>) -> Result<Self, DomainError> {
        let raw = raw.into();
        let parsed = NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d").map_err(|_| {
            DomainError::InvalidOperation(
                "Due date must be a valid date in YYYY-MM-DD format".to_string(),
            )
        })?;
        Ok(Self(parsed.format("%Y-%m-%d").to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_overdue(&self) -> bool {
        self.is_overdue_on(current_local_date())
    }

    pub fn is_overdue_on(&self, today: impl AsRef<str>) -> bool {
        self.0.as_str() < today.as_ref()
    }
}

impl fmt::Display for DueDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
