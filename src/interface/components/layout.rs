use crate::domain::registry::CardRegistry;
#[cfg(target_arch = "wasm32")]
use crate::infrastructure::repository::{AppPersistence, JsonRepository};
use crate::interface::Route;
use crate::interface::components::modal::ModalType;
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
    let mut is_dark = use_context::<Signal<bool>>();
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();
    let registry = use_context::<Signal<CardRegistry>>();
    let persistence_warning = use_context::<Signal<Option<String>>>();
    let nav = navigator();

    rsx! {
        div { class: "min-h-screen flex flex-col selection:bg-sunfire/30 selection:text-white",
            nav { class: "app-navbar",
                div { class: "flex items-center gap-8",
                    Link {
                        to: Route::Home {},
                        class: "group flex items-center gap-3",
                        div { class: "flex h-10 w-10 items-center justify-center rounded-xl bg-sunfire shadow-lg shadow-sunfire/30 transition-transform group-hover:rotate-12",
                            span { class: "text-white text-xl font-black", "K" }
                        }
                        span { class: "app-text-primary text-2xl font-black tracking-tighter transition-colors group-hover:text-sunfire",
                            "Kanban"
                        }
                    }
                    button {
                        class: "app-button-primary ml-4 px-6 py-2.5",
                        onclick: move |_| active_modal.set(Some(ModalType::CreateCard {
                            parent_id: None,
                            bucket_id: None,
                        })),
                        span { class: "text-lg", "+" }
                        "New Board"
                    }
                }

                div { class: "flex items-center gap-4",
                    div { class: "hidden items-center gap-2 border-r pr-4 md:flex", style: "border-color: var(--app-border);",
                        {render_export_button(registry, persistence_warning)}
                        {render_import_button(registry, active_modal, persistence_warning, nav)}
                        {render_clear_cache_button(registry, active_modal, persistence_warning, nav)}
                    }
                    if cfg!(target_arch = "wasm32") {
                        span { class: "app-kicker",
                            "Web Utilities"
                        }
                    }

                    button {
                        class: "app-button-secondary min-w-[7.5rem] px-4 py-3 text-sm",
                        onclick: move |_| is_dark.set(!is_dark()),
                        title: "Toggle Light/Dark Mode",
                        if is_dark() {
                            span { "Sunrise" }
                        } else {
                            span { "Evening" }
                        }
                    }
                }
            }

            main { class: "flex-grow overflow-auto",
                Outlet::<Route> {}
            }
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
                class: "app-utility-button",
                title: "Download a JSON backup of your workspace",
                onclick: move |_| {
                    let snapshot = registry.read().clone();
                    match export_registry_snapshot(&snapshot) {
                        Ok(()) => persistence_warning.set(None),
                        Err(error) => persistence_warning.set(Some(error.to_string())),
                    }
                },
                "Export"
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
                class: "app-utility-button",
                title: "Replace your workspace with a validated JSON import",
                onclick: move |_| {
                    begin_import_flow(
                        registry,
                        active_modal,
                        persistence_warning,
                        nav.clone(),
                    );
                },
                "Import"
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
                class: "app-danger-button",
                title: "Clear saved data and reset the workspace",
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
                "Clear Cache"
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
///         primary_label: "Create Bucket".to_string(),
///         on_primary: move |_| {},
///         secondary_label: "Settings".to_string(),
///         on_secondary: move |_| {},
///     }
/// }
/// ```
#[component]
pub fn TopBar(
    title: String,
    back_route: Route,
    back_label: String,
    primary_label: String,
    on_primary: EventHandler<()>,
    secondary_label: String,
    on_secondary: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "app-panel border-b px-6 py-8 lg:px-12",
            div { class: "max-w-7xl mx-auto flex flex-col gap-8 lg:flex-row lg:items-end lg:justify-between",
                div { class: "flex items-start gap-6",
                    button {
                        class: "app-button-secondary mt-1.5 p-3 group",
                        onclick: move |_| {
                            navigator().push(back_route.clone());
                        },
                        span { class: "transform group-hover:-translate-x-1 block transition-transform", "Back" }
                    }
                    div { class: "min-w-0 border-r pr-8", style: "border-color: var(--app-border);",
                        span { class: "app-kicker mb-1 block",
                            "Board Context / {back_label}"
                        }
                        h1 { class: "app-text-primary max-w-2xl truncate text-5xl font-black tracking-tighter",
                            "{title}"
                        }
                    }
                }

                div { class: "flex flex-wrap items-center gap-4",
                    button {
                        class: "app-button-secondary h-14 px-8 text-sunfire",
                        onclick: move |_| on_primary.call(()),
                        span { class: "text-xl", "+" }
                        "{primary_label}"
                    }
                    button {
                        class: "app-button-secondary h-14 px-8",
                        onclick: move |_| on_secondary.call(()),
                        "{secondary_label}"
                    }
                }
            }
        }
    }
}
