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
                let width = w
                    .inner_width()
                    .ok()
                    .and_then(|val| val.as_f64())
                    .unwrap_or(800.0);
                let height = w
                    .inner_height()
                    .ok()
                    .and_then(|val| val.as_f64())
                    .unwrap_or(600.0);
                pan_x.set(-layout.center_point.0 + (width / 2.0));
                pan_y.set(-layout.center_point.1 + (height / 2.0));
            }
        }
    });

    rsx! {
        div {
            class: "app-map-container relative w-full h-full overflow-hidden bg-[var(--app-surface)]",
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
                        let current_scale = *scale.read();

                        // Viewport for clamping
                        let (vw, vh) = {
                            #[cfg(target_arch = "wasm32")]
                            {
                                if let Some(w) = web_sys::window() {
                                    let width = w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(800.0);
                                    let height = w.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(600.0);
                                    (width, height)
                                } else { (800.0, 600.0) }
                            }
                            #[cfg(not(target_arch = "wasm32"))] { (800.0, 600.0) }
                        };

                        let margin = 100.0;
                        
                        // New candidate positions
                        let next_px = px + dx;
                        let next_py = py + dy;

                        // Clamping logic:
                        // Left edge of layout (layout.min_x * scale + pan_x) should not be too far right.
                        // Right edge of layout (layout.max_x * scale + pan_x) should not be too far left.
                        // We always want to keep at least 'margin' pixels of layout visible.
                        let min_pan_x = (vw - margin) - layout.max_x * current_scale;
                        let max_pan_x = margin - layout.min_x * current_scale;
                        let min_pan_y = (vh - margin) - layout.max_y * current_scale;
                        let max_pan_y = margin - layout.min_y * current_scale;

                        let final_px = if min_pan_x <= max_pan_x {
                            next_px.clamp(min_pan_x, max_pan_x)
                        } else {
                            // Layout is smaller than viewport, keep it centered +/- margin
                            next_px.clamp(max_pan_x, min_pan_x)
                        };
                        let final_py = if min_pan_y <= max_pan_y {
                            next_py.clamp(min_pan_y, max_pan_y)
                        } else {
                            next_py.clamp(max_pan_y, min_pan_y)
                        };

                        pan_x.set(final_px);
                        pan_y.set(final_py);
                    }
                    last_mouse_pos.set(Some((current_x, current_y)));
                }
            },
            onwheel: move |e| {
                let (mouse_x, mouse_y) = (e.client_coordinates().x, e.client_coordinates().y);

                let delta_y = match e.data().delta() {
                    dioxus::prelude::dioxus_elements::geometry::WheelDelta::Pixels(v) => v.y,
                    dioxus::prelude::dioxus_elements::geometry::WheelDelta::Lines(v) => {
                        v.y * 20.0
                    }
                    _ => 0.0,
                };
                let zoom_factor = if delta_y > 0.0 { 0.9 } else { 1.1 };
                let current_scale = *scale.read();

                // Dynamic scale boundaries
                let (vw, vh) = {
                    #[cfg(target_arch = "wasm32")]
                    {
                        if let Some(w) = web_sys::window() {
                            let width = w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(800.0);
                            let height = w.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(600.0);
                            (width, height)
                        } else {
                            (800.0, 600.0)
                        }
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        (800.0, 600.0)
                    }
                };

                let layout_w = (layout.max_x - layout.min_x).max(100.0);
                let layout_h = (layout.max_y - layout.min_y).max(100.0);

                // Min scale: Fit the entire layout with a bit of padding
                let min_scale = (vw / (layout_w + 120.0))
                    .min(vh / (layout_h + 120.0))
                    .min(1.0);

                // Max scale: Zoom in until a card takes about half the screen width, or 3.0x
                let max_scale = (vw / 400.0).clamp(1.5, 4.0);

                let new_scale = (current_scale * zoom_factor).clamp(min_scale, max_scale);

                if new_scale != current_scale {
                    let px = *pan_x.read();
                    let py = *pan_y.read();
                    let svg_x = (mouse_x - px) / current_scale;
                    let svg_y = (mouse_y - py) / current_scale;
                    let next_px = mouse_x - svg_x * new_scale;
                    let next_py = mouse_y - svg_y * new_scale;

                    let margin = 100.0;
                    let min_pan_x = (vw - margin) - layout.max_x * new_scale;
                    let max_pan_x = margin - layout.min_x * new_scale;
                    let min_pan_y = (vh - margin) - layout.max_y * new_scale;
                    let max_pan_y = margin - layout.min_y * new_scale;

                    let final_px = if min_pan_x <= max_pan_x {
                        next_px.clamp(min_pan_x, max_pan_x)
                    } else {
                        next_px.clamp(max_pan_x, min_pan_x)
                    };
                    let final_py = if min_pan_y <= max_pan_y {
                        next_py.clamp(min_pan_y, max_pan_y)
                    } else {
                        next_py.clamp(max_pan_y, min_pan_y)
                    };

                    scale.set(new_scale);
                    pan_x.set(final_px);
                    pan_y.set(final_py);
                }
            },

            svg { class: "w-full h-full cursor-grab active:cursor-grabbing",

                g { transform: "translate({pan_x}, {pan_y}) scale({scale})",

                    // Edges
                    for edge in &layout.edges {
                        path {
                            key: "{edge.source_id}-{edge.target_id}",
                            d: "M {edge.source_point.0} {edge.source_point.1} L {edge.target_point.0} {edge.target_point.1}",
                            stroke: "var(--app-border-strong)",
                            stroke_width: "2",
                            fill: "none",
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
                                        class: "hover:stroke-[#f59e0b] hover:stroke-2 cursor-pointer transition-all duration-200 shadow-sm",
                                    }

                                    text {
                                        x: "{node_width / 2.0}",
                                        y: "{node_height / 2.0 + 5.0}",
                                        text_anchor: "middle",
                                        fill: "var(--app-text)",
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
