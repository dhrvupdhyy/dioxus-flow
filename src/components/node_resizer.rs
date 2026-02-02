//! Node resizer component

use dioxus::prelude::dioxus_elements::input_data::MouseButton;
use dioxus::prelude::*;
use dioxus::prelude::{try_use_context, PointerInteraction, ReadableExt};

use crate::state::{FlowState, NodeIdContext};
use crate::types::{
    CoordinateExtent, Dimensions, HandleBound, HandleBounds, HandleType, NodeExtent,
    NodeResizeEvent, Position, ShouldResize, XYPosition,
};
use wasm_bindgen::JsCast;

#[component]
pub fn NodeResizer<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    #[props(default)] node_id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] is_visible: Option<bool>,
    #[props(default)] handle_class: Option<String>,
    #[props(default)] handle_style: Option<String>,
    #[props(default)] line_class: Option<String>,
    #[props(default)] line_style: Option<String>,
    #[props(default)] color: Option<String>,
    #[props(default = 10.0)] min_width: f64,
    #[props(default = 10.0)] min_height: f64,
    #[props(default)] max_width: Option<f64>,
    #[props(default)] max_height: Option<f64>,
    #[props(default)] keep_aspect_ratio: bool,
    #[props(default = true)] auto_scale: bool,
    #[props(default)] should_resize: Option<ShouldResize<N>>,
    #[props(default)] on_resize_start: Option<EventHandler<NodeResizeEvent<N>>>,
    #[props(default)] on_resize: Option<EventHandler<NodeResizeEvent<N>>>,
    #[props(default)] on_resize_end: Option<EventHandler<NodeResizeEvent<N>>>,
    #[props(default)] _marker: std::marker::PhantomData<(N, E)>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();
    let context_id = try_use_context::<NodeIdContext>().map(|ctx| ctx.0);
    let node_id = node_id.or(context_id);
    let Some(node_id) = node_id else {
        return rsx! {};
    };

    let internal = state.node_lookup.read().get(&node_id).cloned();
    let Some(internal) = internal else {
        return rsx! {};
    };

    let active = is_visible.unwrap_or(true);
    if !active {
        return rsx! {};
    }

    let class = class.unwrap_or_default();
    let handle_class = handle_class.unwrap_or_default();
    let handle_style = handle_style.unwrap_or_default();
    let line_class = line_class.unwrap_or_default();
    let line_style = line_style.unwrap_or_default();
    let color = color.unwrap_or_else(|| "var(--df-node-resizer-color, #1a192b)".to_string());
    let mut resize_state = use_signal(|| None::<ResizeState<N>>);
    let node_id_move = node_id.clone();
    let mut state_move = state.clone();

    let on_pointer_move = move |evt: PointerEvent| {
        let Some(state_value) = resize_state.read().clone() else {
            return;
        };
        let coords = evt.data.client_coordinates();
        let flow_pos = state_move.screen_to_flow_position(XYPosition::new(coords.x, coords.y));
        let delta = XYPosition {
            x: flow_pos.x - state_value.start_pointer.x,
            y: flow_pos.y - state_value.start_pointer.y,
        };

        let mut next_width = state_value.start_dimensions.width;
        let mut next_height = state_value.start_dimensions.height;
        let mut next_position = state_value.start_position;

        match state_value.handle {
            ResizeHandle::TopLeft => {
                next_width -= delta.x;
                next_height -= delta.y;
            }
            ResizeHandle::TopRight => {
                next_width += delta.x;
                next_height -= delta.y;
            }
            ResizeHandle::BottomLeft => {
                next_width -= delta.x;
                next_height += delta.y;
            }
            ResizeHandle::BottomRight => {
                next_width += delta.x;
                next_height += delta.y;
            }
            ResizeHandle::Top => {
                next_height -= delta.y;
            }
            ResizeHandle::Right => {
                next_width += delta.x;
            }
            ResizeHandle::Bottom => {
                next_height += delta.y;
            }
            ResizeHandle::Left => {
                next_width -= delta.x;
            }
        }

        if next_width.is_nan() || next_height.is_nan() {
            return;
        }

        let max_w = max_width.unwrap_or(f64::INFINITY);
        let max_h = max_height.unwrap_or(f64::INFINITY);
        let mut clamped_width = next_width.clamp(min_width, max_w);
        let mut clamped_height = next_height.clamp(min_height, max_h);

        if keep_aspect_ratio {
            let start_w = state_value.start_dimensions.width.max(1.0);
            let start_h = state_value.start_dimensions.height.max(1.0);
            let width_scale = clamped_width / start_w;
            let height_scale = clamped_height / start_h;
            let scale = if matches!(state_value.handle, ResizeHandle::Left | ResizeHandle::Right) {
                width_scale
            } else if matches!(state_value.handle, ResizeHandle::Top | ResizeHandle::Bottom) {
                height_scale
            } else if (clamped_width - start_w).abs() > (clamped_height - start_h).abs() {
                width_scale
            } else {
                height_scale
            };
            clamped_width = (start_w * scale).clamp(min_width, max_w);
            clamped_height = (start_h * scale).clamp(min_height, max_h);
        }

        if matches!(
            state_value.handle,
            ResizeHandle::TopLeft | ResizeHandle::BottomLeft | ResizeHandle::Left
        ) {
            next_position.x += state_value.start_dimensions.width - clamped_width;
        }
        if matches!(
            state_value.handle,
            ResizeHandle::TopLeft | ResizeHandle::TopRight | ResizeHandle::Top
        ) {
            next_position.y += state_value.start_dimensions.height - clamped_height;
        }

        let (next_position, clamped_width, clamped_height) = clamp_resize_to_extent(
            &state_move,
            &state_value.node,
            next_position,
            clamped_width,
            clamped_height,
        );

        if let Some(should_resize) = should_resize {
            if !should_resize(
                &state_value.node,
                Dimensions {
                    width: clamped_width,
                    height: clamped_height,
                },
            ) {
                return;
            }
        }

        let changes = vec![
            crate::types::NodeChange::Position {
                id: node_id_move.clone(),
                position: Some(next_position),
                dragging: false,
            },
            crate::types::NodeChange::Dimensions {
                id: node_id_move.clone(),
                dimensions: Some(Dimensions {
                    width: clamped_width,
                    height: clamped_height,
                }),
                resizing: true,
            },
        ];
        state_move.apply_node_changes(changes);
        update_handle_bounds_from_dom(&mut state_move, &node_id_move);

        if let Some(handler) = &on_resize {
            handler.call(NodeResizeEvent {
                node: state_value.node.clone(),
                dimensions: Dimensions {
                    width: clamped_width,
                    height: clamped_height,
                },
            });
        }
    };

    let mut state_up = state.clone();
    let node_id_up = node_id.clone();
    let on_pointer_up = move |_evt: PointerEvent| {
        if resize_state.read().is_some() {
            if let Some(handler) = &on_resize_end {
                if let Some(internal) = state_up.node_lookup.read().get(&node_id_up).cloned() {
                    handler.call(NodeResizeEvent {
                        node: internal.node.clone(),
                        dimensions: internal.node.get_dimensions(),
                    });
                }
            }
            let change = crate::types::NodeChange::Dimensions {
                id: node_id_up.clone(),
                dimensions: None,
                resizing: false,
            };
            state_up.apply_node_changes(vec![change]);
            update_handle_bounds_from_dom(&mut state_up, &node_id_up);
            resize_state.set(None);
        }
    };

    let state_for_start = state.clone();
    let start_handle = move |handle: ResizeHandle, node_id_start: String| {
        let mut state_start = state_for_start.clone();
        let mut resize_state = resize_state.clone();
        let on_resize_start = on_resize_start.clone();
        move |evt: PointerEvent| {
            if evt.data.trigger_button() != Some(MouseButton::Primary) {
                return;
            }
            evt.stop_propagation();
            let Some(internal) = state_start.node_lookup.read().get(&node_id_start).cloned() else {
                return;
            };
            let coords = evt.data.client_coordinates();
            let flow_pos = state_start.screen_to_flow_position(XYPosition::new(coords.x, coords.y));
            let node = internal.node.clone();
            resize_state.set(Some(ResizeState {
                start_pointer: flow_pos,
                start_dimensions: node.get_dimensions(),
                start_position: node.position,
                handle,
                node: node.clone(),
            }));
            let change = crate::types::NodeChange::Dimensions {
                id: node_id_start.clone(),
                dimensions: None,
                resizing: true,
            };
            state_start.apply_node_changes(vec![change]);
            if let Some(handler) = &on_resize_start {
                handler.call(NodeResizeEvent {
                    node,
                    dimensions: internal.node.get_dimensions(),
                });
            }
        }
    };

    let zoom = state.viewport.read().zoom;
    let scale = if auto_scale {
        1.0 / zoom.max(0.0001)
    } else {
        1.0
    };
    let scale_style = if (scale - 1.0).abs() > f64::EPSILON {
        format!("transform: scale({}); transform-origin: top left;", scale)
    } else {
        String::new()
    };
    let handle_inline_style = format!("{} border-color: {};", handle_style, color);
    let line_inline_style = format!("{} background: {};", line_style, color);

    rsx! {
        div {
            class: "dioxus-flow__node-resizer {class}",
            style: "{scale_style}",
            onpointermove: on_pointer_move,
            onpointerup: on_pointer_up,
            div {
                class: "dioxus-flow__node-resizer-line top {line_class}",
                style: "{line_inline_style}",
            }
            div {
                class: "dioxus-flow__node-resizer-line right {line_class}",
                style: "{line_inline_style}",
            }
            div {
                class: "dioxus-flow__node-resizer-line bottom {line_class}",
                style: "{line_inline_style}",
            }
            div {
                class: "dioxus-flow__node-resizer-line left {line_class}",
                style: "{line_inline_style}",
            }
            div {
                class: "dioxus-flow__node-resizer-handle top-left {handle_class}",
                style: "{handle_inline_style}",
                onpointerdown: start_handle(ResizeHandle::TopLeft, node_id.clone()),
            }
            div {
                class: "dioxus-flow__node-resizer-handle top-right {handle_class}",
                style: "{handle_inline_style}",
                onpointerdown: start_handle(ResizeHandle::TopRight, node_id.clone()),
            }
            div {
                class: "dioxus-flow__node-resizer-handle bottom-left {handle_class}",
                style: "{handle_inline_style}",
                onpointerdown: start_handle(ResizeHandle::BottomLeft, node_id.clone()),
            }
            div {
                class: "dioxus-flow__node-resizer-handle bottom-right {handle_class}",
                style: "{handle_inline_style}",
                onpointerdown: start_handle(ResizeHandle::BottomRight, node_id.clone()),
            }
            div {
                class: "dioxus-flow__node-resizer-handle top {handle_class}",
                style: "{handle_inline_style}",
                onpointerdown: start_handle(ResizeHandle::Top, node_id.clone()),
            }
            div {
                class: "dioxus-flow__node-resizer-handle right {handle_class}",
                style: "{handle_inline_style}",
                onpointerdown: start_handle(ResizeHandle::Right, node_id.clone()),
            }
            div {
                class: "dioxus-flow__node-resizer-handle bottom {handle_class}",
                style: "{handle_inline_style}",
                onpointerdown: start_handle(ResizeHandle::Bottom, node_id.clone()),
            }
            div {
                class: "dioxus-flow__node-resizer-handle left {handle_class}",
                style: "{handle_inline_style}",
                onpointerdown: start_handle(ResizeHandle::Left, node_id.clone()),
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ResizeHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Clone, PartialEq, Debug)]
struct ResizeState<T: Clone + PartialEq + Default = ()> {
    start_pointer: XYPosition,
    start_dimensions: Dimensions,
    start_position: XYPosition,
    handle: ResizeHandle,
    node: crate::types::Node<T>,
}

fn clamp_resize_to_extent<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &FlowState<N, E>,
    node: &crate::types::Node<N>,
    position: XYPosition,
    width: f64,
    height: f64,
) -> (XYPosition, f64, f64) {
    let extent = node.extent.clone().or_else(|| {
        state
            .node_extent
            .read()
            .as_ref()
            .map(|extent| NodeExtent::CoordinateExtent(*extent))
    });

    match extent {
        Some(NodeExtent::Parent) => {
            if let Some(parent_id) = &node.parent_id {
                if let Some(parent) = state.node_lookup.read().get(parent_id) {
                    let max_w = parent.dimensions.width.max(0.0);
                    let max_h = parent.dimensions.height.max(0.0);
                    let clamped_w = width.min(max_w);
                    let clamped_h = height.min(max_h);
                    let max_x = (max_w - clamped_w).max(0.0);
                    let max_y = (max_h - clamped_h).max(0.0);
                    let clamped_pos = XYPosition {
                        x: position.x.clamp(0.0, max_x),
                        y: position.y.clamp(0.0, max_y),
                    };
                    return (clamped_pos, clamped_w, clamped_h);
                }
            }
            (position, width, height)
        }
        Some(NodeExtent::CoordinateExtent(extent)) => {
            let parent_abs = if let Some(parent_id) = &node.parent_id {
                state
                    .node_lookup
                    .read()
                    .get(parent_id)
                    .map(|p| p.position_absolute)
                    .unwrap_or_else(|| XYPosition::new(0.0, 0.0))
            } else {
                XYPosition::new(0.0, 0.0)
            };
            let abs_pos = XYPosition {
                x: position.x + parent_abs.x,
                y: position.y + parent_abs.y,
            };
            let (abs_pos, width, height) = clamp_rect_to_extent(extent, abs_pos, width, height);
            let local_pos = XYPosition {
                x: abs_pos.x - parent_abs.x,
                y: abs_pos.y - parent_abs.y,
            };
            (local_pos, width, height)
        }
        None => (position, width, height),
    }
}

fn clamp_rect_to_extent(
    extent: CoordinateExtent,
    position: XYPosition,
    width: f64,
    height: f64,
) -> (XYPosition, f64, f64) {
    let min_x = extent[0][0];
    let min_y = extent[0][1];
    let max_x = extent[1][0];
    let max_y = extent[1][1];

    let max_width = if max_x.is_finite() && min_x.is_finite() {
        (max_x - min_x).max(0.0)
    } else {
        f64::INFINITY
    };
    let max_height = if max_y.is_finite() && min_y.is_finite() {
        (max_y - min_y).max(0.0)
    } else {
        f64::INFINITY
    };

    let clamped_w = width.min(max_width);
    let clamped_h = height.min(max_height);
    let max_x = if max_x.is_finite() {
        max_x - clamped_w
    } else {
        max_x
    };
    let max_y = if max_y.is_finite() {
        max_y - clamped_h
    } else {
        max_y
    };

    let clamped_pos = XYPosition {
        x: position.x.clamp(min_x, max_x),
        y: position.y.clamp(min_y, max_y),
    };

    (clamped_pos, clamped_w, clamped_h)
}

fn update_handle_bounds_from_dom<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &mut FlowState<N, E>,
    node_id: &str,
) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let selector = format!("[data-id=\"{}\"]", node_id.replace('\"', "\\\""));
    let Ok(Some(element)) = document.query_selector(&selector) else {
        return;
    };
    if let Some(bounds) = compute_handle_bounds(&element) {
        state.update_handle_bounds(node_id, bounds);
    }
}

fn compute_handle_bounds(element: &web_sys::Element) -> Option<HandleBounds> {
    let node_rect = element.get_bounding_client_rect();
    let handles = element.query_selector_all(".dioxus-flow__handle").ok()?;
    let mut bounds = HandleBounds::default();

    for index in 0..handles.length() {
        let handle: web_sys::Element = handles
            .get(index)
            .and_then(|h| h.dyn_into::<web_sys::Element>().ok())?;
        let rect = handle.get_bounding_client_rect();
        let x = rect.x() - node_rect.x();
        let y = rect.y() - node_rect.y();
        let width = rect.width();
        let height = rect.height();
        let id = handle
            .get_attribute("data-handle-id")
            .filter(|v: &String| !v.is_empty());
        let class_name = handle.get_attribute("class").unwrap_or_default();

        let position = if class_name.contains("dioxus-flow__handle-left") {
            Position::Left
        } else if class_name.contains("dioxus-flow__handle-right") {
            Position::Right
        } else if class_name.contains("dioxus-flow__handle-top") {
            Position::Top
        } else {
            Position::Bottom
        };

        let handle_type = if class_name.contains("dioxus-flow__handle-target") {
            HandleType::Target
        } else {
            HandleType::Source
        };

        let is_connectable = class_name.contains("connectable");
        let bound = HandleBound {
            id,
            position,
            x,
            y,
            width,
            height,
            is_connectable,
        };

        match handle_type {
            HandleType::Source => bounds.source.push(bound),
            HandleType::Target => bounds.target.push(bound),
        }
    }

    Some(bounds)
}
