//! MiniMap component

use crate::state::FlowState;
use crate::utils::get_nodes_bounds;
use dioxus::prelude::*;
use dioxus::prelude::{PointerInteraction, ReadableExt};
use dioxus_web::WebEventExt;

#[component]
pub fn MiniMap<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    #[props(default)] class: Option<String>,
    #[props(default)] position: Option<String>,
    #[props(default = 120.0)] width: f64,
    #[props(default = 80.0)] height: f64,
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

    let rects: Vec<(f64, f64, f64, f64, bool)> = nodes
        .iter()
        .map(|node| {
            let dims = node.get_dimensions();
            (
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

    rsx! {
        div {
            class: "dioxus-flow__panel {position}",
            svg {
                class: "dioxus-flow__minimap {class}",
                width: "{width}",
                height: "{height}",
                onpointerdown: on_pointer_down,
                onpointermove: on_pointer_move,
                onpointerup: on_pointer_up,
                onpointerleave: on_pointer_up,
                onmounted: move |evt| {
                    let element: web_sys::Element = evt.as_web_event();
                    minimap_element.set(Some(element));
                },
                for rect in rects {
                    rect {
                        class: if rect.4 { "dioxus-flow__minimap-node selected" } else { "dioxus-flow__minimap-node" },
                        x: "{rect.0}",
                        y: "{rect.1}",
                        width: "{rect.2}",
                        height: "{rect.3}",
                        rx: "2",
                        ry: "2",
                    }
                }
                rect {
                    class: "dioxus-flow__minimap-viewport",
                    x: "{view_x}",
                    y: "{view_y}",
                    width: "{view_width}",
                    height: "{view_height}",
                    fill: "var(--df-minimap-mask-color)",
                    stroke: "var(--df-minimap-mask-stroke-color)",
                    stroke_width: "1",
                }
            }
        }
    }
}
