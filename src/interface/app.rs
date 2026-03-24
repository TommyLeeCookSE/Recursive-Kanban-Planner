//! The root application shell and global state management.
//!
//! This module coordinates the high-level application lifecycle, including
//! theme management, persistence synchronization, and modal orchestration.
//!
//! For an overview of how the app shell is structured, see
//! `docs/rust-for-python-devs.md`.

mod bootstrap;
mod modal_dispatch;

use crate::interface::Route;
use crate::interface::app::bootstrap::{initialize_registry_signal, persist_registry_snapshot};
use crate::interface::app::modal_dispatch::render_modal_overlay;
use crate::interface::components::modal::ModalType;
use dioxus::prelude::*;

/// A wrapper for the application's dark mode preference.
///
/// # Examples
///
/// ```rust
/// use kanban_planner::interface::app::IsDark;
///
/// let preference = IsDark(true);
/// assert!(preference.0);
/// ```
#[derive(Clone, Copy, Default)]
pub struct IsDark(pub bool);

/// A wrapper for the global drag-and-drop state.
///
/// # Examples
///
/// ```rust
/// use kanban_planner::interface::app::IsDragging;
///
/// let state = IsDragging(false);
/// assert!(!state.0);
/// ```
#[derive(Clone, Copy, Default)]
pub struct IsDragging(pub bool);

/// The kind of item being dragged in the UI.
///
/// # Examples
///
/// ```rust
/// use kanban_planner::interface::app::DraggedItemKind;
///
/// let kind = DraggedItemKind::Card;
/// assert!(matches!(kind, DraggedItemKind::Card));
/// ```
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum DraggedItemKind {
    #[default]
    None,
    Card,
}

/// A collection of signals used across the board UI.
#[derive(Clone, Copy)]
pub struct BoardSignals {
    /// The central card registry.
    pub registry: Signal<crate::domain::registry::CardRegistry>,
    /// The currently active modal, if any.
    pub active_modal: Signal<Option<crate::interface::components::modal::ModalType>>,
    /// The global warning message banner content.
    pub warning_message: Signal<Option<String>>,
    /// Whether a drag operation is currently in progress.
    pub is_dragging: Signal<IsDragging>,
    /// The kind of item being dragged.
    pub dragged_item_kind: Signal<DraggedItemKind>,
}

/// A hook to access all standard board signals from the context.
///
/// # Examples
///
/// ```ignore
/// let signals = use_board_signals();
/// ```
pub fn use_board_signals() -> BoardSignals {
    BoardSignals {
        registry: use_context::<Signal<crate::domain::registry::CardRegistry>>(),
        active_modal: use_context::<Signal<Option<crate::interface::components::modal::ModalType>>>(
        ),
        warning_message: use_context::<Signal<Option<String>>>(),
        is_dragging: use_context::<Signal<IsDragging>>(),
        dragged_item_kind: use_context::<Signal<DraggedItemKind>>(),
    }
}

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
    let registry =
        use_context_provider(move || Signal::new(initialize_registry_signal(persistence_warning)));

    // Theme state: default to dark mode.
    let _is_dark = use_context_provider(|| Signal::new(IsDark(true)));
    // Global modal state.
    #[allow(unused_mut)]
    let mut active_modal = use_context_provider(|| Signal::new(None::<ModalType>));
    let is_dragging = use_context_provider(|| Signal::new(IsDragging(false)));
    let _dragged_item_kind = use_context_provider(|| Signal::new(DraggedItemKind::None));

    let is_dark: Signal<IsDark> = use_context();

    // Global keyboard shortcuts
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use wasm_bindgen::closure::Closure;

            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            let callback = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                if event.ctrl_key() && event.key() == "k" {
                    event.prevent_default();
                    active_modal.set(Some(ModalType::Search));
                }
            }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

            document
                .add_event_listener_with_callback("keydown", callback.as_ref().unchecked_ref())
                .unwrap();

            callback.forget();
        }
    });

    let shell_class = format!(
        "{} {}",
        if is_dark().0 {
            "app-shell theme-dark dark"
        } else {
            "app-shell theme-light"
        },
        if is_dragging().0 {
            "app-is-dragging"
        } else {
            ""
        }
    );

    // Debounced persistence: wait 1000ms after the last registry change before saving.
    use_resource(move || async move {
        let registry_snapshot = registry.read().clone();
        gloo_timers::future::TimeoutFuture::new(1000).await;
        persist_registry_snapshot(&registry_snapshot, persistence_warning);
    });

    rsx! {
        div { class: "{shell_class}",
            link { rel: "stylesheet", href: asset!("/assets/app.css") }
            div { class: "app-backdrop" }
            div { class: "app-atmosphere" }

            div { class: "app-content",
                if let Some(message) = persistence_warning() {
                    div { class: "app-warning-banner app-warning-banner-content app-warning-banner-strong",
                        "{message}"
                    }
                }

                Router::<Route> {}
            }

            if let Some(modal) = active_modal() {
                {render_modal_overlay(modal, active_modal, registry)}
            }
        }
    }
}
