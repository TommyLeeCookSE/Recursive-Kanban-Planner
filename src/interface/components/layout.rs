//! High-level layout components for the Kanban Planner.
//!
//! This module provides the shell, navigation bar, and top bar components
//! that structure the overall user interface.
//!
//! For an overview of the UI layout, see
//! `docs/rust-for-python-devs.md`.

use crate::interface::Route;
use crate::interface::app::IsDark;
use crate::interface::components::modal::ModalType;
use crate::interface::components::visuals::{
    render_back_icon, render_evening_icon, render_sunrise_icon,
};
use crate::interface::components::web_utilities::{
    render_clear_cache_button, render_export_button, render_import_button,
};
use dioxus::prelude::*;
// use_route is imported through the router module or prelude depending on features
// In Dioxus 0.7 it should be accessible from the main prelude if router feature is on

/// The main layout wrapper with a sticky top navigation bar.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     NavbarLayout {}
/// }
/// ```
#[component]
pub fn NavbarLayout() -> Element {
    let mut is_dark = use_context::<Signal<IsDark>>();
    let active_modal = use_context::<Signal<Option<ModalType>>>();
    let registry = use_context::<Signal<crate::domain::registry::CardRegistry>>();
    let persistence_warning = use_context::<Signal<Option<String>>>();
    let route = use_route::<Route>();
    let nav = navigator();

    let active_title = match route {
        Route::Board { card_id } => registry
            .read()
            .get_card(card_id)
            .ok()
            .map(|c| c.title().to_string()),
        _ => None,
    };

    let theme_title = if is_dark().0 {
        "Switch to Sunrise mode"
    } else {
        "Switch to Evening mode"
    };

    rsx! {
        div { class: "app-root-shell",
            nav { class: "app-bar app-bar--top",
                div { class: "app-bar-left",
                    Link {
                        to: Route::Home {},
                        class: "app-navbar-brand group",
                        div { class: "app-navbar-brand-mark",
                            span { class: "app-navbar-brand-initial", "K" }
                        }
                        span { class: "app-navbar-brand-text", "Kanban" }
                    }
                }

                if let Some(title) = active_title {
                    div { class: "app-bar-center",
                        div { class: "app-navbar-title-shell",
                            h1 { class: "app-navbar-title app-navbar-title--hero", "{title}" }
                        }
                    }
                }

                div { class: "app-bar-right",
                    {render_export_button(registry, persistence_warning)}
                    {render_import_button(registry, active_modal, persistence_warning, nav)}
                    {render_clear_cache_button(registry, active_modal, persistence_warning, nav)}
                    button {
                        class: "app-theme-toggle",
                        onclick: move |_| is_dark.set(IsDark(!is_dark().0)),
                        title: "{theme_title}",
                        "aria-label": "{theme_title}",
                        "aria-pressed": is_dark().0,
                        div { class: "app-theme-toggle-track",
                            span { class: "app-theme-toggle-icon app-theme-toggle-icon--sunrise", {render_sunrise_icon()} }
                            span { class: "app-theme-toggle-icon app-theme-toggle-icon--evening", {render_evening_icon()} }
                        }
                    }
                }
            }

            main { class: "app-router-main", Outlet::<Route> {} }
        }
    }
}

/// A secondary navigation bar used within specific boards for context and local actions.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     BottomBar {
///         back_route: Route::Home {},
///         back_label: "Workspace".to_string(),
///         button {
///             class: "app-button-secondary h-14 px-8 text-sunfire",
///             onclick: move |_| {},
///             "Action"
///         }
///     }
/// }
/// ```
/// A smart navigation bar that automatically collapses labels based on content width.
#[component]
pub fn BottomBar(back_route: Option<Route>, back_label: String, children: Element) -> Element {
    let back_button = if let Some(route) = back_route {
        rsx! {
            button {
                class: "app-bar-button group",
                onclick: move |_| {
                    navigator().push(route.clone());
                },
                title: "Back to {back_label}",
                span { class: "app-bar-button-icon", {render_back_icon()} }
                span { class: "app-bar-button-label", "Back to: {back_label}" }
            }
        }
    } else {
        rsx! {
            button {
                class: "app-bar-button group",
                disabled: true,
                span { class: "app-bar-button-icon", {render_back_icon()} }
                span { class: "app-bar-button-label", "Back to: {back_label}" }
            }
        }
    };

    rsx! {
        footer { class: "app-bar app-bar--bottom",
            div { class: "app-bar-left",
                {back_button}
            }

            div { class: "app-bar-right",
                {children}
            }
        }
    }
}
