use crate::interface::Route;
use crate::interface::components::modal::ModalType;
use dioxus::prelude::*;

#[component]
pub fn NavbarLayout() -> Element {
    let mut is_dark = use_context::<Signal<bool>>();
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();

    rsx! {
        div { class: "min-h-screen flex flex-col",
            nav { class: "h-16 flex items-center justify-between px-8 bg-white dark:bg-gray-800 border-b dark:border-gray-700 shadow-sm transition-colors",
                div { class: "flex items-center gap-4",
                    Link { to: Route::Home {}, class: "text-sunfire hover:text-sunfire-dark transition-colors",
                        span { class: "font-bold text-xl", "Kanban Planner" }
                    }
                    button {
                        class: "ml-6 px-4 py-1.5 bg-sunfire/10 hover:bg-sunfire/20 text-sunfire font-bold rounded-lg border border-sunfire/30 transition-all",
                        onclick: move |_| active_modal.set(Some(crate::interface::components::modal::ModalType::CreateCard { parent_id: None, bucket_id: None })),
                        "+ New Board"
                    }
                }

                div { class: "flex items-center gap-2",
                    button { class: "px-3 py-1.5 text-sm font-medium bg-gray-200 dark:bg-gray-700 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors",
                        "Export"
                    }
                    button { class: "px-3 py-1.5 text-sm font-medium bg-gray-200 dark:bg-gray-700 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors",
                        "Import"
                    }
                    button {
                        class: "p-2 rounded-full hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors",
                        onclick: move |_| is_dark.set(!is_dark()),
                        if is_dark() { "☼" } else { "☾" }
                    }
                }
            }
            main { class: "flex-grow overflow-auto",
                Outlet::<Route> {}
            }
        }
    }
}
