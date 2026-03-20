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
    let nav = navigator();

    let theme_title = if is_dark().0 {
        "Switch to Sunrise mode"
    } else {
        "Switch to Evening mode"
    };

    rsx! {
        div { class: "app-root-shell",
            nav { class: "app-navbar",
                div { class: "app-navbar-section",
                    Link {
                        to: Route::Home {},
                        class: "app-navbar-brand group",
                        div { class: "app-navbar-brand-mark",
                            span { class: "app-navbar-brand-initial", "K" }
                        }
                        span { class: "app-navbar-brand-text",
                            "Kanban"
                        }
                    }
                }

                div { class: "app-navbar-actions",
                    {render_export_button(registry, persistence_warning)}
                    {render_import_button(registry, active_modal, persistence_warning, nav)}
                    {render_clear_cache_button(registry, active_modal, persistence_warning, nav)}
                }

                div { class: "app-navbar-trailing",
                    button {
                        class: "app-theme-toggle",
                        onclick: move |_| is_dark.set(IsDark(!is_dark().0)),
                        title: "{theme_title}",
                        "aria-label": "{theme_title}",
                        "aria-pressed": is_dark().0,
                        div { class: "app-theme-toggle-track",
                            span { class: "app-theme-toggle-icon app-theme-toggle-icon--sunrise", {render_sunrise_icon()} }
                            span { class: "app-theme-toggle-icon app-theme-toggle-icon--evening", {render_evening_icon()} }
                            div { class: "app-theme-toggle-thumb" }
                        }
                    }
                }
            }

            main { class: "app-router-main",
                Outlet::<Route> {}
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
///     TopBar {
///         title: "Roadmap".to_string(),
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
#[component]
pub fn TopBar(
    title: String,
    back_route: Option<Route>,
    back_label: String,
    children: Element,
) -> Element {
    let back_button = if let Some(route) = back_route {
        rsx! {
                button {
                    class: "app-topbar-back group",
                    onclick: move |_| {
                        let nav = navigator();
                        let destination = route.clone();
                        nav.push(destination);
                    },
                title: "Back to {back_label}",
                "aria-label": "Back to {back_label}",
                span { class: "app-topbar-back-icon", {render_back_icon()} }
                span { class: "app-topbar-back-label", "Back to: {back_label}" }
            }
        }
    } else {
        rsx! {
                button {
                    class: "app-topbar-back app-topbar-back--disabled group",
                disabled: true,
                title: "Back to {back_label}",
                "aria-label": "Back to {back_label}",
                "aria-disabled": "true",
                span { class: "app-topbar-back-icon", {render_back_icon()} }
                span { class: "app-topbar-back-label", "Back to: {back_label}" }
            }
        }
    };

    rsx! {
        div { class: "app-topbar-shell",
            div { class: "app-topbar-grid",
                div { class: "app-topbar-back-shell",
                    {back_button}
                }

                div { class: "app-topbar-title-group",
                    h1 { class: "app-topbar-title",
                        "{title}"
                    }
                    p { class: "app-topbar-context",
                        "Board Context / {back_label}"
                    }
                }

                div { class: "app-topbar-actions",
                    {children}
                }
            }
        }
    }
}
