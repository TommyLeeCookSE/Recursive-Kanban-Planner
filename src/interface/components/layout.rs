use crate::interface::Route;
use crate::interface::app::{IsDark, RouteMotionDirection};
use crate::interface::components::modal::ModalType;
use crate::interface::components::visuals::{render_back_icon, render_day_night_icon};
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
    let theme_label = if is_dark().0 { "Evening" } else { "Sunrise" };

    rsx! {
        div { class: "flex min-h-0 flex-1 flex-col selection:bg-sunfire/30 selection:text-white",
            nav { class: "app-navbar",
                div { class: "min-w-0 shrink-0",
                    Link {
                        to: Route::Home {},
                        class: "app-navbar-brand group",
                        div { class: "app-navbar-brand-mark",
                            span { class: "text-white text-xl font-black", "K" }
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

                div { class: "flex min-w-0 shrink-0 flex-wrap items-center gap-2 sm:gap-3",
                    button {
                        class: "app-navbar-theme-toggle",
                        onclick: move |_| is_dark.set(IsDark(!is_dark().0)),
                        title: "{theme_title}",
                        "aria-label": "{theme_title}",
                        "aria-pressed": is_dark().0,
                        span { class: "shrink-0", {render_day_night_icon()} }
                        span { class: "hidden sm:inline", "{theme_label}" }
                    }
                }
            }

            main { class: "flex-1 min-h-0 overflow-auto",
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
    let mut route_motion = use_context::<Signal<RouteMotionDirection>>();
    let back_button = if let Some(route) = back_route {
        rsx! {
                button {
                    class: "app-topbar-back group",
                    onclick: move |_| {
                        let nav = navigator();
                        let destination = route.clone();
                        route_motion.set(RouteMotionDirection::Backward);
                        nav.push(destination);
                    },
                title: "Back to {back_label}",
                "aria-label": "Back to {back_label}",
                span { class: "inline-flex shrink-0 items-center justify-center text-lg leading-none transform transition-transform group-hover:-translate-x-1", {render_back_icon()} }
                span { class: "hidden truncate sm:inline", "Back to: {back_label}" }
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
                span { class: "inline-flex shrink-0 items-center justify-center text-lg leading-none", {render_back_icon()} }
                span { class: "hidden truncate sm:inline", "Back to: {back_label}" }
            }
        }
    };

    rsx! {
        div { class: "app-topbar-shell",
            div { class: "app-topbar-grid",
                div { class: "min-w-0 shrink-0 justify-self-start",
                    {back_button}
                }

                div { class: "min-w-0 flex-1 basis-[18rem] px-2 text-center",
                    h1 { class: "app-topbar-title",
                        "{title}"
                    }
                    p { class: "app-kicker mt-2 block",
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
