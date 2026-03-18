use crate::application::{Command, build_board_view, execute_and_log};
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::interface::Route;
use crate::interface::components::card_item::{CardItem, MoveTarget};
use crate::interface::components::layout::TopBar;
use crate::interface::components::modal::ModalType;
use dioxus::prelude::*;

#[component]
pub fn Board(card_id: CardId) -> Element {
    let mut registry = use_context::<Signal<CardRegistry>>();
    let mut active_modal = use_context::<Signal<Option<ModalType>>>();

    // Hold the guard for the duration of the component's setup to satisfy the borrow checker
    let reg_guard = registry.read();
    
    let view = build_board_view(card_id, &reg_guard).ok();

    let Some(view) = view else {
        return rsx! { 
            div { class: "p-20 text-center",
                h2 { class: "text-2xl font-bold text-red-500 mb-4", "Board Not Found" }
                p { class: "text-gray-500", "The requested board could not be loaded." }
                button { 
                    class: "mt-8 px-6 py-2 bg-sunfire text-white rounded-lg",
                    onclick: |_| { navigator().push(Route::Home {}); },
                    "Back to Workspace"
                }
            } 
        };
    };

    let (back_route, back_label) = match view.card.parent_id() {
        Some(parent_id) => {
            let label = reg_guard.get_card(parent_id).map(|p| p.title().to_string()).unwrap_or_else(|_| "Workspace".into());
            (Route::Board { card_id: parent_id }, label)
        }
        None => (Route::Home {}, "Workspace".to_string()),
    };

    let move_targets: Vec<_> = view.card.buckets().iter().map(|b| MoveTarget { id: b.id(), name: b.name().to_string() }).collect();
    let board_id = card_id;
    let board_title = view.card.title().to_string();

    rsx! {
        div { class: "h-full flex flex-col bg-gray-50 dark:bg-gray-950 transition-colors",
            TopBar {
                title: board_title.clone(),
                back_route,
                back_label,
                primary_label: "Create Bucket".to_string(),
                on_primary: move |_| active_modal.set(Some(ModalType::CreateBucket { card_id: board_id })),
                secondary_label: "Settings".to_string(),
                on_secondary: move |_| active_modal.set(Some(ModalType::RenameCard { id: board_id, current_title: board_title.clone() })),
            }

            div { class: "px-12 py-5 border-b border-gray-100 dark:border-gray-800/60 bg-white/70 dark:bg-gray-900/50 backdrop-blur-md flex items-center justify-between",
                p { class: "text-xs font-black uppercase tracking-[0.3em] text-gray-400 dark:text-gray-600",
                    "Status: Active — {view.card.children_ids().len()} nested items"
                }
            }

            div { class: "flex-grow overflow-x-auto p-12",
                div { class: "flex gap-8 h-full items-start min-w-max",
                    {view.columns.iter().map(|column| {
                        let bucket_id = column.bucket.id();
                        let bucket_name = column.bucket.name().to_string();
                        rsx! {
                            div { 
                                key: "{bucket_id}",
                                class: "group/col flex-shrink-0 w-80 bg-gray-200/40 dark:bg-gray-800/30 p-5 rounded-3xl flex flex-col max-h-full border border-gray-100 dark:border-gray-800/40 hover:border-sunfire/30 hover:bg-gray-200/60 dark:hover:bg-gray-800/50 transition-all",
                                div { class: "flex items-center justify-between mb-6 px-3",
                                    h2 { class: "text-[11px] font-black uppercase tracking-[0.4em] text-gray-400 dark:text-gray-600 group-hover/col:text-sunfire/60 transition-colors", "{bucket_name}" }
                                    button {
                                        class: "inline-flex items-center justify-center h-8 w-8 rounded-full border-2 border-dashed border-gray-200 dark:border-gray-700 text-gray-400 hover:border-sunfire hover:text-sunfire transition-all hover:rotate-90",
                                        onclick: move |_| active_modal.set(Some(ModalType::CreateCard { parent_id: Some(board_id), bucket_id: Some(bucket_id) })),
                                        "+"
                                    }
                                }
                                div { class: "flex-grow overflow-y-auto space-y-4 pr-2",
                                    {column.cards.iter().map(|card| {
                                        let cid = card.id();
                                        let c_title = card.title().to_string();
                                        let c_nested = card.children_ids().len();
                                        let c_bucket = card.bucket_id();
                                        rsx! {
                                            CardItem {
                                                key: "{cid}",
                                                title: c_title.clone(),
                                                subtitle: format!("{c_nested} items nested"),
                                                current_bucket_id: c_bucket,
                                                move_targets: move_targets.clone(),
                                                on_open: move |_| { navigator().push(Route::Board { card_id: cid }); },
                                                on_rename: move |_| active_modal.set(Some(ModalType::RenameCard { id: cid, current_title: c_title.clone() })),
                                                on_move: move |next_bucket_id| {
                                                    let mut reg = registry.write();
                                                    let _ = execute_and_log(Command::MoveCardToBucket { card_id: cid, bucket_id: next_bucket_id }, &mut reg, "board-route");
                                                },
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    })}
                }
            }
        }
    }
}
