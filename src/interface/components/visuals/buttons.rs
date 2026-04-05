use dioxus::prelude::*;

use super::{toolbar_action_icon_classes, toolbar_button_classes};

#[component]
pub fn BarButton(
    label: String,
    #[props(default)] title: Option<String>,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] draggable: bool,
    #[props(default = true)] show_label: bool,
    #[props(default)] class_name: Option<String>,
    #[props(default)] icon: Option<Element>,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let mut classes = toolbar_button_classes().to_string();
    if let Some(class_name) = class_name.filter(|value| !value.is_empty()) {
        classes.push(' ');
        classes.push_str(&class_name);
    }

    let title = title.unwrap_or_else(|| label.clone());
    let aria_label = aria_label.unwrap_or_else(|| label.clone());

    rsx! {
        button {
            class: "{classes}",
            disabled,
            draggable,
            title: "{title}",
            "aria-label": "{aria_label}",
            type: "button",
            onclick: move |event| on_click.call(event),
            if let Some(icon) = icon {
                span { class: toolbar_action_icon_classes(), {icon} }
            }
            if show_label {
                span { class: "app-bar-button-label", "{label}" }
            }
        }
    }
}
