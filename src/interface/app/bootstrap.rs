use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::infrastructure::repository::AppPersistence;
use dioxus::prelude::*;
use tracing::{Level, info, warn};

pub(super) fn initialize_registry_signal(
    mut persistence_warning: Signal<Option<String>>,
) -> CardRegistry {
    match AppPersistence::load_registry() {
        Ok(Some(registry)) => {
            info!(
                workspace_child_count = registry.workspace_child_count(),
                "Initialized registry from persistence"
            );
            registry
        }
        Ok(None) => {
            info!("Initialized registry with an empty in-memory state");
            CardRegistry::default()
        }
        Err(error) => {
            warn!(error = %error, "Falling back to in-memory registry after persistence load failure");
            record_diagnostic(
                Level::WARN,
                "interface",
                format!("Persistence load warning shown to user: {error}"),
            );
            persistence_warning.set(Some(error.to_string()));
            CardRegistry::default()
        }
    }
}

pub(super) fn persist_registry_snapshot(
    registry_snapshot: &CardRegistry,
    mut persistence_warning: Signal<Option<String>>,
) {
    if let Err(error) = AppPersistence::save_registry(registry_snapshot) {
        warn!(error = %error, "Persistence save failed while app state changed");
        record_diagnostic(
            Level::WARN,
            "interface",
            format!("Persistence save warning shown to user: {error}"),
        );
        persistence_warning.set(Some(error.to_string()));
    }
}
