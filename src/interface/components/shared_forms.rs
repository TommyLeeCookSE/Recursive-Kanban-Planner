use crate::application::Command;
use crate::domain::due_date::DueDate;
use crate::domain::error::DomainError;
use crate::domain::id::{BucketId, CardId};
use crate::domain::label::LabelColor;
use crate::domain::registry::CardRegistry;
use crate::domain::rule::{RuleAction, RuleTrigger};
use dioxus::prelude::*;

pub fn inline_error(message: String) -> Element {
    rsx! {
        p { class: "rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-900/50 dark:bg-red-950/40 dark:text-red-200",
            "{message}"
        }
    }
}

pub fn user_message_for_command_error(error: &DomainError) -> String {
    error.to_string()
}

pub fn toggle_id<T>(signal: &mut Signal<Vec<T>>, id: T)
where
    T: Copy + Eq + 'static,
{
    let mut values = signal();
    if values.contains(&id) {
        values.retain(|value| *value != id);
    } else {
        values.push(id);
    }
    signal.set(values);
}

pub fn build_create_card_command(
    title: String,
    parent_id: Option<CardId>,
    bucket_id: Option<BucketId>,
) -> Result<Command, DomainError> {
    match parent_id {
        Some(parent_id) => {
            let bucket_id = bucket_id.ok_or_else(|| {
                DomainError::InvalidOperation(
                    "Unable to create a child card because no destination column was selected."
                        .to_string(),
                )
            })?;

            Ok(Command::CreateChildCard {
                title,
                parent_id,
                bucket_id,
            })
        }
        None => Ok(Command::CreateRootCard { title }),
    }
}

pub fn parse_label_color(raw: &str) -> LabelColor {
    match raw {
        "Gold" => LabelColor::Gold,
        "Moss" => LabelColor::Moss,
        "Sky" => LabelColor::Sky,
        "Indigo" => LabelColor::Indigo,
        "Rose" => LabelColor::Rose,
        _ => LabelColor::Ember,
    }
}

pub fn build_rule_trigger(kind: &str, bucket_id: &str) -> Result<RuleTrigger, DomainError> {
    match kind {
        "NoteOpened" => Ok(RuleTrigger::NoteOpened),
        "NoteClosed" => Ok(RuleTrigger::NoteClosed),
        "MovedToBucket" => Ok(RuleTrigger::MovedToBucket(bucket_id.parse().map_err(
            |_| {
                DomainError::InvalidOperation(
                    "A bucket-trigger rule requires a selected bucket".to_string(),
                )
            },
        )?)),
        _ => Err(DomainError::InvalidOperation(
            "Unknown rule trigger".to_string(),
        )),
    }
}

pub fn describe_rule_trigger(trigger: &RuleTrigger) -> String {
    match trigger {
        RuleTrigger::NoteOpened => "When a card notebook is opened".to_string(),
        RuleTrigger::NoteClosed => "When a card notebook is closed".to_string(),
        RuleTrigger::MovedToBucket(bucket_id) => {
            format!("When a card is moved to bucket {bucket_id}")
        }
    }
}

pub fn describe_rule_action(action: &RuleAction) -> String {
    match action {
        RuleAction::ShowPopup { title, message } => format!("Show popup '{title}': {message}"),
    }
}

pub fn collect_bucket_choices(registry: &CardRegistry) -> Vec<(String, String)> {
    let mut choices = Vec::new();
    for root in registry.get_root_cards() {
        collect_bucket_choices_for_card(
            root.id(),
            root.title().to_string(),
            registry,
            &mut choices,
        );
    }
    choices
}

fn collect_bucket_choices_for_card(
    card_id: CardId,
    card_title: String,
    registry: &CardRegistry,
    choices: &mut Vec<(String, String)>,
) {
    if let Ok(card) = registry.get_card(card_id) {
        for bucket in card.buckets() {
            choices.push((
                bucket.id().to_string(),
                format!("{card_title} / {}", bucket.name()),
            ));
        }
        for child_id in card.children_ids() {
            if let Ok(child) = registry.get_card(*child_id) {
                collect_bucket_choices_for_card(
                    child.id(),
                    child.title().to_string(),
                    registry,
                    choices,
                );
            }
        }
    }
}

pub fn due_date_string(due_date: Option<&DueDate>) -> String {
    due_date.map(|due| due.to_string()).unwrap_or_default()
}
