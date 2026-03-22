//! Full-screen interactive graph map route.
//!
//! This module renders a pannable and zoomable SVG representation of the
//! workspace topology.

use crate::application::build_graph_topology;
use crate::domain::id::CardId;
use crate::interface::Route;
use crate::interface::actions::map_layout::calculate_layout;
use crate::interface::app::use_board_signals;
use dioxus::prelude::*;

#[component]
pub fn MapScreen(focus_card_id: CardId) -> Element {
    let signals = use_board_signals();
    let registry = signals.registry.read();

    let topology_result = build_graph_topology(focus_card_id, &registry);

    let topology = match topology_result {
        Ok(t) => t,
        Err(e) => {
            return rsx! {
                div { class: "app-page-shell",
                    div { class: "app-empty-state",
                        p { class: "app-error-message", "Failed to load map: {e}" }
                    }
                }
            };
        }
    };

    let layout = calculate_layout(&topology);

    // Viewport state
    let mut pan_x = use_signal(|| {
        let px = -layout.center_point.0;
        #[cfg(target_arch = "wasm32")]
        {
            // Code that would modify px if it were mut
        }
        px
    });
    let mut pan_y = use_signal(|| {
        let py = -layout.center_point.1;
        #[cfg(target_arch = "wasm32")]
        {
            // Code that would modify py if it were mut
        }
        py
    });
    let mut scale = use_signal(|| 1.0f64);
    let mut is_dragging = use_signal(|| false);

    let mut last_mouse_pos = use_signal(|| None::<(f64, f64)>);

    // Initial center adjustment on mount
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                let width = w.inner_width().ok().and_then(|val| val.as_f64()).unwrap_or(800.0);
                let height = w.inner_height().ok().and_then(|val| val.as_f64()).unwrap_or(600.0);
                pan_x.set(-layout.center_point.0 + (width / 2.0));
                pan_y.set(-layout.center_point.1 + (height / 2.0));
            }
        }
    });

    rsx! {
        div {
            class: "app-map-container absolute inset-0 overflow-hidden bg-[var(--app-surface)]",
            onmousedown: move |e| {
                is_dragging.set(true);
                last_mouse_pos.set(Some((e.client_coordinates().x, e.client_coordinates().y)));
            },
            onmouseup: move |_| {
                is_dragging.set(false);
                last_mouse_pos.set(None);
            },
            onmouseleave: move |_| {
                is_dragging.set(false);
                last_mouse_pos.set(None);
            },
            onmousemove: move |e| {
                if *is_dragging.read() {
                    let current_x = e.client_coordinates().x;
                    let current_y = e.client_coordinates().y;
                    
                    if let Some((last_x, last_y)) = *last_mouse_pos.read() {
                        let dx = current_x - last_x;
                        let dy = current_y - last_y;
                        let px = *pan_x.read();
                        let py = *pan_y.read();
                        pan_x.set(px + dx);
                        pan_y.set(py + dy);
                    }
                    last_mouse_pos.set(Some((current_x, current_y)));
                }
            },
            onwheel: move |e| {
                let (mouse_x, mouse_y) = (e.client_coordinates().x, e.client_coordinates().y);
                
                let delta_y = match e.data().delta() {
                    dioxus::prelude::dioxus_elements::geometry::WheelDelta::Pixels(v) => v.y,
                    dioxus::prelude::dioxus_elements::geometry::WheelDelta::Lines(v) => v.y * 20.0,
                    _ => 0.0,
                };
                
                let zoom_factor = if delta_y > 0.0 { 0.9 } else { 1.1 };
                let current_scale = *scale.read();
                let new_scale = (current_scale * zoom_factor).clamp(0.1, 3.0);
                
                if new_scale != current_scale {
                    let mut px = *pan_x.read();
                    let mut py = *pan_y.read();
                    
                    // Zoom towards mouse:
                    // 1. Calculate mouse position in SVG coordinate space
                    let svg_x = (mouse_x - px) / current_scale;
                    let svg_y = (mouse_y - py) / current_scale;
                    
                    // 2. Adjust pan to keep that SVG point under the mouse with the new scale
                    px = mouse_x - svg_x * new_scale;
                    py = mouse_y - svg_y * new_scale;
                    
                    scale.set(new_scale);
                    pan_x.set(px);
                    pan_y.set(py);
                }
            },
            
            button {
                class: "app-button-ghost-compact absolute top-4 left-4 z-10 bg-[var(--app-surface-strong)] shadow-md",
                onclick: move |_| {
                    navigator().go_back();
                },
                "Close Map"
            }

            svg {
                class: "w-full h-full cursor-grab active:cursor-grabbing",
                
                g {
                    transform: "translate({pan_x}, {pan_y}) scale({scale})",
                    
                    // Edges
                    for edge in &layout.edges {
                        path {
                            key: "{edge.source_id}-{edge.target_id}",
                            d: "M {edge.source_point.0} {edge.source_point.1} L {edge.target_point.0} {edge.target_point.1}",
                            stroke: "var(--app-border-strong)",
                            stroke_width: "2",
                            fill: "none"
                        }
                    }

                    // Nodes
                    for node in &layout.nodes {
                        {
                            let node_id = node.id;
                            let node_x = node.x;
                            let node_y = node.y;
                            let node_width = node.width;
                            let node_height = node.height;
                            let node_title = node.title.clone();
                            let is_center = node.is_center;

                            rsx! {
                                g {
                                    key: "{node_id}",
                                    transform: "translate({node_x - node_width / 2.0}, {node_y - node_height / 2.0})",
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        navigator().push(Route::Board { card_id: node_id });
                                    },
                                    
                                    rect {
                                        width: "{node_width}",
                                        height: "{node_height}",
                                        rx: "12",
                                        fill: if is_center { "var(--app-surface-soft)" } else { "var(--app-card)" },
                                        stroke: if is_center { "#f59e0b" } else { "var(--app-border)" },
                                        stroke_width: if is_center { "3" } else { "1" },
                                        class: "hover:stroke-[#f59e0b] hover:stroke-2 cursor-pointer transition-all duration-200 shadow-sm"
                                    }
                                    
                                    text {
                                        x: "{node_width / 2.0}",
                                        y: "{node_height / 2.0 + 5.0}",
                                        text_anchor: "middle",
                                        fill: "var(--app-text-primary)",
                                        font_size: "14px",
                                        font_weight: "bold",
                                        class: "pointer-events-none select-none",
                                        "{node_title}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
