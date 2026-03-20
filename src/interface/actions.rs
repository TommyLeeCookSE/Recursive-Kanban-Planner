//! User interface actions and common command handlers.
//!
//! This module provides the bridge between UI events (clicks, drag-and-drop)
//! and the application command layer.
//!
//! For a trace of how a UI action becomes a domain mutation, see
//! `docs/rust-for-python-devs.md`.

use crate::application::{Command, execute};
use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::domain::registry::DeleteStrategy;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::app::{DraggedItemKind, IsDragging};
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use js_sys::{Function, Reflect};
use std::str::FromStr;
use tracing::{Level, info, warn};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

/// Executes a UI-originated command and routes failures to diagnostics plus the warning banner.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::application::Command;
/// use kanban_planner::interface::actions::execute_command_with_feedback;
///
/// let command = Command::RenameCard { id, title: "New Title".into() };
/// execute_command_with_feedback(command, registry, warning, "rename", "Renaming card");
/// ```
pub fn execute_command_with_feedback(
    command: Command,
    mut registry: Signal<CardRegistry>,
    warning_message: Signal<Option<String>>,
    log_target: &'static str,
    action_label: impl Into<String>,
) -> Result<(), DomainError> {
    let action_label = action_label.into();
    let result = {
        let mut reg = registry.write();
        execute(command, &mut reg)
    };

    report_result(result, warning_message, log_target, action_label)
}

/// Deletes a card through the command layer and surfaces failures consistently.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::interface::actions::delete_card_with_feedback;
///
/// delete_card_with_feedback(card_id, registry, warning, "delete", "Deleting card");
/// ```
pub fn delete_card_with_feedback(
    id: CardId,
    registry: Signal<CardRegistry>,
    warning_message: Signal<Option<String>>,
    log_target: &'static str,
    action_label: impl Into<String>,
) {
    let _ = execute_command_with_feedback(
        Command::DeleteCard {
            id,
            strategy: DeleteStrategy::CascadeDelete,
        },
        registry,
        warning_message,
        log_target,
        action_label,
    );
}

/// Runs a UI mutation inside the browser view-transition API when available.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::interface::actions::run_with_view_transition;
///
/// run_with_view_transition(move || {
///     show_modal.set(None);
/// });
/// ```
pub fn run_with_view_transition<F>(callback: F)
where
    F: FnOnce() + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            callback();
            return;
        };
        let Some(document) = window.document() else {
            callback();
            return;
        };

        let callback_js = Closure::once_into_js(callback);
        let callback_fn = callback_js.clone().dyn_into::<Function>().ok();

        let Ok(start_view_transition_value) =
            Reflect::get(document.as_ref(), &JsValue::from_str("startViewTransition"))
        else {
            if let Some(function) = callback_fn {
                let _ = function.call0(&JsValue::NULL);
            }
            return;
        };

        let Some(start_view_transition) = start_view_transition_value.dyn_into::<Function>().ok()
        else {
            if let Some(function) = callback_fn {
                let _ = function.call0(&JsValue::NULL);
            }
            return;
        };

        let _ = start_view_transition.call1(document.as_ref(), &callback_js);
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    callback();
}

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

/// Shared context for logging and surfacing reorder operations.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::interface::actions::ReorderFeedbackContext;
///
/// let context = ReorderFeedbackContext::new(registry, warning, "board", "Reordering cards");
/// ```
#[derive(Clone)]
pub struct ReorderFeedbackContext {
    pub registry: Signal<CardRegistry>,
    pub warning_message: Signal<Option<String>>,
    pub log_target: &'static str,
    pub action_label: String,
}

impl ReorderFeedbackContext {
    /// Creates a new reorder feedback context.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use kanban_planner::interface::actions::ReorderFeedbackContext;
    ///
    /// let context = ReorderFeedbackContext::new(registry, warning, "board", "Reordering cards");
    /// ```
    pub fn new(
        registry: Signal<CardRegistry>,
        warning_message: Signal<Option<String>>,
        log_target: &'static str,
        action_label: impl Into<String>,
    ) -> Self {
        Self {
            registry,
            warning_message,
            log_target,
            action_label: action_label.into(),
        }
    }
}

/// Reorders a collection and executes the derived command only when the order actually changes.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::interface::actions::execute_reorder_with_feedback;
///
/// execute_reorder_with_feedback(&ids, dragged_id, index, context, |new_ids| {
///     Command::ReorderChildCards { parent_id, child_ids: new_ids }
/// });
/// ```
pub fn execute_reorder_with_feedback<T, F>(
    ordered_ids: &[T],
    dragged_id: T,
    target_index: usize,
    context: ReorderFeedbackContext,
    build_command: F,
) -> Result<(), DomainError>
where
    T: Copy + Eq + std::fmt::Display,
    F: FnOnce(Vec<T>) -> Command,
{
    let action_label = context.action_label.clone();
    let reordered = reorder_ids(ordered_ids, dragged_id, target_index);
    if reordered == ordered_ids {
        return Ok(());
    }

    info!(
        ui_target = context.log_target,
        action = %action_label,
        dragged_id = %dragged_id,
        drop_index = target_index,
        "Attempting reorder"
    );
    record_diagnostic(
        Level::INFO,
        context.log_target,
        format!("Attempting {action_label} for {dragged_id} at index {target_index}"),
    );
    execute_command_with_feedback(
        build_command(reordered),
        context.registry,
        context.warning_message,
        context.log_target,
        action_label,
    )
}

/// Surfaces a command result through tracing, in-memory diagnostics, and the warning banner.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::interface::actions::report_result;
///
/// let result = registry.write().rename_card(id, title);
/// report_result(result, warning, "rename", "Renaming card");
/// ```
pub fn report_result(
    result: Result<(), DomainError>,
    mut warning_message: Signal<Option<String>>,
    log_target: &'static str,
    action_label: impl Into<String>,
) -> Result<(), DomainError> {
    let action_label = action_label.into();

    match result {
        Ok(()) => {
            warning_message.set(None);
            Ok(())
        }
        Err(error_value) => {
            let message = error_value.to_string();
            warn!(ui_target = log_target, action = %action_label, error = %error_value, "UI action failed");
            record_diagnostic(
                Level::WARN,
                log_target,
                format!("UI action '{action_label}' failed: {message}"),
            );
            warning_message.set(Some(message));
            Err(error_value)
        }
    }
}

/// Prepares a browser drag session with a concrete payload and explicit move semantics.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::interface::actions::prime_drag_session;
///
/// prime_drag_session(&event, "board", format!("card:{}", id), DraggedItemKind::Card, kind, active);
/// ```
pub fn prime_drag_session(
    event: &DragEvent,
    log_target: &'static str,
    payload: impl Into<String>,
    dragged_item_kind: DraggedItemKind,
    mut dragged_item_kind_signal: Signal<DraggedItemKind>,
    mut is_dragging: Signal<IsDragging>,
) {
    let payload = payload.into();
    let data_transfer = event.data().data_transfer();

    is_dragging.set(IsDragging(true));
    dragged_item_kind_signal.set(dragged_item_kind);

    if let Err(error) = data_transfer.clear_data(None) {
        warn!(
            ui_target = log_target,
            error = %error,
            "Failed to clear previous drag payload"
        );
        record_diagnostic(
            Level::WARN,
            log_target,
            format!("Failed to clear drag payload before drag start: {error}"),
        );
    }

    if let Err(error) = data_transfer.set_data("text/plain", &payload) {
        warn!(
            ui_target = log_target,
            payload = %payload,
            error = %error,
            "Failed to set drag payload"
        );
        record_diagnostic(
            Level::WARN,
            log_target,
            format!("Failed to set drag payload '{payload}': {error}"),
        );
    }

    data_transfer.set_effect_allowed("move");
    data_transfer.set_drop_effect("move");
}

/// Keeps the browser drag interaction in explicit move mode while a target is hovered.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::interface::actions::prime_drop_target;
///
/// prime_drop_target(&event);
/// ```
pub fn prime_drop_target(event: &DragEvent) {
    event.prevent_default();
    event.data().data_transfer().set_drop_effect("move");
}

/// Extracts a card ID from a browser drag event payload.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::interface::actions::dragged_card_id;
///
/// if let Some(id) = dragged_card_id(&event, "board") {
///     println!("Dragged card {}", id);
/// }
/// ```
pub fn dragged_card_id(event: &DragEvent, log_target: &'static str) -> Option<CardId> {
    let payload = read_drag_payload(event, log_target)?;
    let raw_id = payload.strip_prefix("card:")?.split(':').next()?;
    parse_card_id(raw_id, log_target, &payload)
}

fn read_drag_payload(event: &DragEvent, log_target: &'static str) -> Option<String> {
    let payload = event.data().data_transfer().get_as_text()?;
    if payload.is_empty() {
        record_diagnostic(
            Level::WARN,
            log_target,
            "Drag payload was unexpectedly empty",
        );
        return None;
    }
    Some(payload)
}

fn parse_card_id(raw_id: &str, log_target: &'static str, payload: &str) -> Option<CardId> {
    CardId::from_str(raw_id).ok().or_else(|| {
        warn!(
            ui_target = log_target,
            payload = %payload,
            "Failed to parse card drag payload"
        );
        record_diagnostic(
            Level::WARN,
            log_target,
            format!("Failed to parse card drag payload '{payload}'"),
        );
        None
    })
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
