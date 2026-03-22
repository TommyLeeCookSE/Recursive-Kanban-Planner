//! A small, floating minimap for localized navigation context.

use crate::application::build_graph_topology;
use crate::domain::id::CardId;
use crate::interface::Route;
use crate::interface::actions::map_layout::calculate_layout;
use crate::interface::app::use_board_signals;
use dioxus::prelude::*;

#[component]
pub fn Minimap(current_card_id: CardId) -> Element {
    let signals = use_board_signals();
    let registry = signals.registry.read();

    // The minimap is small, so we only want local neighborhood
    let topology_result = build_graph_topology(current_card_id, &registry);

    let topology = match topology_result {
        Ok(t) => t,
        Err(_) => return rsx! { div {} },
    };

    let layout = calculate_layout(&topology);

    // Map bounds for scaling
    let width = (layout.max_x - layout.min_x).max(100.0);
    let height = (layout.max_y - layout.min_y).max(100.0);
    
    // Scale everything down to fit a small box (e.g. 150px)
    let max_dim = width.max(height);
    let scale_factor = 140.0 / max_dim;
    
    let view_box = format!("{} {} {} {}", 
        layout.min_x - 10.0, 
        layout.min_y - 10.0, 
        width + 20.0, 
        height + 20.0
    );

    rsx! {
        div {
            class: "app-minimap-container fixed bottom-24 right-6 w-40 h-40 bg-[var(--app-surface-soft)] border border-[var(--app-border-strong)] rounded-xl shadow-lg backdrop-blur-md overflow-hidden cursor-pointer hover:border-[#f59e0b] transition-all duration-300 z-40 group",
            onclick: move |_| {
                navigator().push(Route::MapRoute { focus_card_id: current_card_id });
            },
            
            // Label
            div { class: "absolute top-1 left-2 text-[10px] uppercase tracking-widest text-[var(--app-text-soft)] font-bold pointer-events-none opacity-60 group-hover:opacity-100",
                "Map"
            }

            svg {
                class: "w-full h-full",
                view_box: "{view_box}",
                
                // Edges
                for edge in &layout.edges {
                    path {
                        d: "M {edge.source_point.0} {edge.source_point.1} L {edge.target_point.0} {edge.target_point.1}",
                        stroke: "var(--app-border-strong)",
                        stroke_width: "4",
                        fill: "none",
                        opacity: "0.4"
                    }
                }

                // Nodes
                for node in &layout.nodes {
                    circle {
                        cx: "{node.x}",
                        cy: "{node.y}",
                        r: if node.is_center { "15" } else { "10" },
                        fill: if node.is_center { "#f59e0b" } else { "var(--app-text-soft)" },
                        stroke: if node.is_center { "white" } else { "none" },
                        stroke_width: "3"
                    }
                }
            }
        }
    }
}
