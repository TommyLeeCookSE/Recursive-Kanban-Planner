//! SVG icon renderers for the user interface.
//!
//! This module provides a set of consistent, themed SVG icons used
//! throughout the application.
//!
//! For more on how Rust's macros and components work compared to Python,
//! see `docs/rust-for-python-devs.md`.

use dioxus::prelude::*;

/// A base component for rendering consistent SVG icons.
#[component]
fn Icon(
    view_box: &'static str,
    stroke_width: &'static str,
    children: Element,
    class: Option<&'static str>,
) -> Element {
    let class = class.unwrap_or("app-icon-md");
    rsx! {
        svg {
            class: "{class}",
            view_box: "{view_box}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "{stroke_width}",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            {children}
        }
    }
}

macro_rules! define_icon {
    (
        $(#[$meta:meta])*
        $name:ident, $view_box:expr, $stroke_width:expr, class: $class:expr, $( $body:tt )*
    ) => {
        $(#[$meta])*
        pub fn $name() -> Element {
            rsx! {
                Icon { view_box: $view_box, stroke_width: $stroke_width, class: $class,
                    $( $body )*
                }
            }
        }
    };
    (
        $(#[$meta:meta])*
        $name:ident, $view_box:expr, $stroke_width:expr, $( $body:tt )*
    ) => {
        $(#[$meta])*
        pub fn $name() -> Element {
            rsx! {
                Icon { view_box: $view_box, stroke_width: $stroke_width,
                    $( $body )*
                }
            }
        }
    };
}

define_icon! {
    /// Renders a "+" (plus) icon.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// rsx! { render_plus_icon() }
    /// ```
    render_plus_icon, "0 0 20 20", "1.9",
    path { d: "M10 4.5v11" }
    path { d: "M4.5 10h11" }
}

define_icon! {
    /// Renders a document/note icon.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// rsx! { render_note_icon() }
    /// ```
    render_note_icon, "0 0 24 24", "1.8",
    path { d: "M7 3.75h7.75L19.5 8.5v11.75a1.5 1.5 0 0 1-1.5 1.5H7a1.5 1.5 0 0 1-1.5-1.5V5.25A1.5 1.5 0 0 1 7 3.75Z" }
    path { d: "M14.5 3.75V8.5H19.5" }
    path { d: "M8.5 12h7" }
    path { d: "M8.5 15.5h7" }
}

define_icon! {
    /// Renders a book icon.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// rsx! { render_book_icon() }
    /// ```
    render_book_icon, "0 0 24 24", "1.8",
    path { d: "M6.5 4.75h9.75a2 2 0 0 1 2 2v12.5a1.5 1.5 0 0 0-1.5-1.5H6.5a2 2 0 0 1-2-2v-9a2 2 0 0 1 2-2Z" }
    path { d: "M6.5 4.75a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h10.25" }
    path { d: "M8.25 8h6.75" }
    path { d: "M8.25 11h6.75" }
}

define_icon! {
    /// Renders a label/tag icon.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// rsx! { render_label_icon() }
    /// ```
    render_label_icon, "0 0 24 24", "1.8",
    path { d: "M11 4.5H6.75A2.25 2.25 0 0 0 4.5 6.75V11L12.75 19.25a1.5 1.5 0 0 0 2.12 0l4.38-4.38a1.5 1.5 0 0 0 0-2.12L11 4.5Z" }
    circle { cx: "8.25", cy: "8.25", r: "1.1" }
}

define_icon! {
    /// Renders an import/download icon.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// rsx! { render_import_icon() }
    /// ```
    render_import_icon, "0 0 24 24", "1.8",
    path { d: "M12 4.75v10" }
    path { d: "m8.25 11.25 3.75 3.75 3.75-3.75" }
    path { d: "M6.5 16.5v1.75a2 2 0 0 0 2 2h7a2 2 0 0 0 2-2V16.5" }
}

define_icon! {
    /// Renders an export/upload icon.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// rsx! { render_export_icon() }
    /// ```
    render_export_icon, "0 0 24 24", "1.8",
    path { d: "M12 19.25v-10" }
    path { d: "m15.75 12.75-3.75-3.75-3.75 3.75" }
    path { d: "M6.5 7.5V5.75a2 2 0 0 1 2-2h7a2 2 0 0 1 2 2V7.5" }
}

define_icon! {
    /// Renders a trash/delete icon.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// rsx! { render_trash_icon() }
    /// ```
    render_trash_icon, "0 0 24 24", "1.8",
    path { d: "M4.75 7.5h14.5" }
    path { d: "M9 4.75h6" }
    path { d: "M8.25 7.5V5.75a1 1 0 0 1 1-1h5.5a1 1 0 0 1 1 1V7.5" }
    path { d: "M9.5 10.5v6.25" }
    path { d: "M14.5 10.5v6.25" }
    path { d: "M6.75 7.5l.75 10.25a2 2 0 0 0 2 1.85h4.95a2 2 0 0 0 2-1.85L17.2 7.5" }
}

define_icon! {
    /// Renders a sunrise icon.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// rsx! { render_sunrise_icon() }
    /// ```
    render_sunrise_icon, "0 0 24 24", "1.8",
    circle { cx: "12", cy: "12", r: "5" }
    path { d: "M12 1v2" }
    path { d: "M12 21v2" }
    path { d: "M4.22 4.22l1.42 1.42" }
    path { d: "M18.36 18.36l1.42 1.42" }
    path { d: "M1 12h2" }
    path { d: "M21 12h2" }
    path { d: "M4.22 19.78l1.42-1.42" }
    path { d: "M18.36 5.64l1.42-1.42" }
}

define_icon! {
    /// Renders a moon/evening icon.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// rsx! { render_evening_icon() }
    /// ```
    render_evening_icon, "0 0 24 24", "1.8",
    path { d: "M12 3a9 9 0 1 0 9 9 9.75 9.75 0 0 1-9-9Z" }
}

define_icon! {
    /// Renders a settings icon.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// rsx! { render_settings_icon() }
    /// ```
    render_settings_icon, "0 0 24 24", "1.8",
    path { d: "M12 4.75 13.62 6.1l2.08-.33.9 1.9 1.97.74-.2 2.1 1.46 1.52-1.46 1.52.2 2.1-1.97.74-.9 1.9-2.08-.33L12 19.25l-1.62-1.35-2.08.33-.9-1.9-1.97-.74.2-2.1-1.46-1.52 1.46-1.52-.2-2.1 1.97-.74.9-1.9 2.08.33L12 4.75Z" }
    circle { cx: "12", cy: "12", r: "2.6" }
}

define_icon! {
    /// Renders a "back" arrow icon.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// rsx! { render_back_icon() }
    /// ```
    render_back_icon, "0 0 20 20", "1.9", class: "app-icon-back",
    path { d: "M11.5 4.5 6 10l5.5 5.5" }
    path { d: "M6.5 10h8" }
}

define_icon! {
    /// Renders an edit/pencil icon.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// rsx! { render_edit_icon() }
    /// ```
    render_edit_icon, "0 0 24 24", "1.8",
    path { d: "M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z" }
}

define_icon! {
    /// Renders a drag handle (grip vertical) icon.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// rsx! { render_drag_handle_icon() }
    /// ```
    render_drag_handle_icon, "0 0 24 24", "2",
    path { d: "M9 5h.01 M9 12h.01 M9 19h.01 M15 5h.01 M15 12h.01 M15 19h.01" }
}

define_icon! {
    /// Renders a search icon.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// rsx! { render_search_icon() }
    /// ```
    render_search_icon, "0 0 24 24", "1.8",
    circle { cx: "11", cy: "11", r: "8" }
    path { d: "m21 21-4.35-4.35" }
}
