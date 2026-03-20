use crate::domain::error::DomainError;
use crate::domain::id::NotePageId;
use crate::domain::title::normalize_non_empty_title;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotePage {
    id: NotePageId,
    title: String,
    body: String,
}

impl NotePage {
    pub fn new(title: String) -> Result<Self, DomainError> {
        let title = normalize_non_empty_title(title)?;

        Ok(Self {
            id: NotePageId::new(),
            title,
            body: String::new(),
        })
    }

    pub fn id(&self) -> NotePageId {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn rename(&mut self, title: String) -> Result<(), DomainError> {
        self.title = normalize_non_empty_title(title)?;
        Ok(())
    }

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
