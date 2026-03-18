use crate::application::{CardRuleEvent, Command, PopupNotification, evaluate_card_rules, execute};
use crate::domain::error::DomainError;
use crate::domain::id::{BucketId, CardId};
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::app::IsDragging;
use dioxus::prelude::*;
use std::str::FromStr;
use tracing::{Level, info, warn};

/// Executes a UI-originated command and routes failures to diagnostics plus the warning banner.
pub fn execute_command_with_feedback(
    command: Command,
    mut registry: Signal<CardRegistry>,
    warning_message: Signal<Option<String>>,
    log_target: &'static str,
    action_label: impl Into<String>,
) -> Result<(), DomainError> {
    let action_label = action_label.into();
    let result = {
        let mut reg = registry.write();
        execute(command, &mut reg)
    };

    report_result(result, warning_message, log_target, action_label)
}

/// Surfaces a command result through tracing, in-memory diagnostics, and the warning banner.
pub fn report_result(
    result: Result<(), DomainError>,
    mut warning_message: Signal<Option<String>>,
    log_target: &'static str,
    action_label: impl Into<String>,
) -> Result<(), DomainError> {
    let action_label = action_label.into();

    match result {
        Ok(()) => {
            warning_message.set(None);
            Ok(())
        }
        Err(error_value) => {
            let message = error_value.to_string();
            warn!(ui_target = log_target, action = %action_label, error = %error_value, "UI action failed");
            record_diagnostic(
                Level::WARN,
                log_target,
                format!("UI action '{action_label}' failed: {message}"),
            );
            warning_message.set(Some(message));
            Err(error_value)
        }
    }
}

/// Prepares a browser drag session with a concrete payload and explicit move semantics.
pub fn prime_drag_session(
    event: &DragEvent,
    log_target: &'static str,
    payload: impl Into<String>,
    mut is_dragging: Signal<IsDragging>,
) {
    let payload = payload.into();
    let data_transfer = event.data().data_transfer();

    is_dragging.set(IsDragging(true));

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
pub fn prime_drop_target(event: &DragEvent) {
    event.prevent_default();
    event.data().data_transfer().set_drop_effect("move");
}

pub fn dispatch_card_rule_event(
    card_id: CardId,
    event: CardRuleEvent,
    registry: Signal<CardRegistry>,
    mut popup_queue: Signal<Vec<PopupNotification>>,
    log_target: &'static str,
) -> Result<(), DomainError> {
    info!(card_id = %card_id, event = ?event, "Dispatching card rule event");
    record_diagnostic(
        Level::INFO,
        log_target,
        format!("Dispatching card rule event {event:?} for card {card_id}"),
    );

    let outcomes = evaluate_card_rules(card_id, event, &registry.read())?;
    if outcomes.is_empty() {
        return Ok(());
    }

    let mut queued = popup_queue.read().clone();
    for outcome in outcomes {
        info!(
            rule_name = %outcome.rule.name(),
            popup_title = %outcome.popup.title,
            "Queuing popup from rule"
        );
        record_diagnostic(
            Level::INFO,
            log_target,
            format!("Queued popup from rule '{}'", outcome.rule.name()),
        );
        queued.push(outcome.popup);
    }
    popup_queue.set(queued);
    Ok(())
}

pub fn dragged_root_card_id(event: &DragEvent, log_target: &'static str) -> Option<CardId> {
    let payload = read_drag_payload(event, log_target)?;
    let raw_id = payload.strip_prefix("root-card:")?;
    parse_card_id(raw_id, log_target, &payload)
}

pub fn dragged_bucket_id(event: &DragEvent, log_target: &'static str) -> Option<BucketId> {
    let payload = read_drag_payload(event, log_target)?;
    let raw_id = payload.strip_prefix("bucket:")?.split(':').next()?;
    parse_bucket_id(raw_id, log_target, &payload)
}

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

fn parse_bucket_id(raw_id: &str, log_target: &'static str, payload: &str) -> Option<BucketId> {
    BucketId::from_str(raw_id).ok().or_else(|| {
        warn!(
            ui_target = log_target,
            payload = %payload,
            "Failed to parse bucket drag payload"
        );
        record_diagnostic(
            Level::WARN,
            log_target,
            format!("Failed to parse bucket drag payload '{payload}'"),
        );
        None
    })
}
