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
                        onclick: move |_| active_modal.set(Some(ModalType::CreateCard { parent_id: None, bucket_id: None })),
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
                        if is_dark() { "Light" } else { "Dark" }
                    }
                }
            }
            main { class: "flex-grow overflow-auto",
                Outlet::<Route> {}
            }
        }
    }
}

#[component]
pub fn TopBar(
    title: String,
    back_route: Route,
    back_label: String,
    primary_label: String,
    on_primary: EventHandler<()>,
    secondary_label: String,
    on_secondary: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "px-6 py-5 border-b border-gray-200 dark:border-gray-800 bg-white/95 dark:bg-gray-950/90 backdrop-blur-sm",
            div { class: "max-w-7xl mx-auto flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between",
                div { class: "flex items-start gap-4",
                    button {
                        class: "mt-1 inline-flex items-center gap-2 px-3 py-2 rounded-full border border-gray-300 dark:border-gray-700 text-sm font-medium text-gray-600 dark:text-gray-300 hover:border-sunfire hover:text-sunfire transition-colors",
                        onclick: move |_| {
                            navigator().push(back_route.clone());
                        },
                        "<",
                        span { "{back_label}" }
                    }
                    div { class: "min-w-0",
                        p { class: "text-xs font-semibold uppercase tracking-[0.3em] text-gray-400 dark:text-gray-500",
                            "Current Board"
                        }
                        h1 { class: "text-3xl font-black tracking-tight text-gray-900 dark:text-white",
                            "{title}"
                        }
                    }
                }

                div { class: "flex flex-wrap items-center gap-3",
                    button {
                        class: "inline-flex items-center gap-2 px-4 py-2 rounded-full border border-sunfire/40 bg-sunfire/10 text-sunfire font-semibold hover:bg-sunfire/20 transition-colors",
                        onclick: move |_| on_primary.call(()),
                        "+",
                        span { "{primary_label}" }
                    }
                    button {
                        class: "inline-flex items-center gap-2 px-4 py-2 rounded-full border border-gray-300 dark:border-gray-700 text-gray-700 dark:text-gray-200 hover:border-sunfire hover:text-sunfire transition-colors",
                        onclick: move |_| on_secondary.call(()),
                        "Rename",
                        span { "{secondary_label}" }
                    }
                }
            }
        }
    }
}
