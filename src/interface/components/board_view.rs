use crate::application::{CardRuleEvent, Command, PopupNotification, execute};
use crate::domain::card::UNASSIGNED_BUCKET_NAME;
use crate::domain::id::{BucketId, CardId};
use crate::domain::label::LabelColor;
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::Route;
use crate::interface::actions::{
    ReorderFeedbackContext, delete_card_with_feedback, dispatch_card_rule_event,
    execute_command_with_feedback, execute_reorder_with_feedback, prime_drag_session,
    prime_drop_target, report_result,
};
use crate::interface::app::IsDragging;
use crate::interface::components::card_item::CardItem;
use crate::interface::components::layout::TopBar;
use crate::interface::components::modal::ModalType;
use crate::interface::components::shared_forms::confirm_destructive_action;
use crate::interface::components::visuals::{
    CardDisplayData, DropZoneKind, drop_zone_classes, render_label_chip, render_label_icon,
    render_note_icon, render_plus_icon, render_settings_icon, render_trash_icon,
    surface_action_button_classes, surface_destructive_icon_button_classes,
    surface_icon_button_classes, toolbar_button_classes, toolbar_button_label_classes,
    toolbar_button_mobile_icon_classes,
};
use dioxus::prelude::*;
use tracing::{Level, info};

#[derive(Clone, Copy)]
pub(crate) struct BoardDragSignals {
    pub bucket_drop_index: Signal<Option<usize>>,
    pub card_drop_target: Signal<Option<(BucketId, usize)>>,
}

#[derive(Clone)]
pub(crate) struct BoardRenderContext {
    pub board_id: CardId,
    pub registry: Signal<CardRegistry>,
    pub active_modal: Signal<Option<ModalType>>,
    pub warning_message: Signal<Option<String>>,
    pub popup_queue: Signal<Vec<PopupNotification>>,
    pub drag: BoardDragSignals,
    pub is_dragging: Signal<IsDragging>,
}

#[derive(Clone)]
pub(crate) struct ColumnRenderModel {
    pub bucket_id: BucketId,
    pub bucket_name: String,
    pub cards: Vec<CardDisplayData>,
}

pub(crate) fn render_board_screen(
    board_title: String,
    back_route: Route,
    back_label: String,
    board_due_date: String,
    board_labels: Vec<(String, LabelColor)>,
    column_models: Vec<ColumnRenderModel>,
    render_context: BoardRenderContext,
) -> Element {
    rsx! {
        div { class: "flex h-full flex-col",
            {render_board_header(
                board_title,
                back_route,
                back_label,
                board_due_date,
                board_labels,
                render_context.clone(),
            )}
            div { class: "flex-grow overflow-x-auto px-6 py-10 lg:px-12",
                div { class: "flex min-w-max gap-4 lg:gap-8", style: "align-items: stretch;",
                    for (index, column) in column_models.iter().cloned().enumerate() {
                        {render_bucket_drop_zone(
                            render_context.board_id,
                            index,
                            render_context.drag,
                            render_context.registry,
                            render_context.warning_message,
                            render_context.is_dragging,
                        )}
                        {render_column(column, render_context.clone())}
                    }
                    {render_bucket_drop_zone(
                        render_context.board_id,
                        column_models.len(),
                        render_context.drag,
                        render_context.registry,
                        render_context.warning_message,
                        render_context.is_dragging,
                    )}
                }
            }
        }
    }
}

fn render_board_header(
    board_title: String,
    back_route: Route,
    back_label: String,
    board_due_date: String,
    board_labels: Vec<(String, LabelColor)>,
    context: BoardRenderContext,
) -> Element {
    let board_id = context.board_id;
    let mut active_modal = context.active_modal;
    let registry = context.registry;
    let popup_queue = context.popup_queue;
    let warning_message = context.warning_message;

    rsx! {
        TopBar {
            title: board_title,
            back_route,
            back_label: back_label.clone(),
            button {
                class: "{toolbar_button_classes()} text-sunfire",
                onclick: move |_| active_modal.set(Some(ModalType::CreateBucket { card_id: board_id })),
                title: "Create Bucket",
                "aria-label": "Create Bucket",
                span { class: "{toolbar_button_mobile_icon_classes()}", {render_plus_icon()} }
                span { class: "{toolbar_button_label_classes()}", "Create Bucket" }
            }
                button {
                    class: "{toolbar_button_classes()}",
                    onclick: move |_| {
                        let result = dispatch_card_rule_event(
                            board_id,
                        CardRuleEvent::NoteOpened,
                        registry,
                        popup_queue,
                        "board-route",
                    );
                    let _ = report_result(
                        result,
                        warning_message,
                        "board-route",
                        "dispatch note-opened rule",
                    );
                    active_modal.set(Some(ModalType::CardNotes { card_id: board_id }));
                    },
                    title: "Open notes",
                    "aria-label": "Notes",
                    span { class: "{toolbar_button_mobile_icon_classes()}", {render_note_icon()} }
                    span { class: "{toolbar_button_label_classes()}", "Notes" }
                }
            button {
                class: "{toolbar_button_classes()}",
                onclick: move |_| active_modal.set(Some(ModalType::CardLabels { card_id: board_id })),
                title: "Edit labels",
                "aria-label": "Labels",
                span { class: "{toolbar_button_mobile_icon_classes()}", {render_label_icon()} }
                span { class: "{toolbar_button_label_classes()}", "Labels" }
            }
                button {
                    class: "{toolbar_button_classes()}",
                    onclick: move |_| active_modal.set(Some(ModalType::EditCard { id: board_id })),
                    title: "Open settings",
                    "aria-label": "Settings",
                    span { class: "{toolbar_button_mobile_icon_classes()}", {render_settings_icon()} }
                    span { class: "{toolbar_button_label_classes()}", "Settings" }
                }
        }

        div { class: "app-panel flex flex-wrap items-center justify-between gap-4 border-b px-6 py-5 lg:px-12",
            div { class: "flex flex-wrap items-center gap-4",
                p { class: "app-kicker", "Status: Active | Due: {board_due_date} | Labels:" }
                if board_labels.is_empty() {
                    span { class: "app-text-muted text-xs font-black uppercase tracking-widest", "None" }
                } else {
                    div { class: "flex flex-wrap gap-2",
                        for (name, color) in board_labels {
                            {render_label_chip(name, color)}
                        }
                    }
                }
            }
        }
    }
}

fn render_column(column: ColumnRenderModel, context: BoardRenderContext) -> Element {
    let bucket_id = column.bucket_id;
    let bucket_name = column.bucket_name;
    let mut active_modal = context.active_modal;
    let warning_message = context.warning_message;
    let mut bucket_drop_index = context.drag.bucket_drop_index;
    let mut is_dragging = context.is_dragging;
    let can_delete_bucket = bucket_name != UNASSIGNED_BUCKET_NAME;
    let can_rename_bucket = can_delete_bucket;
    let bucket_name_for_delete = bucket_name.clone();
    let column_class = "app-column-surface group flex max-h-full w-80 flex-shrink-0 flex-col rounded-[2rem] p-5 transition-all hover:border-sunfire/30";

    rsx! {
        div {
            key: "{bucket_id}",
            class: "{column_class}",
            draggable: true,
            ondragstart: move |event| {
                prime_drag_session(
                    &event,
                    "board-route",
                    format!("bucket:{bucket_id}:board:{}", context.board_id),
                    is_dragging,
                );
                info!(bucket_id = %bucket_id, board_id = %context.board_id, "Started dragging bucket");
                record_diagnostic(
                    Level::INFO,
                    "board-route",
                    format!("Started dragging bucket {bucket_id} on board {}", context.board_id),
                );
            },
            ondragend: move |_| {
                bucket_drop_index.set(None);
                is_dragging.set(IsDragging(false));
            },
            div {
                class: "mb-6 flex flex-col gap-4 rounded-2xl px-3 py-2",
                div {
                    class: "flex flex-col items-center gap-3 text-center",
                    h2 {
                        class: "app-kicker text-sm leading-tight transition-colors group-hover:text-sunfire md:text-base",
                        "{bucket_name}"
                    }
                    div { class: "flex flex-wrap items-center justify-center gap-2",
                        if can_rename_bucket {
                            button {
                                class: "{surface_action_button_classes()}",
                                title: "Rename this bucket",
                                onclick: move |_| active_modal.set(Some(ModalType::EditBucket {
                                    card_id: context.board_id,
                                    bucket_id,
                                })),
                                "Rename"
                            }
                        }
                        if can_delete_bucket {
                            button {
                                class: "{surface_destructive_icon_button_classes()}",
                                title: "Delete this bucket",
                                onclick: move |_| {
                                    if confirm_destructive_action(&format!(
                                        "Delete the bucket '{bucket_name_for_delete}' and all cards inside it?"
                                    )) {
                                        let _ = execute_command_with_feedback(
                                            Command::RemoveBucket {
                                                card_id: context.board_id,
                                                bucket_id,
                                            },
                                            context.registry,
                                            warning_message,
                                            "board-route",
                                            format!(
                                                "delete bucket {bucket_id} from board {}",
                                                context.board_id
                                            ),
                                        );
                                    }
                                },
                                span { class: "shrink-0", {render_trash_icon()} }
                            }
                        }
                        button {
                            class: "{surface_icon_button_classes()} hover:rotate-90",
                            onclick: move |_| active_modal.set(Some(ModalType::CreateCard {
                                parent_id: Some(context.board_id),
                                bucket_id: Some(bucket_id),
                            })),
                            "+"
                        }
                    }
                }
            }
            div { class: "flex-grow overflow-y-auto space-y-4 pr-2",
                {render_card_drop_zone(bucket_id, 0, context.clone())}
                for (index, card) in column.cards.iter().cloned().enumerate() {
                    {render_card_item(card, bucket_id, index, context.clone())}
                }
            }
        }
    }
}

fn render_card_item(
    card: CardDisplayData,
    bucket_id: BucketId,
    index: usize,
    context: BoardRenderContext,
) -> Element {
    let card_id = card.id;
    let mut active_modal = context.active_modal;
    let mut card_drop_target = context.drag.card_drop_target;
    let mut is_dragging = context.is_dragging;

    rsx! {
        div {
            key: "{card_id}",
            class: "flex flex-col gap-3",
            CardItem {
                title: card.title,
                subtitle: format!("{} nested items", card.nested_item_count),
                draggable: true,
                on_open: move |_| {
                    navigator().push(Route::Board { card_id });
                },
                on_drag_start: move |event| {
                    prime_drag_session(
                        &event,
                        "board-route",
                        format!("card:{card_id}:board:{}", context.board_id),
                        is_dragging,
                    );
                    info!(card_id = %card_id, board_id = %context.board_id, "Started dragging card");
                    record_diagnostic(
                        Level::INFO,
                        "board-route",
                        format!("Started dragging card {card_id} on board {}", context.board_id),
                    );
                },
                on_drag_end: move |_| {
                    card_drop_target.set(None);
                    is_dragging.set(IsDragging(false));
                },
                on_rename: move |_| active_modal.set(Some(ModalType::EditCard { id: card_id })),
                due_date: card.due_date,
                is_overdue: card.is_overdue,
                labels: card.labels,
                preview_sections: card.preview_sections,
                on_delete: move |_| {
                    delete_card_with_feedback(
                        card_id,
                        context.registry,
                        context.warning_message,
                        "board-route",
                        format!("delete board card {card_id}"),
                    );
                },
            }
            {render_card_drop_zone(bucket_id, index + 1, context.clone())}
        }
    }
}

fn render_bucket_drop_zone(
    board_id: CardId,
    index: usize,
    drag: BoardDragSignals,
    registry: Signal<CardRegistry>,
    warning_message: Signal<Option<String>>,
    is_dragging: Signal<IsDragging>,
) -> Element {
    let mut bucket_drop_index = drag.bucket_drop_index;
    let is_active = bucket_drop_index() == Some(index);
    let class_name = drop_zone_classes(DropZoneKind::Bucket, is_active, is_dragging().0);

    rsx! {
        div {
            class: "{class_name}",
            ondragover: move |event| {
                prime_drop_target(&event);
                bucket_drop_index.set(Some(index));
            },
            ondragleave: move |_| {
                if bucket_drop_index() == Some(index) {
                    bucket_drop_index.set(None);
                }
            },
            ondrop: move |event| {
                event.prevent_default();

                let Some(bucket_id) =
                    crate::interface::actions::dragged_bucket_id(&event, "board-route")
                else {
                    return;
                };

                bucket_drop_index.set(None);

                let current_order: Vec<BucketId> = {
                    let reg = registry.read();
                    match reg.get_card(board_id) {
                        Ok(card) => card.buckets().iter().map(|bucket| bucket.id()).collect(),
                        Err(_) => return,
                    }
                };

                let _ = execute_reorder_with_feedback(
                    &current_order,
                    bucket_id,
                    index,
                    ReorderFeedbackContext::new(
                        registry,
                        warning_message,
                        "board-route",
                        format!("reorder buckets on board {board_id} with dragged bucket {bucket_id}"),
                    ),
                    |ordered_ids| Command::ReorderBuckets {
                        card_id: board_id,
                        ordered_ids,
                    },
                );
            },
            if is_dragging().0 {
                span { class: "rotate-90", "Drop" }
            }
        }
    }
}

fn render_card_drop_zone(
    bucket_id: BucketId,
    index: usize,
    context: BoardRenderContext,
) -> Element {
    let board_id = context.board_id;
    let mut registry = context.registry;
    let warning_message = context.warning_message;
    let popup_queue = context.popup_queue;
    let mut card_drop_target = context.drag.card_drop_target;
    let is_dragging = context.is_dragging;
    let is_active = card_drop_target() == Some((bucket_id, index));
    let class_name = drop_zone_classes(DropZoneKind::Card, is_active, is_dragging().0);

    rsx! {
        div {
            class: "{class_name}",
            ondragover: move |event| {
                prime_drop_target(&event);
                card_drop_target.set(Some((bucket_id, index)));
            },
            ondragleave: move |_| {
                if card_drop_target() == Some((bucket_id, index)) {
                    card_drop_target.set(None);
                }
            },
            ondrop: move |event| {
                event.prevent_default();

                let Some(card_id) =
                    crate::interface::actions::dragged_card_id(&event, "board-route")
                else {
                    return;
                };

                card_drop_target.set(None);
                let previous_bucket_id = registry
                    .read()
                    .get_card(card_id)
                    .ok()
                    .and_then(|card| card.bucket_id());

                let result = execute(
                    Command::DropCardAtPosition {
                        board_id,
                        card_id,
                        target_bucket_id: bucket_id,
                        target_index: index,
                    },
                    &mut registry.write(),
                );
                let outcome = report_result(
                    result,
                    warning_message,
                    "board-route",
                    format!("drop card {card_id} into bucket {bucket_id} on board {board_id}"),
                );
                if outcome.is_ok() && previous_bucket_id != Some(bucket_id) {
                    let result = dispatch_card_rule_event(
                        card_id,
                        CardRuleEvent::MovedToBucket(bucket_id),
                        registry,
                        popup_queue,
                        "board-route",
                    );
                    let _ = report_result(
                        result,
                        warning_message,
                        "board-route",
                        "dispatch moved-to-bucket rule",
                    );
                }
            },
            if is_dragging().0 {
                "Drop Here"
            }
        }
    }
}
