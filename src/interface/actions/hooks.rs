//! Reusable Dioxus hooks for common UI actions.
//!
//! This module provides hooks that encapsulate common patterns like
//! executing commands with feedback and handling global state.
//!
//! For more on how Dioxus hooks compare to React's custom hooks,
//! see `docs/rust-for-python-devs.md`.

use crate::application::{Command, execute};
use crate::interface::app::use_board_signals;
use dioxus::prelude::*;

/// A hook that returns a function for executing commands against the registry.
///
/// It automatically handles:
/// - Getting the registry signal from context.
/// - Getting the warning message signal from context.
/// - Executing the command via the application layer.
/// - Updating the registry signal on success.
/// - Showing an error message on failure.
///
/// # Examples
///
/// ```ignore
/// let execute_cmd = use_execute_command();
/// execute_cmd(Command::CreateWorkspaceChildCard { title: "New Board".into() });
/// ```
pub fn use_execute_command() -> impl FnMut(Command) {
    let signals = use_board_signals();
    let mut registry = signals.registry;
    let mut warning_message = signals.warning_message;

    move |command: Command| {
        let mut registry_mut = registry.write();
        match execute(command, &mut registry_mut) {
            Ok(()) => {
                // Success! The registry is already updated via registry_mut.
                // We drop the lock here to allow the rest of the app to read.
                drop(registry_mut);
            }
            Err(error) => {
                warning_message.set(Some(format!("Action failed: {error}")));
            }
        }
    }
}
