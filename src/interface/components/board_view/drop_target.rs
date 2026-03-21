use crate::application::Command;
use crate::interface::actions::{
    dragged_card_id, execute_command_with_feedback, prime_drop_target, run_with_view_transition,
};
use crate::interface::app::DraggedItemKind;
use crate::interface::components::board_view::models::BoardRenderContext;
use crate::interface::components::visuals::{DropZoneKind, drop_zone_classes};
use dioxus::prelude::*;

const BOARD_ROUTE_TARGET: &str = "board-route";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DropZoneMode {
    BoardSurface,
    CardLane,
}

pub(super) fn styled_drop_zone_classes(
    mode: DropZoneMode,
    is_active: bool,
    dragged_item_kind: DraggedItemKind,
) -> String {
    let base_classes = drop_zone_classes(
        match mode {
            DropZoneMode::BoardSurface => DropZoneKind::Board,
            DropZoneMode::CardLane => DropZoneKind::Card,
        },
        is_active,
        dragged_item_kind,
    );

    match mode {
        DropZoneMode::BoardSurface => format!("{base_classes} min-h-[3.25rem] rounded-2xl"),
        DropZoneMode::CardLane => format!("{base_classes} app-drop-slot-lane"),
    }
}

pub(super) fn activate_drop_zone(
    event: &DragEvent,
    mut card_drop_index: Signal<Option<usize>>,
    index: usize,
) {
    prime_drop_target(event);
    card_drop_index.set(Some(index));
}

pub(super) fn clear_drop_zone(mut card_drop_index: Signal<Option<usize>>, index: usize) {
    if card_drop_index() == Some(index) {
        card_drop_index.set(None);
    }
}

pub(super) fn apply_card_drop(
    event: DragEvent,
    target_index: usize,
    context: BoardRenderContext,
    diagnostic_message: String,
) {
    event.prevent_default();

    let Some(card_id) = dragged_card_id(&event, BOARD_ROUTE_TARGET) else {
        return;
    };

    let mut card_drop_index = context.drag.card_drop_index;
    card_drop_index.set(None);

    let registry = context.registry;
    let warning_message = context.warning_message;
    let board_id = context.board_id;

    run_with_view_transition(move || {
        let current_parent_id = registry
            .read()
            .get_card(card_id)
            .ok()
            .and_then(|c| c.parent_id());

        if let Some(old_parent) = current_parent_id
            && old_parent != board_id
        {
            let _ = execute_command_with_feedback(
                Command::ReparentCard {
                    card_id,
                    new_parent_id: board_id,
                },
                registry,
                warning_message,
                BOARD_ROUTE_TARGET,
                format!("Reparenting card for {diagnostic_message}"),
            );
        }

        let _ = execute_command_with_feedback(
            Command::DropChildAtPosition {
                parent_id: board_id,
                card_id,
                target_index,
            },
            registry,
            warning_message,
            BOARD_ROUTE_TARGET,
            diagnostic_message,
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styled_drop_zone_classes_for_board_surface() {
        let classes =
            styled_drop_zone_classes(DropZoneMode::BoardSurface, false, DraggedItemKind::None);
        assert!(classes.contains("min-h-[3.25rem]"));
        assert!(classes.contains("rounded-2xl"));
    }

    #[test]
    fn test_styled_drop_zone_classes_for_card_lane() {
        let classes =
            styled_drop_zone_classes(DropZoneMode::CardLane, false, DraggedItemKind::None);
        assert!(classes.contains("app-drop-slot-lane"));
    }
}
