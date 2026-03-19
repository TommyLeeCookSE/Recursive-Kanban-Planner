use crate::interface::Route;
use crate::interface::app::IsDark;
use crate::interface::components::modal::ModalType;
use crate::interface::components::visuals::{
    render_back_icon, render_book_icon, render_day_night_icon, render_label_icon,
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
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();
    let registry = use_context::<Signal<crate::domain::registry::CardRegistry>>();
    let persistence_warning = use_context::<Signal<Option<String>>>();
    let nav = navigator();

    rsx! {
        div { class: "flex min-h-0 flex-1 flex-col selection:bg-sunfire/30 selection:text-white",
                nav { class: "app-navbar",
                    div { class: "flex min-w-0 flex-wrap items-center gap-3 sm:gap-8",
                        Link {
                            to: Route::Home {},
                            class: "group flex min-w-0 items-center gap-3",
                            div { class: "flex h-10 w-10 items-center justify-center rounded-xl bg-sunfire shadow-lg shadow-sunfire/30 transition-transform group-hover:rotate-12",
                                span { class: "text-white text-xl font-black", "K" }
                            }
                            span { class: "app-text-primary hidden text-2xl font-black tracking-tighter transition-colors group-hover:text-sunfire sm:inline",
                                "Kanban"
                            }
                        }
                    }

                    div { class: "flex min-w-0 flex-wrap items-center justify-end gap-2 sm:gap-3",
                        {render_export_button(registry, persistence_warning)}
                        {render_import_button(registry, active_modal, persistence_warning, nav)}
                        {render_clear_cache_button(registry, active_modal, persistence_warning, nav)}
                        button {
                            class: "app-utility-button inline-flex items-center gap-2",
                            title: "Manage reusable card labels",
                            "aria-label": "Labels",
                            onclick: move |_| active_modal.set(Some(ModalType::ManageLabels {})),
                            span { class: "shrink-0", {render_label_icon()} }
                            span { class: "hidden sm:inline", "Labels" }
                        }
                        button {
                            class: "app-utility-button inline-flex items-center gap-2",
                            title: "Manage card automation rules",
                            "aria-label": "Rules",
                            onclick: move |_| active_modal.set(Some(ModalType::ManageRules {})),
                            span { class: "shrink-0", {render_book_icon()} }
                            span { class: "hidden sm:inline", "Rules" }
                        }
                    }

                    div { class: "flex min-w-0 flex-wrap items-center gap-2 sm:gap-3",
                        button {
                            class: "app-button-secondary inline-flex items-center gap-2 px-3 py-2.5 text-sm sm:min-w-[7.5rem] sm:px-4 sm:py-3",
                            onclick: move |_| is_dark.set(IsDark(!is_dark().0)),
                            title: "Toggle Light/Dark Mode",
                            "aria-label": "Toggle Light/Dark Mode",
                            span { class: "shrink-0", {render_day_night_icon()} }
                            if is_dark().0 {
                                span { class: "hidden sm:inline", "Evening" }
                            } else {
                                span { class: "hidden sm:inline", "Sunrise" }
                            }
                        }
                    }
                }
            }

            main { class: "flex-1 min-h-0 overflow-auto",
                Outlet::<Route> {}
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
pub fn TopBar(title: String, back_route: Route, back_label: String, children: Element) -> Element {
    rsx! {
        div { class: "app-panel border-b px-4 py-6 sm:px-6 lg:px-12",
            div { class: "grid w-full grid-cols-[minmax(0,auto)_minmax(0,1fr)_minmax(0,auto)] items-center gap-3 sm:gap-4 lg:gap-8",
                div { class: "min-w-0 justify-self-start",
                    button {
                        class: "app-button-secondary inline-flex min-h-[3.25rem] max-w-full items-center justify-center gap-2 rounded-2xl px-3 py-3 text-sm font-black sm:min-w-[12rem] sm:px-6 sm:text-base group",
                        onclick: move |_| {
                            navigator().push(back_route.clone());
                        },
                        title: "Back to {back_label}",
                        "aria-label": "Back to {back_label}",
                        span { class: "shrink-0 transform transition-transform group-hover:-translate-x-1", {render_back_icon()} }
                        span { class: "hidden truncate sm:inline", "Back to: {back_label}" }
                    }
                }

                div { class: "min-w-0 px-2 text-center justify-self-center",
                    h1 { class: "app-text-primary mx-auto max-w-xl break-words text-2xl font-black tracking-tighter sm:text-4xl lg:max-w-2xl lg:text-5xl",
                        "{title}"
                    }
                    p { class: "app-kicker mt-2 block",
                        "Board Context / {back_label}"
                    }
                }

                div { class: "flex min-w-0 flex-wrap items-center justify-end gap-2 sm:gap-3 lg:gap-4 justify-self-end",
                    {children}
                }
            }
        }
    }
}
