mod command;
mod execute;
mod projections;

pub use command::Command;
pub use execute::execute;
pub use projections::{BoardView, CardPreviewView, build_board_view, build_card_preview_view};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::error::DomainError;
    use crate::domain::id::CardId;
    use crate::domain::registry::CardRegistry;

    #[test]
    fn test_execute_create_workspace_child() {
        let mut registry = CardRegistry::new();
        execute(
            Command::CreateWorkspaceChildCard {
                title: "Top Level Board".into(),
            },
            &mut registry,
        )
        .unwrap();

        assert_eq!(registry.workspace_child_count(), 1);
    }

    #[test]
    fn test_board_view_returns_ordered_children() {
        let mut registry = CardRegistry::new();
        let workspace_id = registry.workspace_card_id().unwrap();
        let first_id = registry
            .create_child_card("First".into(), workspace_id)
            .unwrap();
        let second_id = registry
            .create_child_card("Second".into(), workspace_id)
            .unwrap();

        let view = build_board_view(workspace_id, &registry).unwrap();
        let ids: Vec<CardId> = view.children.iter().map(|card| card.id()).collect();
        assert_eq!(ids, vec![first_id, second_id]);
    }

    #[test]
    fn test_card_preview_returns_immediate_children_only() {
        let mut registry = CardRegistry::new();
        let workspace_id = registry.workspace_card_id().unwrap();
        let project_id = registry
            .create_child_card("Project".into(), workspace_id)
            .unwrap();
        let task_id = registry
            .create_child_card("Task".into(), project_id)
            .unwrap();

        let preview = build_card_preview_view(project_id, &registry).unwrap();
        assert_eq!(preview.card.id(), project_id);
        assert_eq!(preview.children.len(), 1);
        assert_eq!(preview.children[0].id(), task_id);
    }

    #[test]
    fn test_execute_drop_child_at_position() {
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

        execute(
            Command::DropChildAtPosition {
                parent_id: workspace_id,
                card_id: third,
                target_index: 0,
            },
            &mut registry,
        )
        .unwrap();

        let ids: Vec<CardId> = registry
            .get_children(workspace_id)
            .unwrap()
            .iter()
            .map(|card| card.id())
            .collect();
        assert_eq!(ids, vec![third, first, second]);
    }

    #[test]
    fn test_build_board_view_preserves_missing_card_error() {
        let registry = CardRegistry::new();
        let missing_id = CardId::default();

        let result = build_board_view(missing_id, &registry);

        assert!(matches!(result, Err(DomainError::CardNotFound(id)) if id == missing_id));
    }
}
