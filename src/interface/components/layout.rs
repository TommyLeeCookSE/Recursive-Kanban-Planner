use crate::domain::registry::CardRegistry;
#[cfg(target_arch = "wasm32")]
use crate::infrastructure::repository::{AppPersistence, JsonRepository};
use crate::interface::Route;
use crate::interface::app::IsDark;
use crate::interface::components::modal::ModalType;
use crate::interface::components::visuals::{
    render_back_icon, render_book_icon, render_day_night_icon, render_label_icon,
};
#[cfg(target_arch = "wasm32")]
use crate::interface::components::visuals::{
    render_export_icon, render_import_icon, render_trash_icon,
};
use dioxus::prelude::*;
use dioxus_router::Navigator;
#[cfg(target_arch = "wasm32")]
use js_sys::Array;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, HtmlAnchorElement, HtmlInputElement, Url};

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
    let registry = use_context::<Signal<CardRegistry>>();
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
                        button {
                            class: "app-button-primary inline-flex items-center gap-2 px-4 py-2.5 sm:px-6",
                            onclick: move |_| active_modal.set(Some(ModalType::CreateCard {
                                parent_id: None,
                                bucket_id: None,
                            })),
                            title: "Create a new board",
                            "aria-label": "New Board",
                            span { class: "text-lg", "+" }
                            span { class: "hidden sm:inline", "New Board" }
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
                        if cfg!(target_arch = "wasm32") {
                            span { class: "app-kicker hidden lg:inline",
                                "Web Utilities"
                            }
                        }

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

fn render_export_button(
    registry: Signal<CardRegistry>,
    persistence_warning: Signal<Option<String>>,
) -> Element {
    #[cfg(target_arch = "wasm32")]
    {
        let mut persistence_warning = persistence_warning;
        return rsx! {
            button {
                class: "app-utility-button inline-flex items-center gap-2",
                title: "Download a JSON backup of your workspace",
                "aria-label": "Export",
                onclick: move |_| {
                    let snapshot = registry.read().clone();
                    match export_registry_snapshot(&snapshot) {
                        Ok(()) => persistence_warning.set(None),
                        Err(error) => persistence_warning.set(Some(error.to_string())),
                    }
                },
                span { class: "shrink-0", {render_export_icon()} }
                span { class: "hidden sm:inline", "Export" }
            }
        };
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = registry;
        let _ = persistence_warning;
        disabled_utility_button("Export", "Export is available on web builds only")
    }
}

fn render_import_button(
    registry: Signal<CardRegistry>,
    active_modal: Signal<Option<ModalType>>,
    persistence_warning: Signal<Option<String>>,
    nav: Navigator,
) -> Element {
    #[cfg(target_arch = "wasm32")]
    {
        return rsx! {
            button {
                class: "app-utility-button inline-flex items-center gap-2",
                title: "Replace your workspace with a validated JSON import",
                "aria-label": "Import",
                onclick: move |_| {
                    begin_import_flow(
                        registry,
                        active_modal,
                        persistence_warning,
                        nav.clone(),
                    );
                },
                span { class: "shrink-0", {render_import_icon()} }
                span { class: "hidden sm:inline", "Import" }
            }
        };
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = registry;
        let _ = active_modal;
        let _ = persistence_warning;
        let _ = nav;
        disabled_utility_button("Import", "Import is available on web builds only")
    }
}

fn render_clear_cache_button(
    registry: Signal<CardRegistry>,
    active_modal: Signal<Option<ModalType>>,
    persistence_warning: Signal<Option<String>>,
    nav: Navigator,
) -> Element {
    #[cfg(target_arch = "wasm32")]
    {
        let mut registry = registry;
        let mut active_modal = active_modal;
        let mut persistence_warning = persistence_warning;
        return rsx! {
            button {
                class: "app-danger-button inline-flex items-center gap-2",
                title: "Clear saved data and reset the workspace",
                "aria-label": "Clear Cache",
                onclick: move |_| {
                    match clear_workspace_with_confirmation() {
                        Ok(true) => {
                            registry.set(CardRegistry::new());
                            active_modal.set(None);
                            persistence_warning.set(None);
                            nav.push(Route::Home {});
                        }
                        Ok(false) => {}
                        Err(error) => persistence_warning.set(Some(error.to_string())),
                    }
                },
                span { class: "shrink-0", {render_trash_icon()} }
                span { class: "hidden sm:inline", "Clear Cache" }
            }
        };
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = registry;
        let _ = active_modal;
        let _ = persistence_warning;
        let _ = nav;
        disabled_utility_button("Clear Cache", "Clear Cache is available on web builds only")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn disabled_utility_button(label: &str, title: &str) -> Element {
    rsx! {
        button {
            class: "app-utility-button-disabled",
            disabled: true,
            title: "{title}",
            "aria-label": "{label}",
            "{label}"
        }
        span { class: "app-kicker",
            "Soon"
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn export_registry_snapshot(
    registry: &CardRegistry,
) -> Result<(), crate::domain::error::DomainError> {
    let json = JsonRepository::serialize_registry(registry)?;
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| {
            crate::domain::error::DomainError::InvalidOperation(
                "Failed to access browser document for export".into(),
            )
        })?;

    let data = Array::new();
    data.push(&wasm_bindgen::JsValue::from_str(&json));
    let blob = Blob::new_with_str_sequence(&data).map_err(|_| {
        crate::domain::error::DomainError::InvalidOperation("Failed to prepare export blob".into())
    })?;
    let url = Url::create_object_url_with_blob(&blob).map_err(|_| {
        crate::domain::error::DomainError::InvalidOperation("Failed to create export URL".into())
    })?;

    let anchor: HtmlAnchorElement = document
        .create_element("a")
        .map_err(|_| {
            crate::domain::error::DomainError::InvalidOperation(
                "Failed to create export anchor".into(),
            )
        })?
        .dyn_into()
        .map_err(|_| {
            crate::domain::error::DomainError::InvalidOperation(
                "Failed to cast export anchor".into(),
            )
        })?;
    anchor.set_href(&url);
    anchor.set_download("kanban-planner-export.json");
    anchor.click();
    let _ = Url::revoke_object_url(&url);

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn begin_import_flow(
    registry: Signal<CardRegistry>,
    active_modal: Signal<Option<ModalType>>,
    persistence_warning: Signal<Option<String>>,
    nav: Navigator,
) {
    let document = match web_sys::window().and_then(|window| window.document()) {
        Some(document) => document,
        None => {
            let mut persistence_warning = persistence_warning;
            persistence_warning.set(Some("Failed to access browser document for import".into()));
            return;
        }
    };

    let input: HtmlInputElement = match document.create_element("input") {
        Ok(element) => match element.dyn_into() {
            Ok(input) => input,
            Err(_) => {
                let mut persistence_warning = persistence_warning;
                persistence_warning.set(Some("Failed to create file input for import".into()));
                return;
            }
        },
        Err(_) => {
            let mut persistence_warning = persistence_warning;
            persistence_warning.set(Some("Failed to create file input for import".into()));
            return;
        }
    };

    input.set_type("file");
    input.set_accept(".json,application/json");

    let input_for_change = input.clone();
    let onchange = Closure::<dyn FnMut(_)>::wrap(Box::new(move |_event: web_sys::Event| {
        let Some(file) = input_for_change.files().and_then(|files| files.get(0)) else {
            return;
        };

        let Ok(reader) = web_sys::FileReader::new() else {
            let mut persistence_warning = persistence_warning;
            persistence_warning.set(Some("Failed to create a file reader for import".into()));
            return;
        };

        let reader_for_load = reader.clone();
        let mut registry = registry;
        let mut active_modal = active_modal;
        let mut persistence_warning = persistence_warning;
        let nav = nav.clone();

        let onload =
            Closure::<dyn FnMut(_)>::wrap(Box::new(move |_event: web_sys::ProgressEvent| {
                let result = match reader_for_load.result() {
                    Ok(result) => result,
                    Err(_) => {
                        persistence_warning
                            .set(Some("Failed to read the selected import file".into()));
                        return;
                    }
                };

                let Some(json) = result.as_string() else {
                    persistence_warning.set(Some("Import file could not be read as text".into()));
                    return;
                };

                let confirmed = web_sys::window()
                    .and_then(|window| {
                        window
                            .confirm_with_message(
                                "Importing will replace the current workspace. Continue?",
                            )
                            .ok()
                    })
                    .unwrap_or(false);

                if !confirmed {
                    return;
                }

                match JsonRepository::deserialize_registry(&json) {
                    Ok(imported_registry) => {
                        registry.set(imported_registry);
                        active_modal.set(None);
                        persistence_warning.set(None);
                        nav.push(Route::Home {});
                    }
                    Err(error) => persistence_warning.set(Some(error.to_string())),
                }
            }));

        reader.set_onloadend(Some(onload.as_ref().unchecked_ref()));
        onload.forget();

        if reader.read_as_text(&file).is_err() {
            persistence_warning.set(Some(
                "Failed to begin reading the selected import file".into(),
            ));
        }
    }));

    input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
    onchange.forget();
    input.click();
}

#[cfg(target_arch = "wasm32")]
fn clear_workspace_with_confirmation() -> Result<bool, crate::domain::error::DomainError> {
    let window = web_sys::window().ok_or_else(|| {
        crate::domain::error::DomainError::InvalidOperation(
            "Failed to access browser window for clear-cache confirmation".into(),
        )
    })?;

    let confirmed = window
        .confirm_with_message("This will clear saved board data and reset the workspace. Continue?")
        .map_err(|_| {
            crate::domain::error::DomainError::InvalidOperation(
                "Failed to show clear-cache confirmation dialog".into(),
            )
        })?;

    if !confirmed {
        return Ok(false);
    }

    AppPersistence::clear_registry()?;
    Ok(true)
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
