//! A horizontal strip minimap for localized navigation context.

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

    // The minimap shows local context
    let topology_result = build_graph_topology(current_card_id, &registry);

    let topology = match topology_result {
        Ok(t) => t,
        Err(_) => return rsx! { div { class: "hidden" } },
    };

    let layout = calculate_layout(&topology);

    // Map bounds for scaling
    let width = (layout.max_x - layout.min_x).max(100.0);
    let height = (layout.max_y - layout.min_y).max(100.0);

    let view_box = format!("{} {} {} {}",
        layout.min_x - 60.0, 
        layout.min_y - 30.0, 
        width + 120.0, 
        height + 60.0
    );

    rsx! {
        div {
            class: "app-minimap-strip w-full h-[12vh] min-h-[80px] bg-[var(--app-surface-soft)] border-b border-[var(--app-border-strong)] overflow-hidden cursor-pointer hover:bg-[var(--app-surface-strong)] transition-all duration-300 z-10 group relative",
            onclick: move |_| {
                navigator().push(Route::Map { focus_card_id: current_card_id });
            },

            // Label
            div { class: "absolute top-2 left-4 text-[10px] uppercase tracking-widest text-[var(--app-text-soft)] font-bold pointer-events-none opacity-60 group-hover:opacity-100 flex items-center gap-2", 
                span { "Navigation Map" }
                span { class: "text-[8px] bg-[var(--app-border)] px-1.5 py-0.5 rounded text-[var(--app-text-primary)]", "Click to expand" }
            }

            svg {
                class: "w-full h-full",
                view_box: "{view_box}",
                preserve_aspect_ratio: "xMidYMid meet",

                // Definitions for filters and gradients
                defs {
                    filter { id: "glow", x: "-20%", y: "-20%", width: "140%", height: "140%",
                        feGaussianBlur { std_deviation: "3", result: "blur" }
                        feComposite { in: "SourceGraphic", in2: "blur", operator: "over" }
                    }
                }

                // Edges
                for edge in &layout.edges {
                    path {
                        d: "M {edge.source_point.0} {edge.source_point.1} L {edge.target_point.0} {edge.target_point.1}",
                        stroke: "var(--app-border-strong)",
                        stroke_width: "2",
                        fill: "none",
                        opacity: "0.4",
                    }
                }

                // Nodes
                for node in &layout.nodes {
                    g {
                        class: "minimap-node",
                        
                        // Active node glow
                        if node.is_center {
                            circle {
                                cx: "{node.x}",
                                cy: "{node.y}",
                                r: "16",
                                fill: "#f59e0b",
                                opacity: "0.2",
                                filter: "url(#glow)"
                            }
                        }

                        circle {
                            cx: "{node.x}",
                            cy: "{node.y}",
                            r: if node.is_center { "12" } else { "8" },
                            fill: if node.is_center { "#f59e0b" } else { "var(--app-text-soft)" },    
                            stroke: if node.is_center { "white" } else { "var(--app-border-strong)" },
                            stroke_width: if node.is_center { "2" } else { "1" },
                            opacity: if node.is_center { "1.0" } else { "0.7" }
                        }

                        // Labels: Show for center node, or if we have space
                        if node.is_center || width < 600.0 {
                            text {
                                x: "{node.x}",
                                y: "{node.y + 28.0}",
                                text_anchor: "middle",
                                fill: if node.is_center { "var(--app-text-primary)" } else { "var(--app-text-soft)" },
                                font_size: "14px",
                                font_weight: if node.is_center { "bold" } else { "normal" },
                                class: "pointer-events-none select-none",
                                "{node.title}"
                            }
                        }
                    }
                }
            }
        }
    }
}
