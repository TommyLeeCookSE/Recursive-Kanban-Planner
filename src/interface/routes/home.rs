use crate::application::{build_board_view, build_card_preview_view};
use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::app::{DraggedItemKind, IsDragging};
use crate::interface::components::board_view::{
    BoardDragSignals, BoardRenderContext, render_board_screen,
};
use crate::interface::components::modal::ModalType;
use crate::interface::components::visuals::{CardDisplayData, build_card_display};
use dioxus::prelude::*;
use tracing::{Level, error};

#[derive(Clone, PartialEq, Eq)]
struct WorkspaceScreenData {
    board_id: CardId,
    board_title: String,
    board_due_date: String,
    child_cards: Vec<CardDisplayData>,
}

#[component]
pub fn Home() -> Element {
    let registry = use_context::<Signal<CardRegistry>>();
    let warning_message = use_context::<Signal<Option<String>>>();
    let active_modal = use_context::<Signal<Option<ModalType>>>();
    let is_dragging = use_context::<Signal<IsDragging>>();
    let dragged_item_kind = use_context::<Signal<DraggedItemKind>>();
    let card_drop_index = use_signal(|| None::<usize>);

    let reg_guard = registry.read();
    let Some(screen_data) = build_workspace_screen_data(&reg_guard) else {
        return render_workspace_load_error();
    };

    let render_context = BoardRenderContext {
        board_id: screen_data.board_id,
        registry,
        active_modal,
        warning_message,
        drag: BoardDragSignals { card_drop_index },
        dragged_item_kind,
        is_dragging,
    };

    render_board_screen(
        screen_data.board_title.clone(),
        None,
        screen_data.board_title,
        screen_data.board_due_date,
        screen_data.child_cards,
        render_context,
    )
}

fn build_workspace_screen_data(registry: &CardRegistry) -> Option<WorkspaceScreenData> {
    let workspace_id = match registry.workspace_card_id() {
        Ok(id) => id,
        Err(error_value) => {
            error!(error = %error_value, "Workspace route failed to load workspace card");
            record_diagnostic(
                Level::ERROR,
                "workspace-route",
                format!("Workspace load failed: {error_value}"),
            );
            return None;
        }
    };

    let view = match build_board_view(workspace_id, registry) {
        Ok(view) => view,
        Err(error_value) => {
            error!(
                %workspace_id,
                error = %error_value,
                "Workspace route failed to build board view"
            );
            record_diagnostic(
                Level::ERROR,
                "workspace-route",
                format!("Workspace board view failed for {workspace_id}: {error_value}"),
            );
            return None;
        }
    };

    let board_display = build_card_display(view.card, None);
    let board_due_date = board_display
        .due_date
        .as_deref()
        .unwrap_or("None")
        .to_string();
    let child_cards = match view
        .children
        .iter()
        .map(|card| {
            let preview_view = build_card_preview_view(card.id(), registry)?;
            Ok(build_card_display(card, Some(&preview_view)))
        })
        .collect::<Result<Vec<_>, DomainError>>()
    {
        Ok(cards) => cards,
        Err(error_value) => {
            error!(
                %workspace_id,
                error = %error_value,
                "Workspace route failed while building card previews"
            );
            record_diagnostic(
                Level::ERROR,
                "workspace-route",
                format!("Workspace preview load failed for {workspace_id}: {error_value}"),
            );
            return None;
        }
    };

    Some(WorkspaceScreenData {
        board_id: workspace_id,
        board_title: view.card.title().to_string(),
        board_due_date,
        child_cards,
    })
}

fn render_workspace_load_error() -> Element {
    rsx! {
        div { class: "mx-auto max-w-3xl px-6 py-20 text-center lg:px-12",
            div { class: "app-empty-state rounded-[2rem] px-8 py-16",
                h2 { class: "mb-4 text-2xl font-bold text-red-500", "Workspace could not be loaded" }
                p { class: "app-text-muted", "The workspace data is unavailable or inconsistent. Check the logs for the full error." }
            }
        }
    }
}
