use crate::application::{
    BoardView, CardRuleEvent, Command, PopupNotification, build_board_view, execute,
};
use crate::domain::error::DomainError;
use crate::domain::id::{BucketId, CardId};
use crate::domain::registry::{CardRegistry, DeleteStrategy};
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::Route;
use crate::interface::actions::{
    dispatch_card_rule_event, execute_command_with_feedback, prime_drag_session, prime_drop_target,
    report_result,
};
use crate::interface::app::IsDragging;
use crate::interface::components::card_item::CardItem;
use crate::interface::components::layout::TopBar;
use crate::interface::components::modal::ModalType;
use crate::interface::components::visuals::{
    DropZoneKind, build_card_display, drop_zone_classes, render_label_chip,
};
use dioxus::prelude::*;
use std::collections::HashMap;
use tracing::{Level, error, info};

struct BoardScreenState<'a> {
    view: BoardView<'a>,
    back_route: Route,
    back_label: String,
}

#[derive(Clone, Copy)]
struct BoardDragSignals {
    bucket_drop_index: Signal<Option<usize>>,
    card_drop_target: Signal<Option<(BucketId, usize)>>,
}

#[derive(Clone)]
struct BoardRenderContext {
    board_id: CardId,
    registry: Signal<CardRegistry>,
    active_modal: Signal<Option<ModalType>>,
    warning_message: Signal<Option<String>>,
    popup_queue: Signal<Vec<PopupNotification>>,
    drag: BoardDragSignals,
    is_dragging: Signal<IsDragging>,
}

#[derive(Clone)]
struct ColumnRenderModel {
    bucket_id: BucketId,
    bucket_name: String,
    cards: Vec<CardRenderModel>,
}

#[derive(Clone)]
struct CardRenderModel {
    id: CardId,
    title: String,
    nested_item_count: usize,
    due_date: Option<String>,
    is_overdue: bool,
    labels: Vec<(String, crate::domain::label::LabelColor)>,
}

/// Renders a single board and its buckets for the selected card.
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     Board { card_id: board_id }
/// }
/// ```
#[component]
pub fn Board(card_id: CardId) -> Element {
    let registry = use_context::<Signal<CardRegistry>>();
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();
    let warning_message = use_context::<Signal<Option<String>>>();
    let popup_queue = use_context::<Signal<Vec<PopupNotification>>>();
    let is_dragging = use_context::<Signal<IsDragging>>();
    let bucket_drop_index = use_signal(|| None::<usize>);
    let card_drop_target = use_signal(|| None::<(BucketId, usize)>);

    let reg_guard = registry.read();
    let board_state = match load_board_state(card_id, &reg_guard) {
        Ok(state) => state,
        Err(error_value) => {
            error!(%card_id, error = %error_value, "Board route failed to load board state");
            record_diagnostic(
                Level::ERROR,
                "board-route",
                format!("Board load failed for {card_id}: {error_value}"),
            );
            return render_board_load_error();
        }
    };

    let board_id = card_id;
    let board_title = board_state.view.card.title().to_string();
    let label_definitions = reg_guard.label_definitions().to_vec();
    let board_display = build_card_display(board_state.view.card, &label_definitions);
    let board_due_date = board_display
        .due_date
        .as_deref()
        .unwrap_or("None")
        .to_string();
    let board_labels = board_display.labels.clone();
    let column_models = board_state
        .view
        .columns
        .iter()
        .map(|column| ColumnRenderModel {
            bucket_id: column.bucket.id(),
            bucket_name: column.bucket.name().to_string(),
            cards: column
                .cards
                .iter()
                .map(|card| {
                    let display = build_card_display(card, &label_definitions);
                    CardRenderModel {
                        id: display.id,
                        title: display.title,
                        nested_item_count: display.nested_item_count,
                        due_date: display.due_date,
                        is_overdue: display.is_overdue,
                        labels: display.labels,
                    }
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let render_context = BoardRenderContext {
        board_id,
        registry,
        active_modal,
        warning_message,
        popup_queue,
        drag: BoardDragSignals {
            bucket_drop_index,
            card_drop_target,
        },
        is_dragging,
    };

    rsx! {
        div { class: "flex h-full flex-col",
            TopBar {
                title: board_title.clone(),
                back_route: board_state.back_route.clone(),
                back_label: board_state.back_label.clone(),
                button {
                    class: "app-button-secondary h-14 px-8 text-sunfire",
                    onclick: move |_| active_modal.set(Some(ModalType::CreateBucket { card_id: board_id })),
                    span { class: "text-xl", "+" }
                    "Create Bucket"
                }
                button {
                    class: "app-button-secondary h-14 px-8",
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
                    "Notes"
                }
                button {
                    class: "app-button-secondary h-14 px-8",
                    onclick: move |_| active_modal.set(Some(ModalType::CardLabels { card_id: board_id })),
                    "Labels"
                }
                button {
                    class: "app-button-secondary h-14 px-8",
                    onclick: move |_| active_modal.set(Some(ModalType::EditCard { id: board_id })),
                    "Settings"
                }
            }

            div { class: "app-panel flex items-center justify-between border-b px-6 py-5 lg:px-12",
                div { class: "flex items-center gap-4",
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

fn load_board_state<'a>(
    card_id: CardId,
    registry: &'a CardRegistry,
) -> Result<BoardScreenState<'a>, DomainError> {
    let view = build_board_view(card_id, registry)?;

    let (back_route, back_label) = match view.card.parent_id() {
        Some(parent_id) => {
            let parent = registry.get_card(parent_id)?;
            (
                Route::Board { card_id: parent_id },
                parent.title().to_string(),
            )
        }
        None => (Route::Home {}, "Workspace".to_string()),
    };

    Ok(BoardScreenState {
        view,
        back_route,
        back_label,
    })
}

fn render_column(column: ColumnRenderModel, context: BoardRenderContext) -> Element {
    let bucket_id = column.bucket_id;
    let bucket_name = column.bucket_name;
    let mut active_modal = context.active_modal;
    let warning_message = context.warning_message;
    let mut bucket_drop_index = context.drag.bucket_drop_index;
    let mut is_dragging = context.is_dragging;
    let can_delete_bucket = bucket_name != crate::domain::card::UNASSIGNED_BUCKET_NAME;
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
                class: "mb-6 flex cursor-grab items-center justify-between rounded-2xl px-3 py-2 active:cursor-grabbing",
                div {
                    h2 { class: "app-kicker transition-colors group-hover:text-sunfire", "{bucket_name}" }
                }
                div { class: "flex items-center gap-2",
                    if can_delete_bucket {
                        button {
                            class: "app-button-secondary inline-flex h-8 min-w-[3.5rem] items-center justify-center rounded-full px-3 text-[11px] font-black uppercase tracking-widest text-red-400 hover:text-red-500",
                            title: "Delete this bucket",
                            onclick: move |_| {
                                if execute_command_with_feedback(
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
                                )
                                .is_err()
                                {
                                }
                            },
                            "Delete"
                        }
                    }
                    button {
                        class: "app-button-secondary inline-flex h-8 w-8 items-center justify-center rounded-full border-2 border-dashed p-0 hover:rotate-90",
                        onclick: move |_| active_modal.set(Some(ModalType::CreateCard {
                            parent_id: Some(context.board_id),
                            bucket_id: Some(bucket_id),
                        })),
                        "+"
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
    card: CardRenderModel,
    bucket_id: BucketId,
    index: usize,
    context: BoardRenderContext,
) -> Element {
    let card_id = card.id;
    let card_title = card.title;
    let nested_item_count = card.nested_item_count;
    let mut active_modal = context.active_modal;
    let mut card_drop_target = context.drag.card_drop_target;
    let mut is_dragging = context.is_dragging;

    rsx! {
        div {
            key: "{card_id}",
            class: "flex flex-col gap-3",
            draggable: true,
            ondragstart: move |event| {
                prime_drag_session(
                    &event,
                    "board-route",
                    format!("card:{card_id}:board:{}", context.board_id),
                    is_dragging,
                );
            },
            ondragend: move |_| {
                card_drop_target.set(None);
                is_dragging.set(IsDragging(false));
            },
            CardItem {
                title: card_title.clone(),
                subtitle: format!("{nested_item_count} nested items"),
                on_open: move |_| {
                    navigator().push(Route::Board { card_id });
                },
                on_rename: move |_| active_modal.set(Some(ModalType::EditCard { id: card_id })),
                due_date: card.due_date.clone(),
                is_overdue: card.is_overdue,
                labels: card.labels,
                on_delete: move |_| {
                    delete_card_with_feedback(card_id, context.registry, context.warning_message);
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

                let reordered = reorder_ids(&current_order, bucket_id, index);
                if reordered != current_order {
                    info!(bucket_id = %bucket_id, board_id = %board_id, drop_index = index, "Attempting bucket reorder");
                    record_diagnostic(
                        Level::INFO,
                        "board-route",
                        format!("Attempting bucket reorder for {bucket_id} on board {board_id} at index {index}"),
                    );
                    if execute_command_with_feedback(
                        Command::ReorderBuckets {
                            card_id: board_id,
                            ordered_ids: reordered,
                        },
                        registry,
                        warning_message,
                        "board-route",
                        format!(
                            "reorder buckets on board {board_id} with dragged bucket {bucket_id}"
                        ),
                    )
                    .is_err()
                    {
                    }
                }
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

                let result = {
                    let mut reg = registry.write();
                    apply_card_drop(&mut reg, board_id, card_id, bucket_id, index)
                };
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

fn reorder_ids<T>(ordered_ids: &[T], dragged_id: T, target_index: usize) -> Vec<T>
where
    T: Copy + Eq,
{
    let mut reordered: Vec<T> = ordered_ids
        .iter()
        .copied()
        .filter(|id| *id != dragged_id)
        .collect();
    let insertion_index = target_index.min(reordered.len());
    reordered.insert(insertion_index, dragged_id);
    reordered
}

fn apply_card_drop(
    registry: &mut CardRegistry,
    board_id: CardId,
    card_id: CardId,
    target_bucket_id: BucketId,
    target_index: usize,
) -> Result<(), DomainError> {
    let board = registry.get_card(board_id)?;
    let bucket_order: Vec<BucketId> = board.buckets().iter().map(|bucket| bucket.id()).collect();
    let child_order = board.children_ids().to_vec();

    let mut cards_by_bucket: HashMap<BucketId, Vec<CardId>> = bucket_order
        .iter()
        .copied()
        .map(|bucket_id| (bucket_id, Vec::new()))
        .collect();
    let mut current_bucket_id = None;

    for child_id in child_order {
        let child = registry.get_card(child_id)?;
        let bucket_id = child.bucket_id().ok_or_else(|| {
            DomainError::InvalidOperation(format!(
                "Child card {child_id} is missing its bucket assignment"
            ))
        })?;

        if child_id == card_id {
            current_bucket_id = Some(bucket_id);
            continue;
        }

        cards_by_bucket.entry(bucket_id).or_default().push(child_id);
    }

    let current_bucket_id = current_bucket_id.ok_or(DomainError::CardNotFound(card_id))?;
    let target_cards = cards_by_bucket
        .get_mut(&target_bucket_id)
        .ok_or(DomainError::BucketNotFound(target_bucket_id))?;
    let insertion_index = target_index.min(target_cards.len());
    target_cards.insert(insertion_index, card_id);

    if current_bucket_id != target_bucket_id {
        execute(
            Command::MoveCardToBucket {
                card_id,
                bucket_id: target_bucket_id,
            },
            registry,
        )?;
    }

    let mut reordered_children = Vec::new();
    for bucket_id in bucket_order {
        if let Some(cards) = cards_by_bucket.get(&bucket_id) {
            reordered_children.extend(cards.iter().copied());
        }
    }

    execute(
        Command::ReorderChildren {
            card_id: board_id,
            ordered_ids: reordered_children,
        },
        registry,
    )
}

fn delete_card_with_feedback(
    id: CardId,
    registry: Signal<CardRegistry>,
    warning_message: Signal<Option<String>>,
) {
    if execute_command_with_feedback(
        Command::DeleteCard {
            id,
            strategy: DeleteStrategy::CascadeDelete,
        },
        registry,
        warning_message,
        "board-route",
        format!("delete board card {id}"),
    )
    .is_err()
    {}
}

fn render_board_load_error() -> Element {
    rsx! {
        div { class: "mx-auto max-w-3xl px-6 py-20 text-center lg:px-12",
            div { class: "app-empty-state rounded-[2rem] px-8 py-16",
                h2 { class: "mb-4 text-2xl font-bold text-red-500", "Board could not be loaded" }
                p { class: "app-text-muted", "The board data is unavailable or inconsistent. Check the logs for the full error." }
                button {
                    class: "app-button-primary mt-8 px-6 py-3",
                    onclick: |_| {
                        navigator().push(Route::Home {});
                    },
                    "Back to Workspace"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_board_state_preserves_domain_errors() {
        let registry = CardRegistry::new();
        let missing_id = CardId::default();

        let result = load_board_state(missing_id, &registry);

        assert!(matches!(result, Err(DomainError::CardNotFound(id)) if id == missing_id));
    }
}
