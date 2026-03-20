use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::interface::app::{DraggedItemKind, IsDragging};
use crate::interface::components::modal::ModalType;
use dioxus::prelude::*;

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
