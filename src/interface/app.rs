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
    let is_dark = use_context_provider(|| Signal::new(IsDark(true)));
    // Global modal state.
    let active_modal = use_context_provider(|| Signal::new(None::<ModalType>));
    let _is_dragging = use_context_provider(|| Signal::new(IsDragging(false)));
    let _dragged_item_kind = use_context_provider(|| Signal::new(DraggedItemKind::None));

    let shell_class = if is_dark().0 {
        "app-shell theme-dark dark"
    } else {
        "app-shell theme-light"
    };
    let registry_snapshot = registry.read().clone();
    use_effect(use_reactive!(|(registry_snapshot,)| {
        persist_registry_snapshot(&registry_snapshot, persistence_warning);
    }));

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
