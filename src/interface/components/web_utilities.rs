//! Web-specific utility components for workspace management.
//!
//! This module provides buttons and logic for exporting, importing, and
//! clearing the card registry, primarily leveraging browser APIs (via `web-sys`).
//!
//! For more on Rust's module system and documentation, see `docs/rust-for-python-devs.md`.

use crate::domain::registry::CardRegistry;
#[cfg(target_arch = "wasm32")]
use crate::infrastructure::repository::{AppPersistence, JsonRepository};
#[cfg(target_arch = "wasm32")]
use crate::interface::Route;
use crate::interface::components::modal::ModalType;
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

/// Renders a button that triggers a JSON export of the current registry.
///
/// In non-WASM targets, this renders a disabled placeholder.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     render_export_button(registry, persistence_warning)
/// }
/// ```
pub fn render_export_button(
    registry: Signal<CardRegistry>,
    persistence_warning: Signal<Option<String>>,
) -> Element {
    #[cfg(target_arch = "wasm32")]
    {
        let mut persistence_warning = persistence_warning;
        return rsx! {
            button {
                class: "app-utility-button app-utility-button-row",
                title: "Download a JSON backup of your workspace",
                "aria-label": "Export",
                onclick: move |_| {
                    let snapshot = registry.read().clone();
                    match export_registry_snapshot(&snapshot) {
                        Ok(()) => persistence_warning.set(None),
                        Err(error) => persistence_warning.set(Some(error.to_string())),
                    }
                },
                span { class: "app-icon-slot", {render_export_icon()} }
                span { class: "app-utility-button-label", "Export" }
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

/// Renders a button that opens a file picker to import a JSON registry.
///
/// In non-WASM targets, this renders a disabled placeholder.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     render_import_button(registry, active_modal, persistence_warning, nav)
/// }
/// ```
pub fn render_import_button(
    registry: Signal<CardRegistry>,
    active_modal: Signal<Option<ModalType>>,
    persistence_warning: Signal<Option<String>>,
    nav: Navigator,
) -> Element {
    #[cfg(target_arch = "wasm32")]
    {
        return rsx! {
            button {
                class: "app-utility-button app-utility-button-row",
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
                span { class: "app-icon-slot", {render_import_icon()} }
                span { class: "app-utility-button-label", "Import" }
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

/// Renders a button that clears the local cache and resets the registry.
///
/// In non-WASM targets, this renders a disabled placeholder.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     render_clear_cache_button(registry, active_modal, persistence_warning, nav)
/// }
/// ```
pub fn render_clear_cache_button(
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
                class: "app-danger-button app-utility-button-row",
                title: "Clear saved data and reset the workspace",
                "aria-label": "Clear Cache",
                onclick: move |_| {
                    match clear_workspace_with_confirmation() {
                        Ok(true) => {
                            registry.set(CardRegistry::new());
                            active_modal.set(None);
                            persistence_warning.set(None);
                            navigate_home(nav.clone());
                        }
                        Ok(false) => {}
                        Err(error) => persistence_warning.set(Some(error.to_string())),
                    }
                },
                span { class: "app-icon-slot", {render_trash_icon()} }
                span { class: "app-utility-button-label", "Clear Cache" }
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

#[cfg(target_arch = "wasm32")]
fn navigate_home(nav: Navigator) {
    nav.push(crate::interface::Route::Home {});
}
