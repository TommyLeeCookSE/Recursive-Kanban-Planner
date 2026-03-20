//! Command execution feedback and reporting utilities.
//!
//! This module handles the communication of command results back to the user
//! via the UI warning banner and diagnostics logs.
//!
//! For more on Rust's `Result` handling in UI applications, see
//! `docs/rust-for-python-devs.md`.

use crate::application::{Command, execute};
use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use crate::domain::registry::{CardRegistry, DeleteStrategy};
use crate::infrastructure::logging::record_diagnostic;
use dioxus::prelude::*;
use tracing::{Level, info, warn};
use super::logic::reorder_ids;

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
