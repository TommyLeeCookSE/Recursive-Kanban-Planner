use crate::domain::id::BucketId;
use dioxus::prelude::*;
use std::str::FromStr;

/// Defines a potential destination bucket for a card move action.
///
/// # Examples
///
/// ```rust
/// use kanban_planner::domain::id::BucketId;
/// use kanban_planner::interface::components::card_item::MoveTarget;
///
/// let target = MoveTarget {
///     id: BucketId::default(),
///     name: "Doing".to_string(),
/// };
/// assert_eq!(target.name, "Doing");
/// ```
#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct MoveTarget {
    pub id: BucketId,
    pub name: String,
}

/// A premium, reusable card component for both Workspace and Board views.
///
/// Features:
/// - Clickable area for navigation (opening the card).
/// - Optional "Rename" action.
/// - Optional "Move" dropdown (only visible if `move_targets` is provided).
///
/// # Examples
///
/// ```ignore
/// rsx! {
///     CardItem {
///         title: "Design API".to_string(),
///         subtitle: "2 nested items".to_string(),
///         current_bucket_id: None,
///         on_open: move |_| {},
///     }
/// }
/// ```
#[component]
pub fn CardItem(
    /// Main display text.
    title: String,
    /// Second line of text for counts or metadata.
    subtitle: String,
    /// Current bucket ID (used to pre-select in the Move dropdown).
    current_bucket_id: Option<BucketId>,
    /// List of available buckets to move this card into.
    #[props(default)]
    move_targets: Vec<MoveTarget>,
    /// Triggered when the main body of the card is clicked.
    on_open: EventHandler<()>,
    /// Optional rename event. If None, the rename button is hidden.
    #[props(default)]
    on_rename: Option<EventHandler<()>>,
    /// Optional move event. If None or if `move_targets` is empty, the move UI is hidden.
    #[props(default)]
    on_move: Option<EventHandler<BucketId>>,
) -> Element {
    let current_bucket_value = current_bucket_id
        .map(|id| id.to_string())
        .unwrap_or_default();

    let show_move_ui = on_move.is_some() && !move_targets.is_empty();

    rsx! {
        article {
            class: "app-card-surface group flex flex-col rounded-[1.75rem] transition-all hover:border-sunfire/50 hover:-translate-y-0.5",

            // Primary Action Area
            button {
                class: "flex-grow w-full rounded-t-[1.75rem] p-6 text-left outline-none transition-colors focus:ring-2 focus:ring-sunfire/30",
                onclick: move |_| on_open.call(()),
                h3 { class: "app-text-primary h-12 overflow-hidden text-lg font-semibold transition-colors group-hover:text-sunfire line-clamp-2",
                    "{title}"
                }
                p { class: "app-text-soft mt-3 text-xs font-medium uppercase tracking-widest",
                    "{subtitle}"
                }
            }

            // Secondary Actions (if provided)
            if on_rename.is_some() || show_move_ui {
                div { class: "flex flex-wrap items-center gap-4 rounded-b-[1.75rem] border-t px-5 py-4", style: "border-color: var(--app-border); background-color: color-mix(in srgb, var(--app-surface-soft) 74%, transparent);",

                    if let (Some(move_handler), true) = (on_move, show_move_ui) {
                        div { class: "flex items-center gap-2",
                            label { class: "app-kicker",
                                "Move"
                            }
                            select {
                                class: "app-input rounded-full px-3 py-1.5 text-xs font-semibold",
                                value: "{current_bucket_value}",
                                onchange: move |e| {
                                    if let Ok(bucket_id) = BucketId::from_str(&e.value()) {
                                        move_handler.call(bucket_id);
                                    }
                                },
                                for target in move_targets {
                                    option { value: "{target.id}", "{target.name}" }
                                }
                            }
                        }
                    }

                    if let Some(rename_handler) = on_rename {
                        button {
                            class: "app-button-secondary ml-auto rounded-full px-3 py-1.5 text-[11px] font-black uppercase tracking-widest",
                            onclick: move |_| rename_handler.call(()),
                            "Rename"
                        }
                    }
                }
            }
        }
    }
}
