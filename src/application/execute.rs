//! Execution logic for application commands.
//!
//! This module provides the `execute` function, which acts as the entry point
//! for all mutations. It handles logging, diagnostic recording, and
//! dispatching to the `Command` implementation.
//!
//! For a discussion on how this centralized dispatcher compares to Python's
//! decorator-based or middleware approaches, see `docs/rust-for-python-devs.md`.

use crate::application::command::Command;
use crate::domain::error::DomainError;
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use tracing::{Level, error, info};

/// Executes an application command against the provided registry.
///
/// This function logs the command's start and completion, and records a
/// diagnostic entry if the command fails.
///
/// # Examples
///
/// ```
/// use kanban_planner::application::{Command, execute};
/// use kanban_planner::domain::registry::CardRegistry;
///
/// let mut registry = CardRegistry::new();
/// let cmd = Command::CreateWorkspaceChildCard { title: "New Board".into() };
/// execute(cmd, &mut registry).unwrap();
/// ```
pub fn execute(command: Command, registry: &mut CardRegistry) -> Result<(), DomainError> {
    command.log_start();
    let command_label = command.name();
    let result = command.apply(registry);

    match &result {
        Ok(()) => {
            if let Err(validation_error) = registry.validate() {
                error!(
                    command = command_label,
                    error = %validation_error,
                    "Application command resulted in an invalid registry state"
                );
                record_diagnostic(
                    Level::ERROR,
                    "application",
                    format!("Application command '{command_label}' resulted in invalid state: {validation_error}"),
                );
                return Err(validation_error);
            }
            info!(command = command_label, "Application command completed");
        }
        Err(error_value) => {
            error!(
                command = command_label,
                error = %error_value,
                "Application command failed"
            );
            record_diagnostic(
                Level::ERROR,
                "application",
                format!("Application command '{command_label}' failed: {error_value}"),
            );
        }
    }

    result
}
