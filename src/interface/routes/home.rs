use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use crate::interface::app::{DraggedItemKind, IsDragging};
use crate::interface::components::board_view::{
    BoardDragSignals, BoardRenderContext, render_board_screen,
};
use crate::interface::components::modal::ModalType;
use crate::interface::routes::board_screen::load_workspace_screen_data;
use dioxus::prelude::*;
use tracing::{Level, error};

#[component]
pub fn Home() -> Element {
    let registry = use_context::<Signal<CardRegistry>>();
    let warning_message = use_context::<Signal<Option<String>>>();
    let active_modal = use_context::<Signal<Option<ModalType>>>();
    let is_dragging = use_context::<Signal<IsDragging>>();
    let dragged_item_kind = use_context::<Signal<DraggedItemKind>>();
    let card_drop_index = use_signal(|| None::<usize>);

    let reg_guard = registry.read();
    let screen_data = match load_workspace_screen_data(&reg_guard) {
        Ok(screen_data) => screen_data,
        Err(error_value) => {
            error!(error = %error_value, "Workspace route failed to load workspace card");
            record_diagnostic(
                Level::ERROR,
                "workspace-route",
                format!("Workspace load failed: {error_value}"),
            );
            return render_workspace_load_error();
        }
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
        screen_data.back_label,
        screen_data.board_due_date,
        screen_data.child_cards,
        render_context,
    )
}

fn render_workspace_load_error() -> Element {
    rsx! {
        div { class: "app-page-shell",
            div { class: "app-empty-state app-empty-state-panel",
                h2 { class: "app-error-title", "Workspace could not be loaded" }
                p { class: "app-error-message", "The workspace data is unavailable or inconsistent. Check the logs for the full error." }
            }
        }
    }
}
