use crate::domain::card::Card;
use crate::domain::due_date::DueDate;
use crate::domain::error::DomainError;
use crate::domain::id::{CardId, NotePageId};
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardRegistry {
    store: HashMap<CardId, Card>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeleteStrategy {
    Reject,
    CascadeDelete,
    ReparentToGrandparent,
}

impl Default for CardRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CardRegistry {
    pub fn new() -> Self {
        let workspace = Card::new_root("My Workspace".to_string())
            .expect("workspace title should always be valid");

        Self {
            store: HashMap::from([(workspace.id(), workspace)]),
        }
    }

    pub fn get_card(&self, id: CardId) -> Result<&Card, DomainError> {
        self.store.get(&id).ok_or(DomainError::CardNotFound(id))
    }

    pub fn get_card_mut(&mut self, id: CardId) -> Result<&mut Card, DomainError> {
        self.store.get_mut(&id).ok_or(DomainError::CardNotFound(id))
    }

    pub fn workspace_card(&self) -> Result<&Card, DomainError> {
        let mut top_level_cards = self
            .store
            .values()
            .filter(|card| card.parent_id().is_none());
        let workspace = top_level_cards.next().ok_or_else(|| {
            DomainError::InvalidOperation("Workspace card is missing from the registry".into())
        })?;

        if top_level_cards.next().is_some() {
            return Err(corrupt_state(
                "Registry contains multiple workspace cards".to_string(),
            ));
        }

        Ok(workspace)
    }

    pub fn workspace_card_id(&self) -> Result<CardId, DomainError> {
        Ok(self.workspace_card()?.id())
    }

    pub fn get_children(&self, parent_id: CardId) -> Result<Vec<&Card>, DomainError> {
        let parent = self.get_card(parent_id)?;
        let mut children = Vec::with_capacity(parent.children_ids().len());
        for child_id in parent.children_ids() {
            children.push(self.get_card(*child_id)?);
        }
        Ok(children)
    }

    pub fn workspace_child_count(&self) -> usize {
        self.workspace_card()
            .map(|workspace| workspace.children_ids().len())
            .unwrap_or(0)
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        let mut referenced_children = HashSet::new();
        let mut top_level_cards = HashSet::new();

        for (card_id, card) in &self.store {
            if card.id() != *card_id {
                return Err(corrupt_state(format!(
                    "Registry key {card_id} does not match card id {}",
                    card.id()
                )));
            }

            if card.title().trim().is_empty() {
                return Err(corrupt_state(format!(
                    "Card {card_id} has a blank title in persisted state"
                )));
            }

            validate_note_pages(card)?;

            if let Some(parent_id) = card.parent_id() {
                let parent = self.store.get(&parent_id).ok_or_else(|| {
                    corrupt_state(format!(
                        "Card {card_id} references missing parent {parent_id}"
                    ))
                })?;

                if !parent.children_ids().contains(card_id) {
                    return Err(corrupt_state(format!(
                        "Parent {parent_id} is missing child reference to {card_id}"
                    )));
                }
            }

            let mut local_children = HashSet::new();
            for child_id in card.children_ids() {
                if *child_id == *card_id {
                    return Err(corrupt_state(format!(
                        "Card {card_id} cannot reference itself as a child"
                    )));
                }

                if !local_children.insert(*child_id) {
                    return Err(corrupt_state(format!(
                        "Card {card_id} contains duplicate child reference {child_id}"
                    )));
                }

                if !referenced_children.insert(*child_id) {
                    return Err(corrupt_state(format!(
                        "Child card {child_id} is referenced by more than one parent"
                    )));
                }

                let child = self.store.get(child_id).ok_or_else(|| {
                    corrupt_state(format!(
                        "Card {card_id} references missing child {child_id}"
                    ))
                })?;

                if child.parent_id() != Some(*card_id) {
                    return Err(corrupt_state(format!(
                        "Child {child_id} does not point back to parent {card_id}"
                    )));
                }
            }

            validate_parent_chain(self, *card_id)?;
        }

        for (card_id, card) in &self.store {
            match card.parent_id() {
                None => {
                    top_level_cards.insert(*card_id);
                    if referenced_children.contains(card_id) {
                        return Err(corrupt_state(format!(
                            "Workspace card {card_id} must not be referenced as a child"
                        )));
                    }
                }
                Some(_) => {
                    if !referenced_children.contains(card_id) {
                        return Err(corrupt_state(format!(
                            "Non-workspace card {card_id} is not referenced by its parent"
                        )));
                    }
                }
            }
        }

        if top_level_cards.len() != 1 {
            return Err(corrupt_state(format!(
                "Registry must contain exactly one workspace card, found {}",
                top_level_cards.len()
            )));
        }

        Ok(())
    }

    pub fn create_workspace_child_card(&mut self, title: String) -> Result<CardId, DomainError> {
        let workspace_id = self.workspace_card_id()?;
        self.create_child_card(title, workspace_id)
    }

    pub fn create_child_card(
        &mut self,
        title: String,
        parent_id: CardId,
    ) -> Result<CardId, DomainError> {
        self.get_card(parent_id)?;

        let child = Card::new_child(title, parent_id)?;
        let child_id = child.id();

        self.get_card_mut(parent_id)?.add_child(child_id);
        self.store.insert(child_id, child);
        Ok(child_id)
    }

    pub fn rename_card(&mut self, id: CardId, title: String) -> Result<(), DomainError> {
        self.get_card_mut(id)?.rename(title)
    }

    pub fn add_note_page(
        &mut self,
        card_id: CardId,
        title: String,
    ) -> Result<NotePageId, DomainError> {
        self.get_card_mut(card_id)?.add_note_page(title)
    }

    pub fn rename_note_page(
        &mut self,
        card_id: CardId,
        note_page_id: NotePageId,
        title: String,
    ) -> Result<(), DomainError> {
        self.get_card_mut(card_id)?
            .rename_note_page(note_page_id, title)
    }

    pub fn save_note_page_body(
        &mut self,
        card_id: CardId,
        note_page_id: NotePageId,
        body: String,
    ) -> Result<(), DomainError> {
        self.get_card_mut(card_id)?
            .save_note_page_body(note_page_id, body)
    }

    pub fn delete_note_page(
        &mut self,
        card_id: CardId,
        note_page_id: NotePageId,
    ) -> Result<(), DomainError> {
        self.get_card_mut(card_id)?.delete_note_page(note_page_id)
    }

    pub fn set_due_date(&mut self, card_id: CardId, due_date: DueDate) -> Result<(), DomainError> {
        self.get_card_mut(card_id)?.set_due_date(Some(due_date));
        Ok(())
    }

    pub fn clear_due_date(&mut self, card_id: CardId) -> Result<(), DomainError> {
        self.get_card_mut(card_id)?.set_due_date(None);
        Ok(())
    }

    pub fn reorder_children(
        &mut self,
        parent_id: CardId,
        ordered_ids: Vec<CardId>,
    ) -> Result<(), DomainError> {
        self.get_card_mut(parent_id)?.reorder_children(ordered_ids)
    }

    fn detect_cycle(&self, card_id: CardId, proposed_parent_id: CardId) -> Result<(), DomainError> {
        let mut current_ancestor_id = Some(proposed_parent_id);
        while let Some(ancestor_id) = current_ancestor_id {
            if ancestor_id == card_id {
                return Err(DomainError::CycleDetected);
            }

            current_ancestor_id = self.get_card(ancestor_id)?.parent_id();
        }

        Ok(())
    }

    pub fn reparent_card(
        &mut self,
        card_id: CardId,
        new_parent_id: CardId,
    ) -> Result<(), DomainError> {
        if card_id == new_parent_id {
            return Err(DomainError::CycleDetected);
        }

        self.detect_cycle(card_id, new_parent_id)?;

        let old_parent_id = self.get_card(card_id)?.parent_id();
        if old_parent_id == Some(new_parent_id) {
            return Ok(());
        }

        self.get_card(new_parent_id)?;

        self.get_card_mut(new_parent_id)?.add_child(card_id);

        if let Some(old_parent_id) = old_parent_id {
            self.get_card_mut(old_parent_id)?.remove_child(card_id);
        }

        self.get_card_mut(card_id)?.set_parent(Some(new_parent_id));
        Ok(())
    }

    pub fn delete_card(
        &mut self,
        card_id: CardId,
        strategy: DeleteStrategy,
    ) -> Result<(), DomainError> {
        let workspace_id = self.workspace_card_id()?;
        if card_id == workspace_id {
            return Err(DomainError::InvalidOperation(
                "The workspace card cannot be deleted.".into(),
            ));
        }

        let card = self.get_card(card_id)?;
        let children = card.children_ids().to_vec();
        let parent_id = card.parent_id();

        if !children.is_empty() {
            match strategy {
                DeleteStrategy::Reject => return Err(DomainError::CardHasChildren),
                DeleteStrategy::CascadeDelete => {
                    for child_id in children {
                        self.delete_card(child_id, DeleteStrategy::CascadeDelete)?;
                    }
                }
                DeleteStrategy::ReparentToGrandparent => {
                    let grandparent_id = parent_id.ok_or_else(|| {
                        DomainError::InvalidOperation(
                            "Cannot reparent to grandparent: card has no parent.".into(),
                        )
                    })?;
                    for child_id in children {
                        self.reparent_card(child_id, grandparent_id)?;
                    }
                }
            }
        }

        if let Some(parent_id) = parent_id {
            self.get_card_mut(parent_id)?.remove_child(card_id);
        }

        self.store.remove(&card_id);
        Ok(())
    }
}

fn validate_note_pages(card: &Card) -> Result<(), DomainError> {
    let mut ids = HashSet::new();
    for note in card.notes() {
        if note.title().trim().is_empty() {
            return Err(corrupt_state(format!(
                "Card {} contains a note page with a blank title",
                card.id()
            )));
        }

        if !ids.insert(note.id()) {
            return Err(corrupt_state(format!(
                "Card {} contains duplicate note page id {}",
                card.id(),
                note.id()
            )));
        }
    }
    Ok(())
}

fn validate_parent_chain(registry: &CardRegistry, card_id: CardId) -> Result<(), DomainError> {
    let mut current_parent = registry
        .store
        .get(&card_id)
        .ok_or_else(|| corrupt_state(format!("Card {card_id} is missing from the registry")))?
        .parent_id();
    let mut seen = HashSet::new();

    while let Some(parent_id) = current_parent {
        if !seen.insert(parent_id) {
            return Err(corrupt_state(format!(
                "Card {card_id} participates in a parent cycle involving {parent_id}"
            )));
        }

        let parent = registry.store.get(&parent_id).ok_or_else(|| {
            corrupt_state(format!(
                "Card {card_id} references missing ancestor {parent_id}"
            ))
        })?;

        current_parent = parent.parent_id();
    }

    Ok(())
}

fn corrupt_state(message: impl Into<String>) -> DomainError {
    DomainError::InvalidOperation(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_starts_with_workspace() {
        let registry = CardRegistry::new();
        let workspace = registry.workspace_card().unwrap();
        assert_eq!(workspace.title(), "My Workspace");
        assert!(workspace.parent_id().is_none());
    }

    #[test]
    fn test_create_child_card_adds_ordered_child() {
        let mut registry = CardRegistry::new();
        let workspace_id = registry.workspace_card_id().unwrap();
        let child_id = registry
            .create_child_card("Project".into(), workspace_id)
            .unwrap();

        let children = registry.get_children(workspace_id).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id(), child_id);
    }

    #[test]
    fn test_reorder_children_updates_order() {
        let mut registry = CardRegistry::new();
        let workspace_id = registry.workspace_card_id().unwrap();
        let first = registry
            .create_child_card("First".into(), workspace_id)
            .unwrap();
        let second = registry
            .create_child_card("Second".into(), workspace_id)
            .unwrap();

        registry
            .reorder_children(workspace_id, vec![second, first])
            .unwrap();

        let children: Vec<CardId> = registry
            .get_children(workspace_id)
            .unwrap()
            .iter()
            .map(|card| card.id())
            .collect();
        assert_eq!(children, vec![second, first]);
    }

    #[test]
    fn test_cycle_detection() {
        let mut registry = CardRegistry::new();
        let workspace_id = registry.workspace_card_id().unwrap();
        let child_id = registry
            .create_child_card("Child".into(), workspace_id)
            .unwrap();
        let grandchild_id = registry
            .create_child_card("Grandchild".into(), child_id)
            .unwrap();

        assert!(matches!(
            registry.reparent_card(child_id, grandchild_id),
            Err(DomainError::CycleDetected)
        ));
    }

    #[test]
    fn test_delete_workspace_is_rejected() {
        let mut registry = CardRegistry::new();
        let workspace_id = registry.workspace_card_id().unwrap();

        assert!(matches!(
            registry.delete_card(workspace_id, DeleteStrategy::CascadeDelete),
            Err(DomainError::InvalidOperation(_))
        ));
    }

    #[test]
    fn test_delete_strategy_reparent() {
        let mut registry = CardRegistry::new();
        let workspace_id = registry.workspace_card_id().unwrap();
        let parent_id = registry
            .create_child_card("Parent".into(), workspace_id)
            .unwrap();
        let child_id = registry
            .create_child_card("Child".into(), parent_id)
            .unwrap();

        registry
            .delete_card(parent_id, DeleteStrategy::ReparentToGrandparent)
            .unwrap();

        assert_eq!(
            registry.get_card(child_id).unwrap().parent_id(),
            Some(workspace_id)
        );
    }

    #[test]
    fn test_get_children_fails_on_missing_child() {
        let mut registry = CardRegistry::new();
        let workspace_id = registry.workspace_card_id().unwrap();
        registry
            .get_card_mut(workspace_id)
            .unwrap()
            .add_child(CardId::new());

        assert!(matches!(
            registry.get_children(workspace_id),
            Err(DomainError::CardNotFound(_))
        ));
    }

    #[test]
    fn test_validate_rejects_multiple_top_level_cards() {
        let mut registry = CardRegistry::new();
        let extra_top_level_card = Card::new_root("Extra".into()).unwrap();
        registry
            .store
            .insert(extra_top_level_card.id(), extra_top_level_card);

        assert!(matches!(
            registry.validate(),
            Err(DomainError::InvalidOperation(message))
                if message.contains("exactly one workspace card")
        ));
    }

    #[test]
    fn test_validate_rejects_orphan_nested_card() {
        let mut registry = CardRegistry::new();
        let workspace_id = registry.workspace_card_id().unwrap();
        let child = Card::new_child("Orphan".into(), workspace_id).unwrap();
        registry.store.insert(child.id(), child);

        assert!(matches!(
            registry.validate(),
            Err(DomainError::InvalidOperation(message))
                if message.contains("is not referenced by its parent")
                    || message.contains("missing child reference")
        ));
    }
}
