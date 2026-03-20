//! # Card Registry Domain
//!
//! The registry is the central authority for managing the lifecycle, relationships,
//! and persistence of all cards in the planner.
//!
//! It ensures that structural invariants—such as ensuring exactly one root workspace exists
//! and preventing circular parent-child relationships—are maintained across all operations.
//!
//! See `docs/rust-for-python-devs.md` for a guide on how this module maps to Python concepts.

use crate::domain::card::Card;
use crate::domain::due_date::DueDate;
use crate::domain::error::DomainError;
use crate::domain::id::{CardId, NotePageId};
use std::collections::HashMap;
mod mutations;
mod traversal;
mod validation;
mod workspace;

use serde::{Deserialize, Serialize};

/// The central state container for all cards in a workspace.
///
/// It provides a high-level API for mutating the card tree while maintaining
/// domain invariants.
///
/// # Examples
///
/// ```
/// use kanban_planner::domain::registry::CardRegistry;
///
/// let registry = CardRegistry::new();
/// assert_eq!(registry.workspace_child_count(), 0);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardRegistry {
    store: HashMap<CardId, Card>,
}

/// Strategies for handling children when a parent card is deleted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeleteStrategy {
    /// Refuse to delete if children exist.
    Reject,
    /// Delete the card and all its descendants.
    CascadeDelete,
    /// Delete the card and move its children to its parent.
    ReparentToGrandparent,
}

impl Default for CardRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CardRegistry {
    /// Creates a new registry initialized with a single root "My Workspace" card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let registry = CardRegistry::new();
    /// ```
    pub fn new() -> Self {
        let workspace = Card::new_root("My Workspace".to_string())
            .expect("workspace title should always be valid");

        Self {
            store: HashMap::from([(workspace.id(), workspace)]),
        }
    }

    /// Retrieves a reference to a card by its ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let registry = CardRegistry::new();
    /// let id = registry.workspace_card_id().unwrap();
    /// let card = registry.get_card(id).unwrap();
    /// assert_eq!(card.title(), "My Workspace");
    /// ```
    pub fn get_card(&self, id: CardId) -> Result<&Card, DomainError> {
        self.store.get(&id).ok_or(DomainError::CardNotFound(id))
    }

    /// Retrieves a mutable reference to a card by its ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let mut registry = CardRegistry::new();
    /// let id = registry.workspace_card_id().unwrap();
    /// let card = registry.get_card_mut(id).unwrap();
    /// card.rename("New Title".to_string()).unwrap();
    /// ```
    pub fn get_card_mut(&mut self, id: CardId) -> Result<&mut Card, DomainError> {
        self.store.get_mut(&id).ok_or(DomainError::CardNotFound(id))
    }

    /// Returns a reference to the root workspace card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let registry = CardRegistry::new();
    /// let workspace = registry.workspace_card().unwrap();
    /// assert_eq!(workspace.title(), "My Workspace");
    /// ```
    pub fn workspace_card(&self) -> Result<&Card, DomainError> {
        workspace::workspace_card(self)
    }

    /// Returns the ID of the root workspace card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let registry = CardRegistry::new();
    /// let id = registry.workspace_card_id().unwrap();
    /// ```
    pub fn workspace_card_id(&self) -> Result<CardId, DomainError> {
        workspace::workspace_card_id(self)
    }

    /// Returns a list of references to the immediate children of a parent card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let mut registry = CardRegistry::new();
    /// let workspace_id = registry.workspace_card_id().unwrap();
    /// registry.create_child_card("Child".to_string(), workspace_id).unwrap();
    ///
    /// let children = registry.get_children(workspace_id).unwrap();
    /// assert_eq!(children.len(), 1);
    /// ```
    pub fn get_children(&self, parent_id: CardId) -> Result<Vec<&Card>, DomainError> {
        let parent = self.get_card(parent_id)?;
        let mut children = Vec::with_capacity(parent.children_ids().len());
        for child_id in parent.children_ids() {
            children.push(self.get_card(*child_id)?);
        }
        Ok(children)
    }

    /// Returns the number of immediate children in the workspace.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let registry = CardRegistry::new();
    /// assert_eq!(registry.workspace_child_count(), 0);
    /// ```
    pub fn workspace_child_count(&self) -> usize {
        workspace::workspace_child_count(self)
    }

    /// Validates the internal consistency of the registry.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let registry = CardRegistry::new();
    /// registry.validate().unwrap();
    /// ```
    pub fn validate(&self) -> Result<(), DomainError> {
        validation::validate_registry(self)
    }

    /// Creates a new card as a direct child of the root workspace.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let mut registry = CardRegistry::new();
    /// let id = registry.create_workspace_child_card("Project".to_string()).unwrap();
    /// ```
    pub fn create_workspace_child_card(&mut self, title: String) -> Result<CardId, DomainError> {
        mutations::create_workspace_child_card(self, title)
    }

    /// Creates a new card as a child of the specified parent.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let mut registry = CardRegistry::new();
    /// let workspace_id = registry.workspace_card_id().unwrap();
    /// let id = registry.create_child_card("Project".to_string(), workspace_id).unwrap();
    /// ```
    pub fn create_child_card(
        &mut self,
        title: String,
        parent_id: CardId,
    ) -> Result<CardId, DomainError> {
        mutations::create_child_card(self, title, parent_id)
    }

    /// Renames an existing card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let mut registry = CardRegistry::new();
    /// let id = registry.workspace_card_id().unwrap();
    /// registry.rename_card(id, "New Name".to_string()).unwrap();
    /// ```
    pub fn rename_card(&mut self, id: CardId, title: String) -> Result<(), DomainError> {
        mutations::rename_card(self, id, title)
    }

    /// Adds a new note page to a card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let mut registry = CardRegistry::new();
    /// let card_id = registry.workspace_card_id().unwrap();
    /// let note_id = registry.add_note_page(card_id, "Notes".to_string()).unwrap();
    /// ```
    pub fn add_note_page(
        &mut self,
        card_id: CardId,
        title: String,
    ) -> Result<NotePageId, DomainError> {
        mutations::add_note_page(self, card_id, title)
    }

    /// Renames a note page on a card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let mut registry = CardRegistry::new();
    /// let card_id = registry.workspace_card_id().unwrap();
    /// let note_id = registry.add_note_page(card_id, "Notes".to_string()).unwrap();
    /// registry.rename_note_page(card_id, note_id, "Ideas".to_string()).unwrap();
    /// ```
    pub fn rename_note_page(
        &mut self,
        card_id: CardId,
        note_page_id: NotePageId,
        title: String,
    ) -> Result<(), DomainError> {
        mutations::rename_note_page(self, card_id, note_page_id, title)
    }

    /// Saves the body content of a note page.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let mut registry = CardRegistry::new();
    /// let card_id = registry.workspace_card_id().unwrap();
    /// let note_id = registry.add_note_page(card_id, "Notes".to_string()).unwrap();
    /// registry.save_note_page_body(card_id, note_id, "Content".to_string()).unwrap();
    /// ```
    pub fn save_note_page_body(
        &mut self,
        card_id: CardId,
        note_page_id: NotePageId,
        body: String,
    ) -> Result<(), DomainError> {
        mutations::save_note_page_body(self, card_id, note_page_id, body)
    }

    /// Deletes a note page from a card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let mut registry = CardRegistry::new();
    /// let card_id = registry.workspace_card_id().unwrap();
    /// let note_id = registry.add_note_page(card_id, "Notes".to_string()).unwrap();
    /// registry.delete_note_page(card_id, note_id).unwrap();
    /// ```
    pub fn delete_note_page(
        &mut self,
        card_id: CardId,
        note_page_id: NotePageId,
    ) -> Result<(), DomainError> {
        mutations::delete_note_page(self, card_id, note_page_id)
    }

    /// Sets the due date for a card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    /// use kanban_planner::domain::due_date::DueDate;
    ///
    /// let mut registry = CardRegistry::new();
    /// let card_id = registry.workspace_card_id().unwrap();
    /// let due_date = DueDate::parse("2023-12-31").unwrap();
    /// registry.set_due_date(card_id, due_date).unwrap();
    /// ```
    pub fn set_due_date(&mut self, card_id: CardId, due_date: DueDate) -> Result<(), DomainError> {
        mutations::set_due_date(self, card_id, due_date)
    }

    /// Clears the due date for a card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let mut registry = CardRegistry::new();
    /// let card_id = registry.workspace_card_id().unwrap();
    /// registry.clear_due_date(card_id).unwrap();
    /// ```
    pub fn clear_due_date(&mut self, card_id: CardId) -> Result<(), DomainError> {
        mutations::clear_due_date(self, card_id)
    }

    /// Reorders the children of a parent card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let mut registry = CardRegistry::new();
    /// let workspace_id = registry.workspace_card_id().unwrap();
    /// let c1 = registry.create_child_card("C1".to_string(), workspace_id).unwrap();
    /// let c2 = registry.create_child_card("C2".to_string(), workspace_id).unwrap();
    /// registry.reorder_children(workspace_id, vec![c2, c1]).unwrap();
    /// ```
    pub fn reorder_children(
        &mut self,
        parent_id: CardId,
        ordered_ids: Vec<CardId>,
    ) -> Result<(), DomainError> {
        mutations::reorder_children(self, parent_id, ordered_ids)
    }

    /// Moves a child card to a specific index in the parent's children list.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let mut registry = CardRegistry::new();
    /// let workspace_id = registry.workspace_card_id().unwrap();
    /// let c1 = registry.create_child_card("C1".to_string(), workspace_id).unwrap();
    /// let c2 = registry.create_child_card("C2".to_string(), workspace_id).unwrap();
    /// registry.drop_child_at_position(workspace_id, c2, 0).unwrap();
    /// ```
    pub fn drop_child_at_position(
        &mut self,
        parent_id: CardId,
        card_id: CardId,
        target_index: usize,
    ) -> Result<(), DomainError> {
        mutations::drop_child_at_position(self, parent_id, card_id, target_index)
    }

    /// Changes the parent of a card.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::CardRegistry;
    ///
    /// let mut registry = CardRegistry::new();
    /// let workspace_id = registry.workspace_card_id().unwrap();
    /// let p1 = registry.create_child_card("P1".to_string(), workspace_id).unwrap();
    /// let c1 = registry.create_child_card("C1".to_string(), workspace_id).unwrap();
    /// registry.reparent_card(c1, p1).unwrap();
    /// ```
    pub fn reparent_card(
        &mut self,
        card_id: CardId,
        new_parent_id: CardId,
    ) -> Result<(), DomainError> {
        mutations::reparent_card(self, card_id, new_parent_id)
    }

    /// Deletes a card using the specified strategy.
    ///
    /// # Examples
    ///
    /// ```
    /// use kanban_planner::domain::registry::{CardRegistry, DeleteStrategy};
    ///
    /// let mut registry = CardRegistry::new();
    /// let workspace_id = registry.workspace_card_id().unwrap();
    /// let c1 = registry.create_child_card("C1".to_string(), workspace_id).unwrap();
    /// registry.delete_card(c1, DeleteStrategy::CascadeDelete).unwrap();
    /// ```
    pub fn delete_card(
        &mut self,
        card_id: CardId,
        strategy: DeleteStrategy,
    ) -> Result<(), DomainError> {
        mutations::delete_card(self, card_id, strategy)
    }
}

pub(super) fn corrupt_state(message: impl Into<String>) -> DomainError {
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
    fn test_drop_child_at_position_reorders_existing_child() {
        let mut registry = CardRegistry::new();
        let workspace_id = registry.workspace_card_id().unwrap();
        let first = registry
            .create_child_card("First".into(), workspace_id)
            .unwrap();
        let second = registry
            .create_child_card("Second".into(), workspace_id)
            .unwrap();
        let third = registry
            .create_child_card("Third".into(), workspace_id)
            .unwrap();

        registry
            .drop_child_at_position(workspace_id, third, 0)
            .unwrap();

        let children: Vec<CardId> = registry
            .get_children(workspace_id)
            .unwrap()
            .iter()
            .map(|card| card.id())
            .collect();
        assert_eq!(children, vec![third, first, second]);
    }

    #[test]
    fn test_drop_child_at_position_rejects_non_child_card() {
        let mut registry = CardRegistry::new();
        let workspace_id = registry.workspace_card_id().unwrap();
        let project_id = registry
            .create_child_card("Project".into(), workspace_id)
            .unwrap();
        let task_id = registry
            .create_child_card("Task".into(), project_id)
            .unwrap();

        assert!(matches!(
            registry.drop_child_at_position(workspace_id, task_id, 0),
            Err(DomainError::InvalidOperation(message))
                if message.contains("is not a child of parent")
        ));
    }

    #[test]
    fn test_drop_child_at_position_clamps_target_index_to_end() {
        let mut registry = CardRegistry::new();
        let workspace_id = registry.workspace_card_id().unwrap();
        let first = registry
            .create_child_card("First".into(), workspace_id)
            .unwrap();
        let second = registry
            .create_child_card("Second".into(), workspace_id)
            .unwrap();
        let third = registry
            .create_child_card("Third".into(), workspace_id)
            .unwrap();

        registry
            .drop_child_at_position(workspace_id, first, usize::MAX)
            .unwrap();

        let children: Vec<CardId> = registry
            .get_children(workspace_id)
            .unwrap()
            .iter()
            .map(|card| card.id())
            .collect();
        assert_eq!(children, vec![second, third, first]);
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
