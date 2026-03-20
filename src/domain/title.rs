use crate::domain::error::DomainError;

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
