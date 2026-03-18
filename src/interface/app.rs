use crate::application::PopupNotification;
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::infrastructure::repository::AppPersistence;
use crate::interface::Route;
use crate::interface::components::modal::{
    BucketModal, CardLabelsModal, CardModal, EditBucketModal, EditCardModal, ManageLabelsModal,
    ManageRulesModal, ModalType, NotesModal,
};
use dioxus::prelude::*;
use tracing::{Level, info, warn};

#[derive(Clone, Copy, Default)]
pub struct IsDark(pub bool);

#[derive(Clone, Copy, Default)]
pub struct IsDragging(pub bool);

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
    let is_dark = use_context_provider(|| Signal::new(IsDark(true)));

    // Global modal state.
    let mut active_modal = use_context_provider(|| Signal::new(None::<ModalType>));
    let mut popup_queue = use_context_provider(|| Signal::new(Vec::<PopupNotification>::new()));
    let _is_dragging = use_context_provider(|| Signal::new(IsDragging(false)));

    let shell_class = if is_dark().0 {
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

            if let Some(popup) = popup_queue().first().cloned() {
                div { class: "fixed bottom-6 right-6 z-[60] max-w-sm",
                    div { class: "app-modal-surface rounded-[1.5rem] px-5 py-4",
                        div { class: "mb-3 flex items-start justify-between gap-4",
                            div {
                                h3 { class: "app-text-primary text-lg font-bold", "{popup.title}" }
                                p { class: "app-text-muted mt-2 text-sm", "{popup.message}" }
                            }
                            button {
                                class: "app-button-ghost p-2",
                                onclick: move |_| {
                                    let mut queued = popup_queue();
                                    if !queued.is_empty() {
                                        queued.remove(0);
                                        popup_queue.set(queued);
                                    }
                                },
                                "X"
                            }
                        }
                    }
                }
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
                    ModalType::EditBucket { card_id, bucket_id } => {
                        rsx! {
                            EditBucketModal {
                                on_close: move |_| active_modal.set(None),
                                card_id,
                                bucket_id,
                                registry,
                            }
                        }
                    },
                    ModalType::EditCard { id } => {
                        rsx! {
                            EditCardModal {
                                on_close: move |_| active_modal.set(None),
                                id,
                                registry,
                            }
                        }
                    },
                    ModalType::CardNotes { card_id } => {
                        rsx! {
                            NotesModal {
                                on_close: move |_| active_modal.set(None),
                                card_id,
                                registry,
                            }
                        }
                    }
                    ModalType::CardLabels { card_id } => {
                        rsx! {
                            CardLabelsModal {
                                on_close: move |_| active_modal.set(None),
                                card_id,
                                registry,
                            }
                        }
                    }
                    ModalType::ManageLabels {} => {
                        rsx! {
                            ManageLabelsModal {
                                on_close: move |_| active_modal.set(None),
                                registry,
                            }
                        }
                    }
                    ModalType::ManageRules {} => {
                        rsx! {
                            ManageRulesModal {
                                on_close: move |_| active_modal.set(None),
                                registry,
                            }
                        }
                    }
                }
            }
        }
    }
}
