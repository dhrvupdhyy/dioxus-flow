//! MiniMap component

use crate::state::FlowState;
use crate::types::Node;
use crate::utils::get_nodes_bounds;
use dioxus::prelude::*;
use dioxus::prelude::ReadableExt;
use dioxus_web::WebEventExt;

type MiniMapNodeAttr<N> = fn(&Node<N>) -> String;

#[component]
pub fn MiniMap<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    #[props(default)] class: Option<String>,
    #[props(default)] position: Option<String>,
    #[props(default = 120.0)] width: f64,
    #[props(default = 80.0)] height: f64,
    #[props(default)] node_color: Option<String>,
    #[props(default)] node_stroke_color: Option<String>,
    #[props(default)] node_class_name: Option<String>,
    #[props(default)] node_color_fn: Option<MiniMapNodeAttr<N>>,
    #[props(default)] node_stroke_color_fn: Option<MiniMapNodeAttr<N>>,
    #[props(default)] node_class_name_fn: Option<MiniMapNodeAttr<N>>,
    #[props(default = 1.0)] node_stroke_width: f64,
    #[props(default)] mask_color: Option<String>,
    #[props(default)] mask_stroke_color: Option<String>,
    #[props(default = true)] pannable: bool,
    #[props(default = false)] zoomable: bool,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] _marker: std::marker::PhantomData<(N, E)>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();
    let nodes = state
        .nodes
        .read()
        .iter()
        .filter(|n| !n.hidden)
        .cloned()
        .collect::<Vec<_>>();
    let position = position.unwrap_or_else(|| "bottom-right".to_string());
    let class = class.unwrap_or_default();
    let node_color = node_color.unwrap_or_else(|| "var(--df-node-background-color)".to_string());
    let node_stroke_color =
        node_stroke_color.unwrap_or_else(|| "var(--df-node-border-color)".to_string());
    let node_class_name = node_class_name.unwrap_or_default();
    let aria_label = aria_label
        .or_else(|| state.aria_label_config.read().minimap.clone())
        .unwrap_or_else(|| "Minimap".to_string());
    let mask_color =
        mask_color.unwrap_or_else(|| "var(--df-minimap-mask-color)".to_string());
    let mask_stroke_color =
        mask_stroke_color.unwrap_or_else(|| "var(--df-minimap-mask-stroke-color)".to_string());
    let mut dragging = use_signal(|| false);
    let mut minimap_element = use_signal(|| None::<web_sys::Element>);
    let mut bounds = get_nodes_bounds(&nodes);
    let pad = 0.1;
    bounds.x -= bounds.width * pad;
    bounds.y -= bounds.height * pad;
    bounds.width *= 1.0 + pad * 2.0;
    bounds.height *= 1.0 + pad * 2.0;
    if bounds.width <= 0.0 {
        bounds.width = 1.0;
    }
    if bounds.height <= 0.0 {
        bounds.height = 1.0;
    }
    let scale_x = width / bounds.width;
    let scale_y = height / bounds.height;
    let scale = scale_x.min(scale_y);
    let offset_x = (width - bounds.width * scale) / 2.0;
    let offset_y = (height - bounds.height * scale) / 2.0;

    let rects: Vec<(String, f64, f64, f64, f64, bool)> = nodes
        .iter()
        .map(|node| {
            let dims = node.get_dimensions();
            (
                node.id.clone(),
                offset_x + (node.position.x - bounds.x) * scale,
                offset_y + (node.position.y - bounds.y) * scale,
                dims.width * scale,
                dims.height * scale,
                node.selected,
            )
        })
        .collect();

    let viewport = *state.viewport.read();
    let view_x = (-viewport.x / viewport.zoom - bounds.x) * scale + offset_x;
    let view_y = (-viewport.y / viewport.zoom - bounds.y) * scale + offset_y;
    let width_value = *state.width.read();
    let height_value = *state.height.read();
    let view_width = (width_value / viewport.zoom) * scale;
    let view_height = (height_value / viewport.zoom) * scale;

    let mut state_drag = state.clone();
    let on_pointer_down = move |evt: PointerEvent| {
        if !pannable {
            return;
        }
        let Some(element) = minimap_element.read().clone() else {
            return;
        };
        let coords = evt.data.client_coordinates();
        let rect = element.get_bounding_client_rect();
        let local_x = coords.x - rect.x();
        let local_y = coords.y - rect.y();
        let flow_x = (local_x - offset_x) / scale + bounds.x;
        let flow_y = (local_y - offset_y) / scale + bounds.y;
        state_drag.set_center(flow_x, flow_y, None);
        dragging.set(true);
    };

    let mut state_move = state.clone();
    let on_pointer_move = move |evt: PointerEvent| {
        if !pannable {
            return;
        }
        if !*dragging.read() {
            return;
        }
        let Some(element) = minimap_element.read().clone() else {
            return;
        };
        let coords = evt.data.client_coordinates();
        let rect = element.get_bounding_client_rect();
        let local_x = coords.x - rect.x();
        let local_y = coords.y - rect.y();
        let flow_x = (local_x - offset_x) / scale + bounds.x;
        let flow_y = (local_y - offset_y) / scale + bounds.y;
        state_move.set_center(flow_x, flow_y, None);
    };

    let on_pointer_up = move |_evt: PointerEvent| {
        dragging.set(false);
    };

    let mut state_zoom = state.clone();
    let on_wheel = move |evt: WheelEvent| {
        if !zoomable {
            return;
        }
        evt.prevent_default();
        let delta = match evt.data.delta() {
            dioxus::prelude::dioxus_elements::geometry::WheelDelta::Pixels(v) => v.y,
            dioxus::prelude::dioxus_elements::geometry::WheelDelta::Lines(v) => v.y * 16.0,
            dioxus::prelude::dioxus_elements::geometry::WheelDelta::Pages(v) => v.y * 100.0,
        };
        if delta > 0.0 {
            state_zoom.zoom_out(Some(1.1));
        } else {
            state_zoom.zoom_in(Some(1.1));
        }
    };

    rsx! {
        div {
            class: "dioxus-flow__panel {position}",
            svg {
                class: "dioxus-flow__minimap {class}",
                width: "{width}",
                height: "{height}",
                "aria-label": "{aria_label}",
                onpointerdown: on_pointer_down,
                onpointermove: on_pointer_move,
                onpointerup: on_pointer_up,
                onpointerleave: on_pointer_up,
                onwheel: on_wheel,
                onmounted: move |evt| {
                    let element: web_sys::Element = evt.as_web_event();
                    minimap_element.set(Some(element));
                },
                for rect in rects {
                    rect {
                        class: {
                            let custom = node_class_name_fn
                                .and_then(|func| nodes.iter().find(|n| n.id == rect.0).map(|n| func(n)))
                                .unwrap_or_else(|| node_class_name.clone());
                            if rect.5 {
                                format!("dioxus-flow__minimap-node selected {}", custom)
                            } else {
                                format!("dioxus-flow__minimap-node {}", custom)
                            }
                        },
                        x: "{rect.1}",
                        y: "{rect.2}",
                        width: "{rect.3}",
                        height: "{rect.4}",
                        rx: "2",
                        ry: "2",
                        fill: {
                            if rect.5 {
                                "var(--df-node-border-selected-color)".to_string()
                            } else if let Some(func) = node_color_fn {
                                nodes
                                    .iter()
                                    .find(|n| n.id == rect.0)
                                    .map(|n| func(n))
                                    .unwrap_or_else(|| node_color.clone())
                            } else {
                                node_color.clone()
                            }
                        },
                        stroke: {
                            if let Some(func) = node_stroke_color_fn {
                                nodes
                                    .iter()
                                    .find(|n| n.id == rect.0)
                                    .map(|n| func(n))
                                    .unwrap_or_else(|| node_stroke_color.clone())
                            } else {
                                node_stroke_color.clone()
                            }
                        },
                        stroke_width: "{node_stroke_width}",
                    }
                }
                rect {
                    class: "dioxus-flow__minimap-viewport",
                    x: "{view_x}",
                    y: "{view_y}",
                    width: "{view_width}",
                    height: "{view_height}",
                    fill: "{mask_color}",
                    stroke: "{mask_stroke_color}",
                    stroke_width: "1",
                }
            }
        }
    }
}
