use crate::domain::card::{Card, UNASSIGNED_BUCKET_NAME};
use crate::domain::error::DomainError;
use crate::domain::id::{BucketId, CardId};
use std::collections::HashMap;

/// The central store managing all cards and enforcing cross-card, structural invariants.
pub struct CardRegistry {
    store: HashMap<CardId, Card>,
}

/// The strategy to use when deleting a card that has children.
pub enum DeleteStrategy {
    /// Reject the deletion if the card has any children.
    Reject,
    /// Recursively delete the card and all its descendants.
    CascadeDelete,
    /// Move all immediate children to the deleted card's parent before deletion.
    ReparentToGrandparent,
}

impl Default for CardRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CardRegistry {
    /// Creates a new, empty `CardRegistry`.
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    // -------------------------------------------------------------------------
    // Reads
    // -------------------------------------------------------------------------

    /// Retrieves a reference to a specific card.
    pub fn get_card(&self, id: CardId) -> Result<&Card, DomainError> {
        self.store.get(&id).ok_or(DomainError::CardNotFound(id))
    }

    /// Retrieves a mutable reference to a specific card.
    pub fn get_card_mut(&mut self, id: CardId) -> Result<&mut Card, DomainError> {
        self.store.get_mut(&id).ok_or(DomainError::CardNotFound(id))
    }

    /// Returns a list of all root cards (cards with no parent).
    pub fn get_root_cards(&self) -> Vec<&Card> {
        let mut roots: Vec<&Card> = self
            .store
            .values()
            .filter(|card| card.parent_id().is_none())
            .collect();
        // Since we don't have a stable root ordering yet, sort by ID for determinism
        roots.sort_by_key(|c| c.id());
        roots
    }

    /// Returns an ordered list of immediate children for a given parent card.
    pub fn get_children(&self, parent_id: CardId) -> Result<Vec<&Card>, DomainError> {
        let parent = self.get_card(parent_id)?;
        let mut children = Vec::with_capacity(parent.children_ids().len());
        for child_id in parent.children_ids() {
            let child = self.get_card(*child_id)?;
            children.push(child);
        }
        Ok(children)
    }

    /// Returns the children of a card grouped by their assigned buckets.
    ///
    /// # Errors
    /// - `DomainError::CardNotFound` if the parent card does not exist.
    /// - `DomainError::CardNotFound` if any child ID in the parent's `children_ids` list
    ///   is not present in the registry (corrupt state — should never occur via normal mutations).
    /// - `DomainError::InvalidOperation` if a child card has no `bucket_id` (root-like child).
    /// - `DomainError::BucketNotFound` if a child's `bucket_id` does not exist on the parent's
    ///   board (corrupt state — should never occur via normal mutations).
    pub fn board_projection(
        &self,
        card_id: CardId,
    ) -> Result<HashMap<BucketId, Vec<&Card>>, DomainError> {
        let parent = self.get_card(card_id)?;
        let mut projection: HashMap<BucketId, Vec<&Card>> = HashMap::new();

        // Initialize all valid buckets with an empty list
        for bucket in parent.buckets() {
            projection.insert(bucket.id(), Vec::new());
        }

        // Group children
        for child_id in parent.children_ids() {
            let child = self.get_card(*child_id)?;
            let b_id = child.bucket_id().ok_or_else(|| {
                DomainError::InvalidOperation(format!(
                    "Child card {child_id} is a child but has no bucket_id"
                ))
            })?;

            if let Some(cards) = projection.get_mut(&b_id) {
                cards.push(child);
            } else {
                return Err(DomainError::BucketNotFound(b_id));
            }
        }

        Ok(projection)
    }

    // -------------------------------------------------------------------------
    // Creation
    // -------------------------------------------------------------------------

    /// Creates and stores a new root card.
    pub fn create_root_card(&mut self, title: String) -> Result<CardId, DomainError> {
        let card = Card::new_root(title)?;
        let id = card.id();
        self.store.insert(id, card);
        Ok(id)
    }

    /// Creates and stores a new child card, assigning it to a specific parent and bucket.
    pub fn create_child_card(
        &mut self,
        title: String,
        parent_id: CardId,
        bucket_id: BucketId,
    ) -> Result<CardId, DomainError> {
        // Validate parent exists and has the requested bucket
        let parent = self.get_card(parent_id)?;
        if !parent.buckets().iter().any(|b| b.id() == bucket_id) {
            return Err(DomainError::BucketNotFound(bucket_id));
        }

        // Create the child
        let child = Card::new_child(title, parent_id, bucket_id)?;
        let child_id = child.id();

        // Enforce bidirectional relationship
        let parent_mut = self.get_card_mut(parent_id).unwrap(); // safe because we verified it above
        parent_mut.add_child(child_id);

        // Store the child
        self.store.insert(child_id, child);
        Ok(child_id)
    }

    // -------------------------------------------------------------------------
    // Single-card mutations (Delegation)
    // -------------------------------------------------------------------------

    pub fn rename_card(&mut self, id: CardId, title: String) -> Result<(), DomainError> {
        self.get_card_mut(id)?.rename(title)
    }

    pub fn add_bucket(&mut self, card_id: CardId, name: String) -> Result<BucketId, DomainError> {
        self.get_card_mut(card_id)?.add_bucket(name)
    }

    pub fn rename_bucket(
        &mut self,
        card_id: CardId,
        bucket_id: BucketId,
        new_name: String,
    ) -> Result<(), DomainError> {
        self.get_card_mut(card_id)?
            .rename_bucket(bucket_id, new_name)
    }

    pub fn reorder_buckets(
        &mut self,
        card_id: CardId,
        ordered_ids: Vec<BucketId>,
    ) -> Result<(), DomainError> {
        self.get_card_mut(card_id)?.reorder_buckets(ordered_ids)
    }

    pub fn reorder_children(
        &mut self,
        card_id: CardId,
        ordered_ids: Vec<CardId>,
    ) -> Result<(), DomainError> {
        self.get_card_mut(card_id)?.reorder_children(ordered_ids)
    }

    // -------------------------------------------------------------------------
    // Cross-card mutations
    // -------------------------------------------------------------------------

    /// Moves a child card to a different bucket within its current parent's board.
    pub fn move_card_to_bucket(
        &mut self,
        card_id: CardId,
        bucket_id: BucketId,
    ) -> Result<(), DomainError> {
        let card = self.get_card(card_id)?;
        let parent_id = card.parent_id().ok_or_else(|| {
            DomainError::InvalidOperation("Root cards do not have buckets".into())
        })?;

        // Verify the bucket exists in the parent
        let parent = self.get_card(parent_id)?;
        if !parent.buckets().iter().any(|b| b.id() == bucket_id) {
            return Err(DomainError::BucketNotFound(bucket_id));
        }

        // Assign
        self.get_card_mut(card_id)
            .unwrap()
            .assign_to_bucket(bucket_id);
        Ok(())
    }

    /// Removes a bucket from a card's board... but ONLY if no children are still in it.
    pub fn remove_bucket(
        &mut self,
        card_id: CardId,
        bucket_id: BucketId,
    ) -> Result<(), DomainError> {
        // First check children
        for child_id in self.get_card(card_id)?.children_ids().to_vec() {
            if let Ok(child) = self.get_card(child_id) {
                if child.bucket_id() == Some(bucket_id) {
                    return Err(DomainError::BucketNotEmpty);
                }
            }
        }

        // Delegate to remove
        self.get_card_mut(card_id)?.remove_bucket(bucket_id)
    }

    /// Internal cycle detection: walk up ancestry line to see if target parent is
    /// actually inside the card being moved.
    fn detect_cycle(&self, card_id: CardId, proposed_parent_id: CardId) -> Result<(), DomainError> {
        let mut current_ancestor_id = Some(proposed_parent_id);
        while let Some(ancestor_id) = current_ancestor_id {
            if ancestor_id == card_id {
                return Err(DomainError::CycleDetected);
            }
            // Move up one level
            if let Ok(ancestor) = self.get_card(ancestor_id) {
                current_ancestor_id = ancestor.parent_id();
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Safely reparent a card. Handles cycle checks, bidirectional tree updates,
    /// and bucket reassignments.
    pub fn reparent_card(
        &mut self,
        card_id: CardId,
        new_parent_id: CardId,
    ) -> Result<(), DomainError> {
        // Prevent reparenting self or cycle
        if card_id == new_parent_id {
            return Err(DomainError::CycleDetected);
        }
        self.detect_cycle(card_id, new_parent_id)?;

        // Verify card to move exists, and pull state
        let card = self.get_card(card_id)?;

        // Early return if new_parent_id is the current parent (no-op)
        if card.parent_id() == Some(new_parent_id) {
            return Ok(());
        }

        let old_parent_id = card.parent_id();

        // Update the new parent first (which acts as existence check)
        let unassigned_bucket_id = self
            .get_card(new_parent_id)?
            .buckets()
            .iter()
            .find(|b| b.name() == UNASSIGNED_BUCKET_NAME)
            .map(|b| b.id())
            .unwrap();

        self.get_card_mut(new_parent_id)?.add_child(card_id);

        // Remove from the old parent (if it had one)
        if let Some(old_p) = old_parent_id {
            if let Ok(p_mut) = self.get_card_mut(old_p) {
                p_mut.remove_child(card_id);
            }
        }

        // Update the card itself
        let card_mut = self.get_card_mut(card_id).unwrap();
        card_mut.set_parent(Some(new_parent_id));
        card_mut.set_bucket(Some(unassigned_bucket_id));

        Ok(())
    }

    /// Deletes a card using the specified strategy to handle any children.
    pub fn delete_card(
        &mut self,
        card_id: CardId,
        strategy: DeleteStrategy,
    ) -> Result<(), DomainError> {
        let card = self.get_card(card_id)?;
        let children = card.children_ids().to_vec();
        let parent_id = card.parent_id();

        if !children.is_empty() {
            match strategy {
                DeleteStrategy::Reject => {
                    return Err(DomainError::CardHasChildren);
                }
                DeleteStrategy::CascadeDelete => {
                    for child_id in children {
                        self.delete_card(child_id, DeleteStrategy::CascadeDelete)?;
                    }
                }
                DeleteStrategy::ReparentToGrandparent => {
                    if let Some(grandparent_id) = parent_id {
                        for child_id in children {
                            self.reparent_card(child_id, grandparent_id)?;
                        }
                    } else {
                        return Err(DomainError::InvalidOperation(
                            "Cannot reparent to grandparent: card is a root card without a parent."
                                .into(),
                        ));
                    }
                }
            }
        }

        // If the card has a parent, remove it from the parent's list
        if let Some(p_id) = parent_id {
            if let Ok(p_mut) = self.get_card_mut(p_id) {
                p_mut.remove_child(card_id);
            }
        }

        // Finally, remove the card
        self.store.remove(&card_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_and_projections() {
        let mut reg = CardRegistry::new();
        let root_id = reg.create_root_card("Root".into()).unwrap();

        assert_eq!(reg.get_root_cards().len(), 1);
        let root = reg.get_card(root_id).unwrap();
        let bucket_id = root.buckets()[0].id();

        let child1_id = reg
            .create_child_card("Child 1".into(), root_id, bucket_id)
            .unwrap();
        reg.create_child_card("Child 2".into(), root_id, bucket_id)
            .unwrap();

        assert_eq!(reg.get_children(root_id).unwrap().len(), 2);

        let projection = reg.board_projection(root_id).unwrap();
        assert_eq!(projection.get(&bucket_id).unwrap().len(), 2);

        // Move child to a new bucket
        let new_bucket_id = reg.add_bucket(root_id, "In Progress".into()).unwrap();
        reg.move_card_to_bucket(child1_id, new_bucket_id).unwrap();

        let projection = reg.board_projection(root_id).unwrap();
        assert_eq!(projection.get(&new_bucket_id).unwrap().len(), 1);
        assert_eq!(projection.get(&bucket_id).unwrap().len(), 1);
    }

    #[test]
    fn test_cycle_detection() {
        let mut reg = CardRegistry::new();
        let root_id = reg.create_root_card("Root".into()).unwrap();
        let b_unassigned = reg.get_card(root_id).unwrap().buckets()[0].id();

        let child_id = reg
            .create_child_card("Child".into(), root_id, b_unassigned)
            .unwrap();
        let child_b = reg.get_card(child_id).unwrap().buckets()[0].id();

        let grandchild_id = reg
            .create_child_card("Grandchild".into(), child_id, child_b)
            .unwrap();

        // Reparenting child to grandchild should fail (cycle)
        assert!(matches!(
            reg.reparent_card(child_id, grandchild_id),
            Err(DomainError::CycleDetected)
        ));

        // Reparenting root to child should fail (cycle)
        assert!(matches!(
            reg.reparent_card(root_id, child_id),
            Err(DomainError::CycleDetected)
        ));
    }

    #[test]
    fn test_delete_strategy_reject() {
        let mut reg = CardRegistry::new();
        let root_id = reg.create_root_card("Root".into()).unwrap();
        let bid = reg.get_card(root_id).unwrap().buckets()[0].id();
        reg.create_child_card("Child".into(), root_id, bid).unwrap();

        assert!(matches!(
            reg.delete_card(root_id, DeleteStrategy::Reject),
            Err(DomainError::CardHasChildren)
        ));
    }

    #[test]
    fn test_delete_strategy_cascade() {
        let mut reg = CardRegistry::new();
        let root_id = reg.create_root_card("Root".into()).unwrap();
        let bid = reg.get_card(root_id).unwrap().buckets()[0].id();
        let child_id = reg.create_child_card("Child".into(), root_id, bid).unwrap();

        reg.delete_card(root_id, DeleteStrategy::CascadeDelete)
            .unwrap();
        assert!(reg.get_card(root_id).is_err());
        assert!(reg.get_card(child_id).is_err());
    }

    #[test]
    fn test_delete_strategy_reparent() {
        let mut reg = CardRegistry::new();
        let root_id = reg.create_root_card("Root".into()).unwrap();
        let r_bid = reg.get_card(root_id).unwrap().buckets()[0].id();

        let child_id = reg
            .create_child_card("Child".into(), root_id, r_bid)
            .unwrap();
        let c_bid = reg.get_card(child_id).unwrap().buckets()[0].id();

        let grandchild_id = reg
            .create_child_card("Grandchild".into(), child_id, c_bid)
            .unwrap();

        // Delete child and reparent grandchild to root
        reg.delete_card(child_id, DeleteStrategy::ReparentToGrandparent)
            .unwrap();
        assert!(reg.get_card(child_id).is_err());

        let grandchild = reg.get_card(grandchild_id).unwrap();
        assert_eq!(grandchild.parent_id(), Some(root_id));
        // It should have moved to the Unassigned bucket of the new parent
        assert_eq!(grandchild.bucket_id(), Some(r_bid));
    }

    #[test]
    fn test_remove_bucket_fails_if_not_empty() {
        let mut reg = CardRegistry::new();
        let root_id = reg.create_root_card("Root".into()).unwrap();
        let new_bucket = reg.add_bucket(root_id, "In Progress".into()).unwrap();

        reg.create_child_card("Child".into(), root_id, new_bucket)
            .unwrap();

        // Trying to delete "In Progress" should fail because the child is in it
        assert!(matches!(
            reg.remove_bucket(root_id, new_bucket),
            Err(DomainError::BucketNotEmpty)
        ));
    }

    #[test]
    fn test_reparent_to_same_parent_is_noop() {
        let mut reg = CardRegistry::new();
        let root_id = reg.create_root_card("Root".into()).unwrap();
        let b_unassigned = reg.get_card(root_id).unwrap().buckets()[0].id();

        // Add a child
        let child_id = reg
            .create_child_card("Child".into(), root_id, b_unassigned)
            .unwrap();

        // Move it to a new bucket so we can verify it doesn't reset to Unassigned
        let b_progress = reg.add_bucket(root_id, "Progress".into()).unwrap();
        reg.move_card_to_bucket(child_id, b_progress).unwrap();

        // Reparent to same parent
        reg.reparent_card(child_id, root_id).unwrap();

        let child = reg.get_card(child_id).unwrap();
        assert_eq!(child.parent_id(), Some(root_id));
        assert_eq!(
            child.bucket_id(),
            Some(b_progress),
            "Bucket should NOT have been reset to Unassigned"
        );

        let parent = reg.get_card(root_id).unwrap();
        assert_eq!(
            parent.children_ids().len(),
            1,
            "Child should still be present exactly once"
        );
        assert_eq!(parent.children_ids()[0], child_id);
    }

    #[test]
    fn test_get_children_fails_on_missing_child() {
        let mut reg = CardRegistry::new();
        let root_id = reg.create_root_card("Root".into()).unwrap();

        {
            let root = reg.get_card_mut(root_id).unwrap();
            root.add_child(CardId::new()); // Manually corrupt with non-existent child
        }

        assert!(matches!(
            reg.get_children(root_id),
            Err(DomainError::CardNotFound(_))
        ));
    }

    #[test]
    fn test_board_projection_fails_on_unknown_bucket() {
        let mut reg = CardRegistry::new();
        let root_id = reg.create_root_card("Root".into()).unwrap();
        let b_unassigned = reg.get_card(root_id).unwrap().buckets()[0].id();

        let child_id = reg
            .create_child_card("Child".into(), root_id, b_unassigned)
            .unwrap();

        {
            let child = reg.get_card_mut(child_id).unwrap();
            child.assign_to_bucket(BucketId::new()); // Manually corrupt with non-existent bucket
        }

        assert!(matches!(
            reg.board_projection(root_id),
            Err(DomainError::BucketNotFound(_))
        ));
    }
}
