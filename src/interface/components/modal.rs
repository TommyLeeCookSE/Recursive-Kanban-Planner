use crate::application::{CardRuleEvent, Command, PopupNotification, execute};
use crate::domain::due_date::DueDate;
use crate::domain::error::DomainError;
use crate::domain::id::{BucketId, CardId};
use crate::domain::label::LabelColor;
use crate::domain::registry::CardRegistry;
use crate::domain::rule::{RuleAction, RuleTrigger};
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::actions::{dispatch_card_rule_event, report_result};
use crate::interface::components::shared_forms::{
    inline_error, toggle_id, user_message_for_command_error,
};
use crate::interface::components::visuals::render_label_chip;
use dioxus::prelude::*;
use tracing::{Level, warn};

#[derive(Clone, Debug, PartialEq)]
pub enum ModalType {
    CreateCard {
        parent_id: Option<CardId>,
        bucket_id: Option<BucketId>,
    },
    EditCard {
        id: CardId,
    },
    CreateBucket {
        card_id: CardId,
    },
    CardNotes {
        card_id: CardId,
    },
    CardLabels {
        card_id: CardId,
    },
    ManageLabels {},
    ManageRules {},
}

#[component]
pub fn Modal(on_close: EventHandler<()>, title: String, children: Element) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4 backdrop-blur-sm animate-in fade-in duration-200",
            onclick: move |_| on_close.call(()),
            div {
                class: "app-modal-surface w-full max-w-md overflow-hidden rounded-[2rem] animate-in zoom-in-95 duration-200",
                onclick: |e| e.stop_propagation(),
                div { class: "flex items-center justify-between border-b px-6 py-4", style: "border-color: var(--app-border);",
                    h2 { class: "app-text-primary text-lg font-bold", "{title}" }
                    button { class: "app-button-ghost p-2", onclick: move |_| on_close.call(()), "X" }
                }
                div { class: "p-6", {children} }
            }
        }
    }
}

#[component]
pub fn CardModal(
    on_close: EventHandler<()>,
    parent_id: Option<CardId>,
    bucket_id: Option<BucketId>,
    registry: Signal<CardRegistry>,
) -> Element {
    let mut input_title = use_signal(String::new);
    let mut error_message = use_signal(|| None::<String>);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: if parent_id.is_some() { "New Child Card" } else { "New Root Board" },
            div { class: "flex flex-col gap-4",
                input {
                    class: "app-input",
                    placeholder: "Enter title...",
                    value: "{input_title}",
                    oninput: move |e| input_title.set(e.value()),
                    autofocus: true,
                }
                if let Some(message) = error_message() {
                    {inline_error(message)}
                }
                div { class: "flex justify-end gap-2",
                    button { class: "app-button-ghost px-4 py-2", onclick: move |_| on_close.call(()), "Cancel" }
                    button {
                        class: "app-button-primary px-6 py-3 disabled:opacity-50",
                        disabled: input_title().trim().is_empty(),
                        onclick: move |_| {
                            let trimmed_title = input_title().trim().to_string();
                            let command = match build_create_card_command(trimmed_title, parent_id, bucket_id) {
                                Ok(command) => command,
                                Err(error_value) => {
                                    let user_message = user_message_for_command_error(&error_value);
                                    warn!(
                                        parent_id = ?parent_id,
                                        bucket_id = ?bucket_id,
                                        error = %error_value,
                                        "Card modal rejected invalid create-card context"
                                    );
                                    error_message.set(Some(user_message.clone()));
                                    record_diagnostic(Level::WARN, "ui-modal", user_message);
                                    return;
                                }
                            };

                            let mut reg = registry.write();
                            match execute(command, &mut reg) {
                                Ok(()) => {
                                    error_message.set(None);
                                    on_close.call(());
                                }
                                Err(error_value) => error_message.set(Some(user_message_for_command_error(&error_value))),
                            }
                        },
                        "Create Item"
                    }
                }
            }
        }
    }
}

#[component]
pub fn BucketModal(
    on_close: EventHandler<()>,
    card_id: CardId,
    registry: Signal<CardRegistry>,
) -> Element {
    let mut input_name = use_signal(String::new);
    let mut error_message = use_signal(|| None::<String>);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "New Column",
            div { class: "flex flex-col gap-4",
                input {
                    class: "app-input",
                    placeholder: "Column Name (e.g., Todo, Doing)",
                    value: "{input_name}",
                    oninput: move |e| input_name.set(e.value()),
                    autofocus: true,
                }
                if let Some(message) = error_message() { {inline_error(message)} }
                div { class: "flex justify-end gap-2",
                    button { class: "app-button-ghost px-4 py-2", onclick: move |_| on_close.call(()), "Cancel" }
                    button {
                        class: "app-button-primary px-6 py-3 disabled:opacity-50",
                        disabled: input_name().trim().is_empty(),
                        onclick: move |_| {
                            let mut reg = registry.write();
                            match execute(Command::AddBucket { card_id, name: input_name().trim().to_string() }, &mut reg) {
                                Ok(()) => {
                                    error_message.set(None);
                                    on_close.call(());
                                }
                                Err(error_value) => error_message.set(Some(user_message_for_command_error(&error_value))),
                            }
                        },
                        "Add Column"
                    }
                }
            }
        }
    }
}

#[component]
pub fn EditCardModal(
    on_close: EventHandler<()>,
    id: CardId,
    registry: Signal<CardRegistry>,
) -> Element {
    let popup_queue = use_context::<Signal<Vec<PopupNotification>>>();
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();
    let warning_message = use_context::<Signal<Option<String>>>();

    let card_snapshot = {
        let reg = registry.read();
        let card = match reg.get_card(id) {
            Ok(card) => card.clone(),
            Err(_) => {
                return rsx! {
                    Modal { on_close: move |_| on_close.call(()), title: "Edit Item".to_string(),
                        p { class: "app-text-muted", "Card could not be loaded." }
                    }
                };
            }
        };
        (
            card,
            reg.label_definitions().to_vec(),
            reg.rule_definitions().to_vec(),
        )
    };

    let (card, label_definitions, rule_definitions) = card_snapshot;
    let initial_title = card.title().to_string();
    let initial_due_date = card
        .due_date()
        .map(|due| due.to_string())
        .unwrap_or_default();
    let initial_labels = card.label_ids().to_vec();
    let initial_rules = card.rule_ids().to_vec();
    let mut input_title = use_signal(move || initial_title.clone());
    let mut due_date_input = use_signal(move || initial_due_date.clone());
    let mut selected_labels = use_signal(move || initial_labels.clone());
    let mut selected_rules = use_signal(move || initial_rules.clone());
    let mut error_message = use_signal(|| None::<String>);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "Edit Item",
            div { class: "flex flex-col gap-4",
                label { class: "app-kicker", "Title" }
                input {
                    class: "app-input",
                    value: "{input_title}",
                    oninput: move |e| input_title.set(e.value()),
                    autofocus: true,
                }

                label { class: "app-kicker", "Due Date" }
                input {
                    class: "app-input",
                    r#type: "date",
                    value: "{due_date_input}",
                    oninput: move |e| due_date_input.set(e.value()),
                }

                div { class: "flex items-center justify-between gap-4",
                    label { class: "app-kicker", "Labels" }
                    button {
                        class: "app-button-secondary px-4 py-2 text-xs",
                        onclick: move |_| active_modal.set(Some(ModalType::ManageLabels {})),
                        "Manage Labels"
                    }
                }
                if label_definitions.is_empty() {
                    p { class: "app-text-muted text-sm", "No labels created yet." }
                } else {
                    div { class: "space-y-3",
                        for label in label_definitions.iter().cloned() {
                            label { class: "flex items-center gap-3 rounded-xl border px-4 py-3", style: "border-color: var(--app-border);",
                                input {
                                    r#type: "checkbox",
                                    checked: selected_labels().contains(&label.id()),
                                    onclick: move |_| toggle_id(&mut selected_labels, label.id()),
                                }
                                span { class: "app-text-primary text-sm font-medium", "{label.name()}" }
                            }
                        }
                    }
                }

                div { class: "flex items-center justify-between gap-4",
                    label { class: "app-kicker", "Rules" }
                    button {
                        class: "app-button-secondary px-4 py-2 text-xs",
                        onclick: move |_| active_modal.set(Some(ModalType::ManageRules {})),
                        "Manage Rules"
                    }
                }
                if rule_definitions.is_empty() {
                    p { class: "app-text-muted text-sm", "No rules created yet." }
                } else {
                    div { class: "space-y-3",
                        for rule in rule_definitions.iter().cloned() {
                            label { class: "flex items-center gap-3 rounded-xl border px-4 py-3", style: "border-color: var(--app-border);",
                                input {
                                    r#type: "checkbox",
                                    checked: selected_rules().contains(&rule.id()),
                                    onclick: move |_| toggle_id(&mut selected_rules, rule.id()),
                                }
                                span { class: "app-text-primary text-sm font-medium", "{rule.name()}" }
                            }
                        }
                    }
                }

                if let Some(message) = error_message() { {inline_error(message)} }

                div { class: "flex flex-wrap justify-between gap-3",
                    button {
                        class: "app-button-secondary px-4 py-2",
                        onclick: move |_| {
                            let result = dispatch_card_rule_event(
                                id,
                                CardRuleEvent::NoteOpened,
                                registry,
                                popup_queue,
                                "ui-modal",
                            );
                            let _ = report_result(
                                result,
                                warning_message,
                                "ui-modal",
                                "dispatch note-opened rule",
                            );
                            active_modal.set(Some(ModalType::CardNotes { card_id: id }));
                        },
                        "Open Notes"
                    }
                    div { class: "flex gap-2",
                        button { class: "app-button-ghost px-4 py-2", onclick: move |_| on_close.call(()), "Cancel" }
                        button {
                            class: "app-button-primary px-6 py-3 disabled:opacity-50",
                            disabled: input_title().trim().is_empty(),
                            onclick: move |_| {
                                let due_date_value = if due_date_input().trim().is_empty() {
                                    None
                                } else {
                                    match DueDate::parse(due_date_input()) {
                                        Ok(due_date) => Some(due_date),
                                        Err(error_value) => {
                                            error_message.set(Some(user_message_for_command_error(&error_value)));
                                            return;
                                        }
                                    }
                                };

                                let mut reg = registry.write();
                                let rename_result = execute(
                                    Command::RenameCard { id, title: input_title().trim().to_string() },
                                    &mut reg,
                                );
                                let due_result = match due_date_value {
                                    Some(due_date) => execute(Command::SetDueDate { card_id: id, due_date }, &mut reg),
                                    None => execute(Command::ClearDueDate { card_id: id }, &mut reg),
                                };
                                let labels_result = execute(
                                    Command::SetCardLabels { card_id: id, label_ids: selected_labels() },
                                    &mut reg,
                                );
                                let rules_result = execute(
                                    Command::SetCardRules { card_id: id, rule_ids: selected_rules() },
                                    &mut reg,
                                );

                                if let Some(error_value) = rename_result
                                    .err()
                                    .or_else(|| due_result.err())
                                    .or_else(|| labels_result.err())
                                    .or_else(|| rules_result.err())
                                {
                                    error_message.set(Some(user_message_for_command_error(&error_value)));
                                } else {
                                    error_message.set(None);
                                    on_close.call(());
                                }
                            },
                            "Save Changes"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn CardLabelsModal(
    on_close: EventHandler<()>,
    card_id: CardId,
    registry: Signal<CardRegistry>,
) -> Element {
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();
    let card_snapshot = {
        let reg = registry.read();
        let card = match reg.get_card(card_id) {
            Ok(card) => card.clone(),
            Err(_) => {
                return rsx! {
                    Modal { on_close: move |_| on_close.call(()), title: "Edit Labels".to_string(),
                        p { class: "app-text-muted", "Card could not be loaded." }
                    }
                };
            }
        };
        (card, reg.label_definitions().to_vec())
    };

    let (card, label_definitions) = card_snapshot;
    let initial_labels = card.label_ids().to_vec();
    let mut selected_labels = use_signal(move || initial_labels.clone());
    let mut error_message = use_signal(|| None::<String>);

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "Edit Labels",
            div { class: "flex flex-col gap-4",
                div { class: "flex items-center justify-between gap-4",
                    label { class: "app-kicker", "Assigned Labels" }
                    button {
                        class: "app-button-secondary px-4 py-2 text-xs",
                        onclick: move |_| active_modal.set(Some(ModalType::ManageLabels {})),
                        "Manage Labels"
                    }
                }
                if label_definitions.is_empty() {
                    p { class: "app-text-muted text-sm", "No labels created yet." }
                } else {
                    div { class: "space-y-3",
                        for label in label_definitions.iter().cloned() {
                            label { class: "flex items-center gap-3 rounded-xl border px-4 py-3", style: "border-color: var(--app-border);",
                                input {
                                    r#type: "checkbox",
                                    checked: selected_labels().contains(&label.id()),
                                    onclick: move |_| toggle_id(&mut selected_labels, label.id()),
                                }
                                span { class: "app-text-primary text-sm font-medium", "{label.name()}" }
                            }
                        }
                    }
                }

                if let Some(message) = error_message() { {inline_error(message)} }

                div { class: "flex justify-end gap-2",
                    button { class: "app-button-ghost px-4 py-2", onclick: move |_| on_close.call(()), "Cancel" }
                    button {
                        class: "app-button-primary px-6 py-3",
                        onclick: move |_| {
                            let mut reg = registry.write();
                            match execute(Command::SetCardLabels { card_id, label_ids: selected_labels() }, &mut reg) {
                                Ok(()) => {
                                    error_message.set(None);
                                    on_close.call(());
                                }
                                Err(error_value) => error_message.set(Some(user_message_for_command_error(&error_value))),
                            }
                        },
                        "Save Labels"
                    }
                }
            }
        }
    }
}

#[component]
pub fn NotesModal(
    on_close: EventHandler<()>,
    card_id: CardId,
    registry: Signal<CardRegistry>,
) -> Element {
    let popup_queue = use_context::<Signal<Vec<PopupNotification>>>();
    let warning_message = use_context::<Signal<Option<String>>>();
    let notes = {
        let reg = registry.read();
        match reg.get_card(card_id) {
            Ok(card) => card.notes().to_vec(),
            Err(_) => Vec::new(),
        }
    };

    let initial_selected = notes.first().map(|note| note.id());
    let mut selected_note_id = use_signal(move || initial_selected);
    let mut title_input = use_signal(|| {
        notes
            .first()
            .map(|note| note.title().to_string())
            .unwrap_or_default()
    });
    let mut body_input = use_signal(|| {
        notes
            .first()
            .map(|note| note.body().to_string())
            .unwrap_or_default()
    });
    let mut new_page_title = use_signal(|| "New Page".to_string());
    let mut error_message = use_signal(|| None::<String>);

    let current_notes = {
        let reg = registry.read();
        reg.get_card(card_id)
            .map(|card| card.notes().to_vec())
            .unwrap_or_default()
    };

    let selected_note = current_notes
        .iter()
        .find(|note| Some(note.id()) == selected_note_id());

    rsx! {
        Modal {
            on_close: move |_| {
                let result = dispatch_card_rule_event(
                    card_id,
                    CardRuleEvent::NoteClosed,
                    registry,
                    popup_queue,
                    "ui-modal",
                );
                let _ = report_result(
                    result,
                    warning_message,
                    "ui-modal",
                    "dispatch note-closed rule",
                );
                on_close.call(());
            },
            title: "Notebook",
            div { class: "flex max-h-[75vh] flex-col gap-4 lg:flex-row",
                div { class: "w-full space-y-3 lg:w-56",
                    input {
                        class: "app-input",
                        placeholder: "New page title",
                        value: "{new_page_title}",
                        oninput: move |e| new_page_title.set(e.value()),
                    }
                    button {
                        class: "app-button-secondary w-full px-4 py-2 text-sm",
                        onclick: move |_| {
                            let title = new_page_title().trim().to_string();
                            let mut reg = registry.write();
                            match execute(Command::AddNotePage { card_id, title }, &mut reg) {
                                Ok(()) => {
                                    let latest_note = reg.get_card(card_id).ok().and_then(|card| card.notes().last().cloned());
                                    if let Some(note) = latest_note {
                                        selected_note_id.set(Some(note.id()));
                                        title_input.set(note.title().to_string());
                                        body_input.set(note.body().to_string());
                                    }
                                    new_page_title.set("New Page".to_string());
                                    error_message.set(None);
                                }
                                Err(error_value) => error_message.set(Some(user_message_for_command_error(&error_value))),
                            }
                        },
                        "Add Page"
                    }
                    div { class: "max-h-64 space-y-2 overflow-y-auto pr-1",
                        for note in current_notes.iter().cloned() {
                            button {
                                class: if Some(note.id()) == selected_note_id() { "app-button-primary w-full px-4 py-2 text-left text-sm" } else { "app-button-secondary w-full px-4 py-2 text-left text-sm" },
                                onclick: move |_| {
                                    selected_note_id.set(Some(note.id()));
                                    title_input.set(note.title().to_string());
                                    body_input.set(note.body().to_string());
                                },
                                "{note.title()}"
                            }
                        }
                    }
                }

                div { class: "flex min-h-[24rem] flex-1 flex-col gap-3",
                    if selected_note.is_some() {
                        input {
                            class: "app-input",
                            placeholder: "Page title",
                            value: "{title_input}",
                            oninput: move |e| title_input.set(e.value()),
                        }
                        textarea {
                            class: "app-input min-h-[18rem]",
                            placeholder: "Write your notes here...",
                            value: "{body_input}",
                            oninput: move |e| body_input.set(e.value()),
                        }
                        div { class: "flex flex-wrap justify-end gap-2",
                            button {
                                class: "app-button-secondary px-4 py-2",
                                onclick: move |_| {
                                    let Some(note_id) = selected_note_id() else { return; };
                                    let mut reg = registry.write();
                                    let rename_result = execute(
                                        Command::RenameNotePage { card_id, note_page_id: note_id, title: title_input().trim().to_string() },
                                        &mut reg,
                                    );
                                    let save_result = execute(
                                        Command::SaveNotePageBody { card_id, note_page_id: note_id, body: body_input() },
                                        &mut reg,
                                    );
                                    if let Some(error_value) = rename_result.err().or_else(|| save_result.err()) {
                                        error_message.set(Some(user_message_for_command_error(&error_value)));
                                    } else {
                                        error_message.set(None);
                                    }
                                },
                                "Save Page"
                            }
                            button {
                                class: "app-danger-button px-4 py-2",
                                onclick: move |_| {
                                    let Some(note_id) = selected_note_id() else { return; };
                                    let mut reg = registry.write();
                                    match execute(Command::DeleteNotePage { card_id, note_page_id: note_id }, &mut reg) {
                                        Ok(()) => {
                                            let fallback = reg.get_card(card_id).ok().and_then(|card| card.notes().first().cloned());
                                            selected_note_id.set(fallback.as_ref().map(|note| note.id()));
                                            title_input.set(fallback.as_ref().map(|note| note.title().to_string()).unwrap_or_default());
                                            body_input.set(fallback.as_ref().map(|note| note.body().to_string()).unwrap_or_default());
                                            error_message.set(None);
                                        }
                                        Err(error_value) => error_message.set(Some(user_message_for_command_error(&error_value))),
                                    }
                                },
                                "Delete Page"
                            }
                        }
                    } else {
                        div { class: "app-empty-state flex min-h-[18rem] items-center justify-center rounded-[1.5rem] p-6 text-center",
                            p { class: "app-text-muted", "No note pages yet. Add one to start writing." }
                        }
                    }
                }
                if let Some(message) = error_message() { {inline_error(message)} }
            }
        }
    }
}

#[component]
pub fn ManageLabelsModal(on_close: EventHandler<()>, registry: Signal<CardRegistry>) -> Element {
    let mut name_input = use_signal(String::new);
    let mut color_input = use_signal(|| LabelColor::Ember.as_str().to_string());
    let mut error_message = use_signal(|| None::<String>);
    let labels = registry.read().label_definitions().to_vec();

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "Manage Labels",
            div { class: "flex flex-col gap-4",
                input { class: "app-input", placeholder: "Label name", value: "{name_input}", oninput: move |e| name_input.set(e.value()) }
                select {
                    class: "app-input",
                    value: "{color_input}",
                    oninput: move |e| color_input.set(e.value()),
                    for color in LabelColor::ALL {
                        option { value: "{color.as_str()}", "{color.as_str()}" }
                    }
                }
                button {
                    class: "app-button-primary px-6 py-3",
                    onclick: move |_| {
                        let color = parse_label_color(&color_input());
                        let mut reg = registry.write();
                        match execute(Command::CreateLabelDefinition { name: name_input().trim().to_string(), color }, &mut reg) {
                            Ok(()) => {
                                name_input.set(String::new());
                                error_message.set(None);
                            }
                            Err(error_value) => error_message.set(Some(user_message_for_command_error(&error_value))),
                        }
                    },
                    "Create Label"
                }
                if let Some(message) = error_message() { {inline_error(message)} }
                div { class: "space-y-3",
                    for label in labels.iter().cloned() {
                        div { class: "flex items-center justify-between gap-3 rounded-xl border px-4 py-3", style: "border-color: var(--app-border);",
                            div { class: "flex items-center gap-3",
                                {render_label_chip(label.name().to_string(), label.color())}
                            }
                            button {
                                class: "app-danger-button px-3 py-2",
                                onclick: move |_| {
                                    let mut reg = registry.write();
                                if execute(Command::DeleteLabelDefinition { label_id: label.id() }, &mut reg).is_err() {}
                                },
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ManageRulesModal(on_close: EventHandler<()>, registry: Signal<CardRegistry>) -> Element {
    let mut name_input = use_signal(String::new);
    let mut trigger_kind = use_signal(|| "NoteOpened".to_string());
    let mut popup_title = use_signal(String::new);
    let mut popup_message = use_signal(String::new);
    let mut selected_bucket_id = use_signal(String::new);
    let mut error_message = use_signal(|| None::<String>);
    let rules = registry.read().rule_definitions().to_vec();
    let bucket_choices = collect_bucket_choices(&registry.read());

    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            title: "Manage Rules",
            div { class: "flex flex-col gap-4",
                input { class: "app-input", placeholder: "Rule name", value: "{name_input}", oninput: move |e| name_input.set(e.value()) }
                select {
                    class: "app-input",
                    value: "{trigger_kind}",
                    oninput: move |e| trigger_kind.set(e.value()),
                    option { value: "NoteOpened", "Note Opened" }
                    option { value: "NoteClosed", "Note Closed" }
                    option { value: "MovedToBucket", "Moved To Bucket" }
                }
                if trigger_kind() == "MovedToBucket" {
                    select {
                        class: "app-input",
                        value: "{selected_bucket_id}",
                        oninput: move |e| selected_bucket_id.set(e.value()),
                        option { value: "", "Select a bucket" }
                        for (id, label) in bucket_choices.iter().cloned() {
                            option { value: "{id}", "{label}" }
                        }
                    }
                }
                input { class: "app-input", placeholder: "Popup title", value: "{popup_title}", oninput: move |e| popup_title.set(e.value()) }
                textarea { class: "app-input min-h-[10rem]", placeholder: "Popup message", value: "{popup_message}", oninput: move |e| popup_message.set(e.value()) }
                button {
                    class: "app-button-primary px-6 py-3",
                    onclick: move |_| {
                        let trigger = match build_rule_trigger(&trigger_kind(), &selected_bucket_id()) {
                            Ok(trigger) => trigger,
                            Err(error_value) => {
                                error_message.set(Some(user_message_for_command_error(&error_value)));
                                return;
                            }
                        };
                        let mut reg = registry.write();
                        match execute(
                            Command::CreateRuleDefinition {
                                name: name_input().trim().to_string(),
                                trigger,
                                action: RuleAction::ShowPopup {
                                    title: popup_title().trim().to_string(),
                                    message: popup_message().trim().to_string(),
                                },
                            },
                            &mut reg,
                        ) {
                            Ok(()) => {
                                name_input.set(String::new());
                                popup_title.set(String::new());
                                popup_message.set(String::new());
                                selected_bucket_id.set(String::new());
                                trigger_kind.set("NoteOpened".to_string());
                                error_message.set(None);
                            }
                            Err(error_value) => error_message.set(Some(user_message_for_command_error(&error_value))),
                        }
                    },
                    "Create Rule"
                }
                if let Some(message) = error_message() { {inline_error(message)} }
                div { class: "space-y-3",
                    for rule in rules.iter().cloned() {
                        div { class: "rounded-xl border px-4 py-3", style: "border-color: var(--app-border);",
                            div { class: "mb-2 flex items-center justify-between gap-3",
                                h3 { class: "app-text-primary font-semibold", "{rule.name()}" }
                                button {
                                    class: "app-danger-button px-3 py-2",
                                    onclick: move |_| {
                                        let mut reg = registry.write();
                                        if execute(Command::DeleteRuleDefinition { rule_id: rule.id() }, &mut reg).is_err() {}
                                    },
                                    "Delete"
                                }
                            }
                            p { class: "app-text-muted text-sm", "{describe_rule_trigger(rule.trigger())}" }
                            p { class: "app-text-soft mt-1 text-sm", "{describe_rule_action(rule.action())}" }
                        }
                    }
                }
            }
        }
    }
}

fn build_create_card_command(
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

fn parse_label_color(raw: &str) -> LabelColor {
    match raw {
        "Gold" => LabelColor::Gold,
        "Moss" => LabelColor::Moss,
        "Sky" => LabelColor::Sky,
        "Indigo" => LabelColor::Indigo,
        "Rose" => LabelColor::Rose,
        _ => LabelColor::Ember,
    }
}

fn build_rule_trigger(kind: &str, bucket_id: &str) -> Result<RuleTrigger, DomainError> {
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

fn describe_rule_trigger(trigger: &RuleTrigger) -> String {
    match trigger {
        RuleTrigger::NoteOpened => "When a card notebook is opened".to_string(),
        RuleTrigger::NoteClosed => "When a card notebook is closed".to_string(),
        RuleTrigger::MovedToBucket(bucket_id) => {
            format!("When a card is moved to bucket {bucket_id}")
        }
    }
}

fn describe_rule_action(action: &RuleAction) -> String {
    match action {
        RuleAction::ShowPopup { title, message } => format!("Show popup '{title}': {message}"),
    }
}

fn collect_bucket_choices(registry: &CardRegistry) -> Vec<(String, String)> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_child_card_requires_bucket_id() {
        let parent_id = CardId::default();
        let error = build_create_card_command("Child".to_string(), Some(parent_id), None)
            .expect_err("missing child bucket should be rejected");

        assert!(
            matches!(error, DomainError::InvalidOperation(message) if message.contains("destination column"))
        );
    }

    #[test]
    fn create_child_card_command_preserves_bucket_id() {
        let parent_id = CardId::default();
        let bucket_id = BucketId::default();

        let command =
            build_create_card_command("Child".to_string(), Some(parent_id), Some(bucket_id))
                .expect("valid child command should be created");

        match command {
            Command::CreateChildCard {
                parent_id: actual_parent_id,
                bucket_id: actual_bucket_id,
                ..
            } => {
                assert_eq!(actual_parent_id, parent_id);
                assert_eq!(actual_bucket_id, bucket_id);
            }
            _ => panic!("expected child card command"),
        }
    }
}
