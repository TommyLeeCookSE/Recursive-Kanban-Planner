//! Browser and JavaScript interoperability for UI actions.
//!
//! This module provides wrappers for browser-specific APIs like View Transitions
//! and the HTML5 Drag-and-Drop API.
//!
//! For more on Rust's `wasm-bindgen` and JS interop, see
//! `docs/rust-for-python-devs.md`.

use crate::domain::id::CardId;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::app::{DraggedItemKind, IsDragging};
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use js_sys::{Function, Reflect};
use std::str::FromStr;
use tracing::{Level, warn};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

/// Runs a UI mutation inside the browser view-transition API when available.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::interface::actions::run_with_view_transition;
///
/// run_with_view_transition(move || {
///     show_modal.set(None);
/// });
/// ```
pub fn run_with_view_transition<F>(callback: F)
where
    F: FnOnce() + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            callback();
            return;
        };
        let Some(document) = window.document() else {
            callback();
            return;
        };

        let callback_js = Closure::once_into_js(callback);
        let callback_fn = callback_js.clone().dyn_into::<Function>().ok();

        let Ok(start_view_transition_value) =
            Reflect::get(document.as_ref(), &JsValue::from_str("startViewTransition"))
        else {
            if let Some(function) = callback_fn {
                let _ = function.call0(&JsValue::NULL);
            }
            return;
        };

        let Some(start_view_transition) = start_view_transition_value.dyn_into::<Function>().ok()
        else {
            if let Some(function) = callback_fn {
                let _ = function.call0(&JsValue::NULL);
            }
            return;
        };

        let _ = start_view_transition.call1(document.as_ref(), &callback_js);
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    callback();
}

/// Prepares a browser drag session with a concrete payload and explicit move semantics.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::interface::actions::prime_drag_session;
///
/// prime_drag_session(&event, "board", format!("card:{}", id), DraggedItemKind::Card, kind, active);
/// ```
pub fn prime_drag_session(
    event: &DragEvent,
    log_target: &'static str,
    payload: impl Into<String>,
    dragged_item_kind: DraggedItemKind,
    mut dragged_item_kind_signal: Signal<DraggedItemKind>,
    mut is_dragging: Signal<IsDragging>,
) {
    let payload = payload.into();
    let data_transfer = event.data().data_transfer();

    is_dragging.set(IsDragging(true));
    dragged_item_kind_signal.set(dragged_item_kind);

    if let Err(error) = data_transfer.clear_data(None) {
        warn!(
            ui_target = log_target,
            error = %error,
            "Failed to clear previous drag payload"
        );
        record_diagnostic(
            Level::WARN,
            log_target,
            format!("Failed to clear drag payload before drag start: {error}"),
        );
    }

    if let Err(error) = data_transfer.set_data("text/plain", &payload) {
        warn!(
            ui_target = log_target,
            payload = %payload,
            error = %error,
            "Failed to set drag payload"
        );
        record_diagnostic(
            Level::WARN,
            log_target,
            format!("Failed to set drag payload '{payload}': {error}"),
        );
    }

    data_transfer.set_effect_allowed("move");
    data_transfer.set_drop_effect("move");
}

/// Keeps the browser drag interaction in explicit move mode while a target is hovered.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::interface::actions::prime_drop_target;
///
/// prime_drop_target(&event);
/// ```
pub fn prime_drop_target(event: &DragEvent) {
    event.prevent_default();
    event.data().data_transfer().set_drop_effect("move");
}

/// Extracts a card ID from a browser drag event payload.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::interface::actions::dragged_card_id;
///
/// if let Some(id) = dragged_card_id(&event, "board") {
///     println!("Dragged card {}", id);
/// }
/// ```
pub fn dragged_card_id(event: &DragEvent, log_target: &'static str) -> Option<CardId> {
    let payload = read_drag_payload(event, log_target)?;
    let raw_id = payload.strip_prefix("card:")?.split(':').next()?;
    parse_card_id(raw_id, log_target, &payload)
}

fn read_drag_payload(event: &DragEvent, log_target: &'static str) -> Option<String> {
    let payload = event.data().data_transfer().get_as_text()?;
    if payload.is_empty() {
        record_diagnostic(
            Level::WARN,
            log_target,
            "Drag payload was unexpectedly empty",
        );
        return None;
    }
    Some(payload)
}

fn parse_card_id(raw_id: &str, log_target: &'static str, payload: &str) -> Option<CardId> {
    CardId::from_str(raw_id).ok().or_else(|| {
        warn!(
            ui_target = log_target,
            payload = %payload,
            "Failed to parse card drag payload"
        );
        record_diagnostic(
            Level::WARN,
            log_target,
            format!("Failed to parse card drag payload '{payload}'"),
        );
        None
    })
}

/// Displays a browser confirmation dialog for destructive actions.
///
/// Only functional in WASM targets; returns `true` automatically elsewhere.
///
/// # Examples
///
/// ```ignore
/// if confirm_destructive_action("Are you sure?") {
///     // delete something
/// }
/// ```
pub fn confirm_destructive_action(message: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|window| window.confirm_with_message(message).ok())
            .unwrap_or(false)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = message;
        true
    }
}
