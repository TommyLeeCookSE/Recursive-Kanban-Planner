use dioxus::prelude::*;
use crate::domain::registry::CardRegistry;
use crate::infrastructure::repository::LocalStorageRepository;
use crate::interface::Route;
use crate::interface::components::modal::{Modal, ModalType, CardModal};

pub fn App() -> Element {
    // Initialize the registry signal from local storage if available, 
    // otherwise create a new empty registry.
    let registry = use_context_provider(|| {
        let initial = LocalStorageRepository::load_from_local_storage()
            .ok()
            .flatten()
            .unwrap_or_else(CardRegistry::new);
        Signal::new(initial)
    });

    // Theme state: default to dark mode.
    let is_dark = use_context_provider(|| Signal::new(true));

    // Global modal state.
    let mut active_modal = use_context_provider(|| Signal::new(None::<crate::interface::components::modal::ModalType>));

    // Side effect: Save to local storage whenever the registry changes.
    use_effect(move || {
        let _ = LocalStorageRepository::save_to_local_storage(&registry.read());
    });

    rsx! {
        // Root container with dark class toggle
        div { class: if is_dark() { "dark" } else { "" },
            div { class: "bg-gray-100 dark:bg-gray-900 min-h-screen text-gray-900 dark:text-gray-100 transition-colors duration-200",
                style { {include_str!("tailwind.css")} }
                Router::<Route> {}
                
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
                            use crate::interface::components::modal::BucketModal;
                            rsx! {
                                BucketModal {
                                    on_close: move |_| active_modal.set(None),
                                    card_id,
                                    registry,
                                }
                            }
                        },
                        _ => rsx! { div {} }
                    }
                }
            }
        }
    }
}
