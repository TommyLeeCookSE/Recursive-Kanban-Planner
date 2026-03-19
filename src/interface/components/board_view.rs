use crate::application::Command;
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::Route;
use crate::interface::actions::{
    delete_card_with_feedback, execute_command_with_feedback, prime_drag_session, prime_drop_target,
};
use crate::interface::app::{DraggedItemKind, IsDragging};
use crate::interface::components::card_item::CardItem;
use crate::interface::components::layout::TopBar;
use crate::interface::components::modal::ModalType;
use crate::interface::components::visuals::{
    CardDisplayData, DropZoneKind, drop_zone_classes, render_note_icon, render_plus_icon,
    render_settings_icon, toolbar_action_icon_classes, toolbar_button_classes,
    toolbar_button_label_classes,
};
use dioxus::prelude::*;
use tracing::{Level, info};

#[derive(Clone, Copy)]
pub(crate) struct BoardDragSignals {
    pub card_drop_index: Signal<Option<usize>>,
}

#[derive(Clone)]
pub(crate) struct BoardRenderContext {
    pub board_id: CardId,
    pub registry: Signal<CardRegistry>,
    pub active_modal: Signal<Option<ModalType>>,
    pub warning_message: Signal<Option<String>>,
    pub drag: BoardDragSignals,
    pub dragged_item_kind: Signal<DraggedItemKind>,
    pub is_dragging: Signal<IsDragging>,
}

pub(crate) fn render_board_screen(
    board_title: String,
    back_route: Option<Route>,
    back_label: String,
    board_due_date: String,
    child_models: Vec<CardDisplayData>,
    render_context: BoardRenderContext,
) -> Element {
    rsx! {
        div { class: "flex h-full flex-col",
            {render_board_header(
                board_title,
                back_route,
                back_label,
                board_due_date,
                render_context.clone(),
            )}
            div { class: "flex-grow overflow-x-hidden px-6 py-10 lg:px-12",
                {render_board_grid(child_models, render_context)}
            }
        }
    }
}

fn render_board_header(
    board_title: String,
    back_route: Option<Route>,
    back_label: String,
    board_due_date: String,
    context: BoardRenderContext,
) -> Element {
    let board_id = context.board_id;
    let mut active_modal = context.active_modal;

    rsx! {
        TopBar {
            title: board_title,
            back_route,
            back_label: back_label.clone(),
            button {
                class: "{toolbar_button_classes()} text-sunfire",
                onclick: move |_| active_modal.set(Some(ModalType::CreateCard { parent_id: Some(board_id) })),
                title: "Create Card",
                "aria-label": "Create Card",
                span { class: "{toolbar_action_icon_classes()}", {render_plus_icon()} }
                span { class: "{toolbar_button_label_classes()}", "Create Card" }
            }
            button {
                class: "{toolbar_button_classes()}",
                onclick: move |_| {
                    active_modal.set(Some(ModalType::CardNotes { card_id: board_id }));
                },
                title: "Open notes",
                "aria-label": "Notes",
                span { class: "{toolbar_action_icon_classes()}", {render_note_icon()} }
                span { class: "{toolbar_button_label_classes()}", "Notes" }
            }
            button {
                class: "{toolbar_button_classes()}",
                onclick: move |_| active_modal.set(Some(ModalType::EditCard { id: board_id })),
                title: "Open settings",
                "aria-label": "Settings",
                span { class: "{toolbar_action_icon_classes()}", {render_settings_icon()} }
                span { class: "{toolbar_button_label_classes()}", "Settings" }
            }
        }

        div { class: "app-panel flex flex-wrap items-center justify-between gap-4 border-b px-6 py-5 lg:px-12",
            div { class: "flex flex-wrap items-center gap-4",
                p { class: "app-kicker", "Status: Active | Due: {board_due_date}" }
            }
        }
    }
}

fn render_board_grid(child_models: Vec<CardDisplayData>, context: BoardRenderContext) -> Element {
    rsx! {
        div { class: "flex flex-wrap items-stretch gap-4 lg:gap-6",
            if child_models.is_empty() {
                {render_empty_drop_zone(context.clone())}
            } else {
                div {
                    class: "flex flex-wrap items-stretch gap-4 lg:gap-6",
                    for (index, card) in child_models.iter().cloned().enumerate() {
                        {render_card_slot(
                            card,
                            index,
                            index + 1 == child_models.len(),
                            context.clone(),
                        )}
                    }
                }
            }
        }
    }
}

fn render_empty_drop_zone(context: BoardRenderContext) -> Element {
    let mut card_drop_index = context.drag.card_drop_index;
    let dragged_item_kind = context.dragged_item_kind;
    let is_dragging = context.is_dragging;
    let registry = context.registry;
    let warning_message = context.warning_message;
    let board_id = context.board_id;
    let is_active = card_drop_index() == Some(0);
    let class_name = drop_zone_classes(DropZoneKind::Board, is_active, dragged_item_kind());

    rsx! {
        div {
            class: "{class_name} min-h-[10rem] rounded-[1.75rem] border-2 border-dashed",
            ondragover: move |event| {
                prime_drop_target(&event);
                card_drop_index.set(Some(0));
            },
            ondragleave: move |_| {
                if card_drop_index() == Some(0) {
                    card_drop_index.set(None);
                }
            },
            ondrop: move |event| {
                event.prevent_default();

                let Some(card_id) = crate::interface::actions::dragged_card_id(&event, "board-route") else {
                    return;
                };

                card_drop_index.set(None);
                let _ = execute_command_with_feedback(
                    Command::DropChildAtPosition {
                        parent_id: board_id,
                        card_id,
                        target_index: 0,
                    },
                    registry,
                    warning_message,
                    "board-route",
                    format!("drop card {card_id} into empty board {board_id}"),
                );
            },
            if is_dragging().0 {
                div { class: "flex h-full items-center justify-center p-6 text-center",
                    span { class: "app-kicker", "Drop a card here" }
                }
            } else {
                div { class: "flex h-full items-center justify-center p-6 text-center",
                    p { class: "app-text-muted", "No child cards yet. Create one to start this board." }
                }
            }
        }
    }
}

fn render_card_slot(
    card: CardDisplayData,
    index: usize,
    is_last: bool,
    context: BoardRenderContext,
) -> Element {
    rsx! {
        div {
            class: "flex min-w-[18rem] flex-[1_1_18rem] items-stretch gap-2 lg:gap-3",
            {render_card_drop_zone(index, false, context.clone(), true)}
            div { class: "min-w-0 flex-1",
                {render_card_item(card, context.clone())}
            }
            if is_last {
                {render_card_drop_zone(index + 1, false, context, true)}
            } else {
                div { class: "w-0 shrink-0" }
            }
        }
    }
}

fn render_card_item(card: CardDisplayData, context: BoardRenderContext) -> Element {
    let card_id = card.id;
    let mut active_modal = context.active_modal;
    let mut card_drop_index = context.drag.card_drop_index;
    let mut dragged_item_kind = context.dragged_item_kind;
    let mut is_dragging = context.is_dragging;

    rsx! {
        div {
            key: "{card_id}",
            class: "flex min-w-0 flex-col gap-3",
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
                        format!("card:{card_id}:parent:{}", context.board_id),
                        DraggedItemKind::Card,
                        dragged_item_kind,
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
                    card_drop_index.set(None);
                    is_dragging.set(IsDragging(false));
                    dragged_item_kind.set(DraggedItemKind::None);
                },
                on_rename: move |_| active_modal.set(Some(ModalType::EditCard { id: card_id })),
                due_date: card.due_date,
                is_overdue: card.is_overdue,
                preview_items: card.preview_items,
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
        }
    }
}

fn render_card_drop_zone(
    index: usize,
    emphasized: bool,
    context: BoardRenderContext,
    side_oriented: bool,
) -> Element {
    let board_id = context.board_id;
    let registry = context.registry;
    let warning_message = context.warning_message;
    let mut card_drop_index = context.drag.card_drop_index;
    let dragged_item_kind = context.dragged_item_kind;
    let is_dragging = context.is_dragging;
    let is_active = card_drop_index() == Some(index);
    let class_name = drop_zone_classes(
        if emphasized {
            DropZoneKind::Board
        } else {
            DropZoneKind::Card
        },
        is_active,
        dragged_item_kind(),
    );

    rsx! {
        div {
            class: if emphasized {
                "{class_name} min-h-[3.25rem] rounded-2xl"
            } else if side_oriented {
                "{class_name} h-full min-h-[12rem] w-5 shrink-0 self-stretch rounded-full sm:w-6"
            } else {
                "{class_name} h-4 rounded-full"
            },
            ondragover: move |event| {
                prime_drop_target(&event);
                card_drop_index.set(Some(index));
            },
            ondragleave: move |_| {
                if card_drop_index() == Some(index) {
                    card_drop_index.set(None);
                }
            },
            ondrop: move |event| {
                event.prevent_default();

                let Some(card_id) = crate::interface::actions::dragged_card_id(&event, "board-route") else {
                    return;
                };

                card_drop_index.set(None);
                let _ = execute_command_with_feedback(
                    Command::DropChildAtPosition {
                        parent_id: board_id,
                        card_id,
                        target_index: index,
                    },
                    registry,
                    warning_message,
                    "board-route",
                    format!("drop card {card_id} at index {index} on board {board_id}"),
                );
            },
            if emphasized && is_dragging().0 {
                div { class: "flex h-full items-center justify-center px-4 py-3",
                    span { class: "app-kicker", "Drop Here" }
                }
            }
        }
    }
}
