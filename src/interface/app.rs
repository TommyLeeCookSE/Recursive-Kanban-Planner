use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::infrastructure::repository::AppPersistence;
use crate::interface::Route;
use crate::interface::components::modal::{BucketModal, CardModal, ModalType, RenameCardModal};
use dioxus::prelude::*;
use tracing::{Level, info, warn};

/// The root application component.
///
/// Sets up the global state (Persistence, Registry, Theme, Active Modal)
/// and provides them via the Context API to the rest of the component tree.
///
/// # Examples
///
/// ```ignore
/// dioxus::launch(kanban_planner::interface::app::App);
/// ```
pub fn App() -> Element {
    let persistence_warning = use_context_provider(|| Signal::new(None::<String>));

    // Initialize the registry signal from persistence if available,
    // otherwise create a new empty registry and surface a warning.
    let mut load_warning = persistence_warning;
    let registry = use_context_provider(move || {
        let initial = match AppPersistence::load_registry() {
            Ok(Some(registry)) => {
                info!(
                    root_count = registry.get_root_cards().len(),
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
                load_warning.set(Some(error.to_string()));
                CardRegistry::default()
            }
        };
        Signal::new(initial)
    });

    // Theme state: default to dark mode.
    let is_dark = use_context_provider(|| Signal::new(true));

    // Global modal state.
    let mut active_modal = use_context_provider(|| Signal::new(None::<ModalType>));

    let shell_class = if is_dark() {
        "app-shell theme-dark dark"
    } else {
        "app-shell theme-light"
    };

    // Side effect: Save to persistence whenever the registry changes.
    let mut save_warning = persistence_warning;
    use_effect(move || {
        if let Err(error) = AppPersistence::save_registry(&registry.read()) {
            warn!(error = %error, "Persistence save failed while app state changed");
            record_diagnostic(
                Level::WARN,
                "interface",
                format!("Persistence save warning shown to user: {error}"),
            );
            save_warning.set(Some(error.to_string()));
        }
    });

    rsx! {
        div { class: "{shell_class}",
            link { rel: "stylesheet", href: asset!("/assets/app.css") }
            div { class: "app-backdrop" }
            div { class: "app-atmosphere" }

            div { class: "app-content",
                if let Some(message) = persistence_warning() {
                    div { class: "app-warning-banner px-6 py-3 text-sm font-semibold lg:px-12",
                        "{message}"
                    }
                }

                Router::<Route> {}
            }

            // Modal Overlay Dispatcher
            if let Some(modal) = active_modal() {
                match modal {
                    ModalType::CreateCard { parent_id, bucket_id } => {
                        rsx! {
                            CardModal {
                                on_close: move |_| active_modal.set(None),
                                parent_id,
                                bucket_id,
                                registry,
                            }
                        }
                    },
                    ModalType::CreateBucket { card_id } => {
                        rsx! {
                            BucketModal {
                                on_close: move |_| active_modal.set(None),
                                card_id,
                                registry,
                            }
                        }
                    },
                    ModalType::RenameCard { id, current_title } => {
                        rsx! {
                            RenameCardModal {
                                on_close: move |_| active_modal.set(None),
                                id,
                                current_title,
                                registry,
                            }
                        }
                    },
                }
            }
        }
    }
}
