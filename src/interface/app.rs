use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::infrastructure::repository::AppPersistence;
use crate::interface::Route;
use crate::interface::components::modal::{CardModal, EditCardModal, ModalType, NotesModal};
use dioxus::prelude::*;
use tracing::{Level, info, warn};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;

#[derive(Clone, Copy, Default)]
pub struct IsDark(pub bool);

#[derive(Clone, Copy, Default)]
pub struct IsDragging(pub bool);

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum DraggedItemKind {
    #[default]
    None,
    Card,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum RouteMotionDirection {
    #[default]
    None,
    Forward,
    Backward,
}

const ROUTE_MOTION_DURATION_MS: i32 = 720;

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
                load_warning.set(Some(error.to_string()));
                CardRegistry::default()
            }
        };
        Signal::new(initial)
    });

    // Theme state: default to dark mode.
    let is_dark = use_context_provider(|| Signal::new(IsDark(true)));
    let route_motion = use_context_provider(|| Signal::new(RouteMotionDirection::None));

    // Global modal state.
    let mut active_modal = use_context_provider(|| Signal::new(None::<ModalType>));
    let _is_dragging = use_context_provider(|| Signal::new(IsDragging(false)));
    let _dragged_item_kind = use_context_provider(|| Signal::new(DraggedItemKind::None));

    let shell_class = if is_dark().0 {
        "app-shell theme-dark dark"
    } else {
        "app-shell theme-light"
    };
    let route_motion_class = match route_motion() {
        RouteMotionDirection::None => "",
        RouteMotionDirection::Forward => " app-content--motion-forward",
        RouteMotionDirection::Backward => " app-content--motion-backward",
    };

    // Side effect: Save to persistence only when the registry snapshot changes.
    let registry_snapshot = registry.read().clone();
    let mut save_warning = persistence_warning;
    use_effect(use_reactive!(|(registry_snapshot,)| {
        if let Err(error) = AppPersistence::save_registry(&registry_snapshot) {
            warn!(error = %error, "Persistence save failed while app state changed");
            record_diagnostic(
                Level::WARN,
                "interface",
                format!("Persistence save warning shown to user: {error}"),
            );
            save_warning.set(Some(error.to_string()));
        }
    }));

    let route_motion_clear = route_motion;
    use_effect(move || {
        if route_motion() != RouteMotionDirection::None {
            schedule_route_motion_clear(route_motion_clear);
        }
    });

    rsx! {
        div { class: "{shell_class}",
            link { rel: "stylesheet", href: asset!("/assets/app.css") }
            div { class: "app-backdrop" }
            div { class: "app-atmosphere" }

            div { class: "app-content{route_motion_class}",
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
                    ModalType::CreateCard { parent_id } => {
                        rsx! {
                            CardModal {
                                on_close: move |_| active_modal.set(None),
                                parent_id,
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
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn schedule_route_motion_clear(mut route_motion: Signal<RouteMotionDirection>) {
    let Some(window) = web_sys::window() else {
        route_motion.set(RouteMotionDirection::None);
        return;
    };

    let clear = Closure::wrap(Box::new(move || {
        route_motion.set(RouteMotionDirection::None);
    }) as Box<dyn FnMut()>);

    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        clear.as_ref().unchecked_ref(),
        ROUTE_MOTION_DURATION_MS,
    );
    clear.forget();
}

#[cfg(not(target_arch = "wasm32"))]
fn schedule_route_motion_clear(mut route_motion: Signal<RouteMotionDirection>) {
    route_motion.set(RouteMotionDirection::None);
}
