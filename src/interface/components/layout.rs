use crate::interface::Route;
use crate::interface::components::modal::ModalType;
use dioxus::prelude::*;

/// The main layout wrapper with a sticky top navigation bar.
#[component]
pub fn NavbarLayout() -> Element {
    let mut is_dark = use_context::<Signal<bool>>();
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();

    rsx! {
        div { class: "min-h-screen flex flex-col bg-gray-50 dark:bg-gray-950 font-sans antialiased selection:bg-sunfire/30 selection:text-sunfire-dark",
            // Sticky Main Navigation
            nav { class: "sticky top-0 z-40 h-20 flex items-center justify-between px-12 bg-white/90 dark:bg-gray-900/95 border-b border-gray-100 dark:border-gray-800/80 backdrop-blur-xl transition-all",
                div { class: "flex items-center gap-8",
                    Link { 
                        to: Route::Home {}, 
                        class: "group flex items-center gap-3",
                        div { class: "h-10 w-10 bg-sunfire rounded-xl shadow-lg shadow-sunfire/30 flex items-center justify-center transform group-hover:rotate-12 transition-transform",
                            span { class: "text-white text-xl font-black", "K" }
                        }
                        span { class: "font-black text-2xl tracking-tighter text-gray-900 dark:text-white group-hover:text-sunfire transition-colors", 
                            "Kanban" 
                        }
                    }
                    button {
                        class: "ml-4 px-6 py-2.5 bg-gray-900 dark:bg-white text-white dark:text-gray-900 font-bold rounded-2xl shadow-xl hover:shadow-sunfire/10 hover:bg-sunfire dark:hover:bg-sunfire transition-all transform hover:-translate-y-0.5 active:translate-y-0 flex items-center gap-2",
                        onclick: move |_| active_modal.set(Some(ModalType::CreateCard { parent_id: None, bucket_id: None })),
                        span { class: "text-lg", "+" }
                        "New Board"
                    }
                }

                div { class: "flex items-center gap-4",
                    // Utility Buttons
                    div { class: "hidden md:flex items-center gap-2 pr-4 border-r border-gray-100 dark:border-gray-800",
                        button { class: "px-4 py-2 text-xs font-black uppercase tracking-widest text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors",
                            "Export"
                        }
                        button { class: "px-4 py-2 text-xs font-black uppercase tracking-widest text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors",
                            "Import"
                        }
                    }
                    
                    // Dark Mode Toggle
                    button {
                        class: "p-3 rounded-2xl bg-gray-100 dark:bg-gray-800 text-gray-500 dark:text-gray-400 hover:text-sunfire hover:bg-sunfire/10 transition-all",
                        onclick: move |_| is_dark.set(!is_dark()),
                        title: "Toggle Light/Dark Mode",
                        if is_dark() { 
                            span { "☼" } 
                        } else { 
                            span { "☾" } 
                        }
                    }
                }
            }

            // Main Content Area
            main { class: "flex-grow overflow-auto",
                Outlet::<Route> {}
            }
        }
    }
}

/// A secondary navigation bar used within specific boards for context and local actions.
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
        div { class: "px-12 py-8 bg-white dark:bg-gray-900 border-b border-gray-100 dark:border-gray-800/60 shadow-sm",
            div { class: "max-w-7xl mx-auto flex flex-col gap-8 lg:flex-row lg:items-end lg:justify-between",
                
                // Context & Title
                div { class: "flex items-start gap-6",
                    button {
                        class: "mt-1.5 p-3 rounded-2xl border border-gray-100 dark:border-gray-800 text-gray-400 hover:text-sunfire hover:border-sunfire/50 transition-all group",
                        onclick: move |_| {
                            navigator().push(back_route.clone());
                        },
                        span { class: "transform group-hover:-translate-x-1 block transition-transform", "←" }
                    }
                    div { class: "min-w-0 pr-8 border-r border-gray-100 dark:border-gray-800",
                        span { class: "text-[10px] font-black uppercase tracking-[0.5em] text-gray-300 dark:text-gray-700 block mb-1",
                            "Board Context / {back_label}"
                        }
                        h1 { class: "text-5xl font-black tracking-tighter text-gray-900 dark:text-white truncate max-w-2xl",
                            "{title}"
                        }
                    }
                }

                // Actions
                div { class: "flex flex-wrap items-center gap-4",
                    button {
                        class: "h-14 px-8 rounded-2xl border-2 border-sunfire/40 bg-sunfire/5 text-sunfire font-bold hover:bg-sunfire/10 transition-all flex items-center gap-3 active:scale-95 shadow-lg shadow-sunfire/5",
                        onclick: move |_| on_primary.call(()),
                        span { class: "text-xl", "+" }
                        "{primary_label}"
                    }
                    button {
                        class: "h-14 px-8 rounded-2xl border-2 border-gray-100 dark:border-gray-800 text-gray-500 dark:text-gray-400 font-bold hover:border-gray-300 dark:hover:border-gray-600 hover:text-gray-900 dark:hover:text-white transition-all active:scale-95",
                        onclick: move |_| on_secondary.call(()),
                        "{secondary_label}"
                    }
                }
            }
        }
    }
}
