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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_creation() {
        let rule = RuleDefinition::new(
            "My Rule".into(),
            RuleTrigger::NoteOpened,
            RuleAction::ShowPopup {
                title: "T".into(),
                message: "M".into(),
            },
        )
        .unwrap();
        assert_eq!(rule.name(), "My Rule");
        assert_eq!(rule.trigger(), &RuleTrigger::NoteOpened);
    }

    #[test]
    fn test_rule_rejects_blank_name() {
        let result = RuleDefinition::new(
            "  ".into(),
            RuleTrigger::NoteOpened,
            RuleAction::ShowPopup {
                title: "T".into(),
                message: "M".into(),
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_rule_rejects_empty_popup_fields() {
        let result = RuleDefinition::new(
            "Rule".into(),
            RuleTrigger::NoteOpened,
            RuleAction::ShowPopup {
                title: "".into(),
                message: "Valid".into(),
            },
        );
        assert!(result.is_err());

        let result = RuleDefinition::new(
            "Rule".into(),
            RuleTrigger::NoteOpened,
            RuleAction::ShowPopup {
                title: "Valid".into(),
                message: "   ".into(),
            },
        );
        assert!(result.is_err());
    }
}
