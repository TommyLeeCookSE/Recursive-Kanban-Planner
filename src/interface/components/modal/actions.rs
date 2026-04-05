use dioxus::prelude::*;

/// Standardized modal footer action row.
///
/// This is intentionally a thin layout primitive: it only normalizes spacing and alignment,
/// while leaving button behavior and styling to the caller.
#[component]
pub fn ModalActions(#[props(default)] class_name: Option<String>, children: Element) -> Element {
    let mut classes = "app-form-actions".to_string();
    if let Some(extra) = class_name.filter(|value| !value.is_empty()) {
        classes.push(' ');
        classes.push_str(&extra);
    }

    rsx! {
        div { class: "{classes}",
            {children}
        }
    }
}
