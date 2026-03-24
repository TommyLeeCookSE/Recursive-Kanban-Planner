use crate::application::Command;
use crate::domain::due_date::DueDate;
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::form_row;
use crate::interface::components::modal::Modal;
use crate::interface::components::shared_forms::{
    inline_error, user_message_for_command_error,
};
use dioxus::prelude::*;

#[component]
pub fn MinimalTest(
) -> Element {
    let mut title = use_signal(String::new);
    rsx! {
        div {
            form_row! {
                label: "Test",
                id: "test-id",
                input: rsx! {
                    input {
                        id: "test-id",
                        value: "{title}",
                        oninput: move |e| title.set(e.value()),
                    }
                }
            }
        }
    }
}
