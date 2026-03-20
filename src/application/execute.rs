use crate::application::command::Command;
use crate::domain::error::DomainError;
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use tracing::{Level, error, info};

pub fn execute(command: Command, registry: &mut CardRegistry) -> Result<(), DomainError> {
    command.log_start();
    let command_label = command.name();
    let result = command.apply(registry);

    match &result {
        Ok(()) => info!(command = command_label, "Application command completed"),
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
