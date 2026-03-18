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
        div { class: "min-h-screen flex flex-col bg-gray-50 dark:bg-gray-950 font-sans antialiased selection:bg-sunfire/30 selection:text-sunfire-dark",
            nav { class: "sticky top-0 z-40 h-20 flex items-center justify-between px-12 bg-white/90 dark:bg-gray-900/95 border-b border-gray-100 dark:border-gray-800/80 backdrop-blur-xl transition-all",
                div { class: "flex items-center gap-8",
                    Link {
                        to: Route::Home {},
                        class: "group flex items-center gap-3",
                        div { class: "h-10 w-10 bg-sunfire rounded-xl shadow-lg shadow-sunfire/30 flex items-center justify-center transform group-hover:rotate-12 transition-transform",
                            span { class: "text-white text-xl font-black", "K" }
                        }
                        span { class: "font-black text-2xl tracking-tighter text-gray-900 dark:text-white group-hover:text-sunfire transition-colors",
                            "Kanban"
                        }
                    }
                    button {
                        class: "ml-4 px-6 py-2.5 bg-gray-900 dark:bg-white text-white dark:text-gray-900 font-bold rounded-2xl shadow-xl hover:shadow-sunfire/10 hover:bg-sunfire dark:hover:bg-sunfire transition-all transform hover:-translate-y-0.5 active:translate-y-0 flex items-center gap-2",
                        onclick: move |_| active_modal.set(Some(ModalType::CreateCard {
                            parent_id: None,
                            bucket_id: None,
                        })),
                        span { class: "text-lg", "+" }
                        "New Board"
                    }
                }

                div { class: "flex items-center gap-4",
                    div { class: "hidden md:flex items-center gap-2 pr-4 border-r border-gray-100 dark:border-gray-800",
                        {render_export_button(registry, persistence_warning)}
                        {render_import_button(registry, active_modal, persistence_warning, nav)}
                        {render_clear_cache_button(registry, active_modal, persistence_warning, nav)}
                    }
                    if cfg!(target_arch = "wasm32") {
                        span { class: "text-[9px] font-black uppercase tracking-[0.3em] text-gray-300 dark:text-gray-700",
                            "Web Utilities"
                        }
                    }

                    button {
                        class: "p-3 rounded-2xl bg-gray-100 dark:bg-gray-800 text-gray-500 dark:text-gray-400 hover:text-sunfire hover:bg-sunfire/10 transition-all",
                        onclick: move |_| is_dark.set(!is_dark()),
                        title: "Toggle Light/Dark Mode",
                        if is_dark() {
                            span { "Light" }
                        } else {
                            span { "Dark" }
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
                class: "rounded-2xl px-4 py-2 text-xs font-black uppercase tracking-widest text-gray-500 hover:text-sunfire dark:text-gray-400 dark:hover:text-sunfire transition-colors",
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
                class: "rounded-2xl px-4 py-2 text-xs font-black uppercase tracking-widest text-gray-500 hover:text-sunfire dark:text-gray-400 dark:hover:text-sunfire transition-colors",
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
                class: "rounded-2xl px-4 py-2 text-xs font-black uppercase tracking-widest text-red-400 hover:text-red-500 dark:text-red-300 dark:hover:text-red-200 transition-colors",
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
            class: "cursor-not-allowed rounded-2xl px-4 py-2 text-xs font-black uppercase tracking-widest text-gray-300 dark:text-gray-600 border border-transparent opacity-70",
            disabled: true,
            title: "{title}",
            "{label}"
        }
        span { class: "text-[9px] font-black uppercase tracking-[0.3em] text-gray-300 dark:text-gray-700",
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
        div { class: "px-12 py-8 bg-white dark:bg-gray-900 border-b border-gray-100 dark:border-gray-800/60 shadow-sm",
            div { class: "max-w-7xl mx-auto flex flex-col gap-8 lg:flex-row lg:items-end lg:justify-between",
                div { class: "flex items-start gap-6",
                    button {
                        class: "mt-1.5 p-3 rounded-2xl border border-gray-100 dark:border-gray-800 text-gray-400 hover:text-sunfire hover:border-sunfire/50 transition-all group",
                        onclick: move |_| {
                            navigator().push(back_route.clone());
                        },
                        span { class: "transform group-hover:-translate-x-1 block transition-transform", "Back" }
                    }
                    div { class: "min-w-0 pr-8 border-r border-gray-100 dark:border-gray-800",
                        span { class: "text-[10px] font-black uppercase tracking-[0.5em] text-gray-300 dark:text-gray-700 block mb-1",
                            "Board Context / {back_label}"
                        }
                        h1 { class: "text-5xl font-black tracking-tighter text-gray-900 dark:text-white truncate max-w-2xl",
                            "{title}"
                        }
                    }
                }

                div { class: "flex flex-wrap items-center gap-4",
                    button {
                        class: "h-14 px-8 rounded-2xl border-2 border-sunfire/40 bg-sunfire/5 text-sunfire font-bold hover:bg-sunfire/10 transition-all flex items-center gap-3 active:scale-95 shadow-lg shadow-sunfire/5",
                        onclick: move |_| on_primary.call(()),
                        span { class: "text-xl", "+" }
                        "{primary_label}"
                    }
                    button {
                        class: "h-14 px-8 rounded-2xl border-2 border-gray-100 dark:border-gray-800 text-gray-500 dark:text-gray-400 font-bold hover:border-gray-300 dark:hover:border-gray-600 hover:text-gray-900 dark:hover:text-white transition-all active:scale-95",
                        onclick: move |_| on_secondary.call(()),
                        "{secondary_label}"
                    }
                }
            }
        }
    }
}
