use crate::domain::error::DomainError;
use crate::domain::id::{BucketId, RuleId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleTrigger {
    NoteOpened,
    NoteClosed,
    MovedToBucket(BucketId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleAction {
    ShowPopup { title: String, message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleDefinition {
    id: RuleId,
    name: String,
    trigger: RuleTrigger,
    action: RuleAction,
}

impl RuleDefinition {
    pub fn new(
        name: String,
        trigger: RuleTrigger,
        action: RuleAction,
    ) -> Result<Self, DomainError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(DomainError::InvalidOperation(
                "Rule name cannot be empty or blank".to_string(),
            ));
        }

        match &action {
            RuleAction::ShowPopup { title, message } => {
                if title.trim().is_empty() || message.trim().is_empty() {
                    return Err(DomainError::InvalidOperation(
                        "Popup rules require both a title and message".to_string(),
                    ));
                }
            }
        }

        Ok(Self {
            id: RuleId::new(),
            name: trimmed.to_string(),
            trigger,
            action,
        })
    }

    pub fn id(&self) -> RuleId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn trigger(&self) -> &RuleTrigger {
        &self.trigger
    }

    pub fn action(&self) -> &RuleAction {
        &self.action
    }
}
