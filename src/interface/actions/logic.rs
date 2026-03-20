//! Pure logic for UI reordering and manipulation.
//!
//! This module contains algorithms for managing item positions and ID lists
//! without any dependencies on the UI framework or browser APIs.
//!
//! For more on how Rust's data manipulation compares to Python's list comprehension,
//! see `docs/rust-for-python-devs.md`.

/// Reorders a list by moving one item to a target index after removing the dragged item.
///
/// # Examples
///
/// ```rust
/// use kanban_planner::interface::actions::reorder_ids;
///
/// let original = vec![1, 2, 3];
/// let reordered = reorder_ids(&original, 3, 0);
/// assert_eq!(reordered, vec![3, 1, 2]);
/// ```
pub fn reorder_ids<T>(ordered_ids: &[T], dragged_id: T, target_index: usize) -> Vec<T>
where
    T: Copy + Eq,
{
    let mut reordered: Vec<T> = ordered_ids
        .iter()
        .copied()
        .filter(|id| *id != dragged_id)
        .collect();
    let insertion_index = target_index.min(reordered.len());
    reordered.insert(insertion_index, dragged_id);
    reordered
}

#[cfg(test)]
mod tests {
    use super::reorder_ids;

    #[test]
    fn reorder_ids_moves_item_to_target_index() {
        let reordered = reorder_ids(&[1, 2, 3, 4], 3, 1);
        assert_eq!(reordered, vec![1, 3, 2, 4]);
    }

    #[test]
    fn reorder_ids_clamps_insertion_to_end() {
        let reordered = reorder_ids(&[1, 2, 3], 1, 99);
        assert_eq!(reordered, vec![2, 3, 1]);
    }
}
