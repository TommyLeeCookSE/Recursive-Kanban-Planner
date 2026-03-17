#![allow(non_snake_case)]
use dioxus::prelude::*;

pub fn App() -> Element {
    rsx! {
        div {
            style: "padding: 2rem; font-family: sans-serif;",
            h1 { "Recursive Kanban Planner" }
            p { "Dioxus shell initialized. Domain logic is being verified." }
        }
    }
}
