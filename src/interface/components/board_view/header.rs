use crate::interface::Route;
use crate::interface::components::board_view::models::BoardRenderContext;
use crate::interface::components::layout::TopBar;
use crate::interface::components::modal::ModalType;
use crate::interface::components::visuals::{
    render_note_icon, render_plus_icon, render_settings_icon, toolbar_action_icon_classes,
    toolbar_button_classes, toolbar_button_label_classes,
};
use dioxus::prelude::*;

pub(super) fn render_board_header(
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
                class: "{toolbar_button_classes()} app-toolbar-button-accent",
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

        div { class: "app-board-status",
            div { class: "app-board-status-inner",
                p { class: "app-kicker", "Status: Active | Due: {board_due_date}" }
            }
        }
    }
}
