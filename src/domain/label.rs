use crate::domain::error::DomainError;
use crate::domain::id::LabelId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LabelColor {
    Ember,
    Gold,
    Moss,
    Sky,
    Indigo,
    Rose,
}

impl LabelColor {
    pub const ALL: [LabelColor; 6] = [
        LabelColor::Ember,
        LabelColor::Gold,
        LabelColor::Moss,
        LabelColor::Sky,
        LabelColor::Indigo,
        LabelColor::Rose,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            LabelColor::Ember => "Ember",
            LabelColor::Gold => "Gold",
            LabelColor::Moss => "Moss",
            LabelColor::Sky => "Sky",
            LabelColor::Indigo => "Indigo",
            LabelColor::Rose => "Rose",
        }
    }

    pub fn palette(&self) -> (&'static str, &'static str) {
        match self {
            LabelColor::Ember => ("rgba(251,146,60,0.18)", "#c2410c"),
            LabelColor::Gold => ("rgba(250,204,21,0.2)", "#854d0e"),
            LabelColor::Moss => ("rgba(74,222,128,0.18)", "#166534"),
            LabelColor::Sky => ("rgba(56,189,248,0.18)", "#0369a1"),
            LabelColor::Indigo => ("rgba(129,140,248,0.18)", "#4338ca"),
            LabelColor::Rose => ("rgba(251,113,133,0.18)", "#be123c"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelDefinition {
    id: LabelId,
    name: String,
    color: LabelColor,
}

impl LabelDefinition {
    pub fn new(name: String, color: LabelColor) -> Result<Self, DomainError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(DomainError::InvalidOperation(
                "Label name cannot be empty or blank".to_string(),
            ));
        }

        Ok(Self {
            id: LabelId::new(),
            name: trimmed.to_string(),
            color,
        })
    }

    pub fn id(&self) -> LabelId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn color(&self) -> LabelColor {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_creation() {
        let label = LabelDefinition::new("Urgent".into(), LabelColor::Rose).unwrap();
        assert_eq!(label.name(), "Urgent");
        assert_eq!(label.color(), LabelColor::Rose);
    }

    #[test]
    fn test_label_rejects_blank_name() {
        let result = LabelDefinition::new("   ".into(), LabelColor::Ember);
        assert!(result.is_err());
    }

    #[test]
    fn test_label_color_as_str() {
        assert_eq!(LabelColor::Ember.as_str(), "Ember");
        assert_eq!(LabelColor::Rose.as_str(), "Rose");
    }

    #[test]
    fn test_label_color_palette() {
        let (bg, text) = LabelColor::Sky.palette();
        assert!(bg.contains("rgba"));
        assert!(text.starts_with("#"));
    }
}
