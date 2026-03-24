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
    render_back_icon, render_evening_icon, render_search_icon, render_sunrise_icon,
};
use crate::interface::components::web_utilities::{
    render_clear_cache_button, render_download_logs_button, render_export_button,
    render_import_button,
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
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();
    let registry = use_context::<Signal<crate::domain::registry::CardRegistry>>();
    let persistence_warning = use_context::<Signal<Option<String>>>();
    let route = use_route::<Route>();
    let nav = navigator();

    let theme_title = if is_dark().0 {
        "Switch to Sunrise mode"
    } else {
        "Switch to Evening mode"
    };

    rsx! {
        div { class: "app-root-shell flex flex-col h-screen",
            // Principal Top Bar
            nav { class: "app-bar app-bar--top sticky top-0 z-50",
                div { class: "app-bar-left",
                    Link { to: Route::Home {}, class: "app-navbar-brand group",
                        div { class: "app-navbar-brand-mark",
                            span { class: "app-navbar-brand-initial", "K" }
                        }
                        span { class: "app-navbar-brand-text", "Kanban" }
                    }
                }

                // Title moved to ContextBar below

                div { class: "app-bar-right ml-auto",
                    button {
                        class: "app-bar-button h-10 w-10 flex items-center justify-center rounded-full hover:bg-[var(--app-surface-soft)] transition-colors",
                        onclick: move |_| active_modal.set(Some(ModalType::Search)),
                        title: "Search Workspace (Ctrl+K)",
                        span { class: "app-bar-button-icon", {render_search_icon()} }
                    }
                    {render_download_logs_button(persistence_warning)}
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
                            span { class: "app-theme-toggle-icon app-theme-toggle-icon--sunrise",
                                {render_sunrise_icon()}
                            }
                            span { class: "app-theme-toggle-icon app-theme-toggle-icon--evening",
                                {render_evening_icon()}
                            }
                        }
                    }
                }
            }

            // Context Bar (Secondary Bar)
            ContextBar { route }

            main { class: "app-router-main flex-1 overflow-auto",
                Outlet::<Route> {}
            }
        }
    }
}

/// A secondary navigation bar that displays details about the currently active card.
#[component]
pub fn ContextBar(route: Route) -> Element {
    let registry = use_context::<Signal<crate::domain::registry::CardRegistry>>();
    let signals = registry.read();

    let active_card = match route {
        Route::Board { card_id } => signals.get_card(card_id).ok(),
        Route::Map { focus_card_id } => signals.get_card(focus_card_id).ok(),
        _ => signals.workspace_card().ok(),
    };

    let (title, description, due_date) = match active_card {
        Some(card) => (
            card.title().to_string(),
            card.description().map(|s| s.to_string()),
            card.due_date().map(|d| d.to_string()),
        ),
        None => ("My Workspace".to_string(), None, None),
    };

    rsx! {
        div {
            class: "sticky top-[var(--app-bar-height)] z-40 bg-[var(--app-surface-soft)] border-b border-[var(--app-border)] backdrop-blur-md px-6 flex items-center gap-6 overflow-hidden",
            style: "height: var(--app-context-bar-height); min-height: 48px;",

            // Card Title Section
            div { class: "flex items-center gap-3 min-w-0 flex-initial",
                h2 { class: "text-[var(--app-text)] font-black text-lg tracking-tight truncate uppercase opacity-90",
                    "{title}"
                }
            }

            // Separator if needed
            if due_date.is_some() || description.is_some() {
                div { class: "w-px h-6 bg-[var(--app-border-strong)] opacity-20" }
            }

            // Due Date
            if let Some(due) = due_date {
                div { class: "flex items-center gap-2 text-[var(--app-text-muted)] text-sm whitespace-nowrap",
                    span { class: "text-sunfire", {crate::interface::components::visuals::render_calendar_icon()} }
                    span { "{due}" }
                }
            }

            // Description - truncate heavily to keep it one-line
            if let Some(desc) = description {
                div { class: "flex items-center gap-2 text-[var(--app-text-soft)] text-sm min-w-0 max-w-xl",
                    span { {crate::interface::components::visuals::render_note_icon()} }
                    span { class: "truncate italic", "{desc}" }
                }
            }
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
                span { class: "app-bar-button-label", "Back" }
            }
        }
    } else {
        rsx! {
            button {
                class: "app-bar-button group",
                disabled: true,
                title: "Back to {back_label}",
                span { class: "app-bar-button-icon", {render_back_icon()} }
                span { class: "app-bar-button-label", "Back" }
            }
        }
    };

    rsx! {
        footer { class: "app-bar app-bar--bottom app-bar--distributed sticky bottom-0 z-50",
            div { class: "app-bar-left contents", {back_button} }

            div { class: "app-bar-right contents", {children} }
        }
    }
}
