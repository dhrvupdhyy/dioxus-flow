//! Graph view component

use crate::components::{EdgeRenderer, NodeRenderer, PanZoomPane, SelectionListener};
use crate::state::FlowState;
use crate::types::{HandleBound, HandleBounds, HandleType, Position};
use crate::utils::{
    get_bezier_path, get_simple_bezier_path, get_smooth_step_path, get_step_path, get_straight_path,
};
use dioxus::prelude::ReadableExt;
use dioxus::prelude::*;
use std::collections::HashMap;
use wasm_bindgen::JsCast;

#[component]
pub fn GraphView<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    #[props(default)] node_types: Option<
        HashMap<String, Component<crate::components::NodeProps<N, E>>>,
    >,
    #[props(default)] edge_types: Option<
        HashMap<String, Component<crate::components::EdgeComponentProps<E>>>,
    >,
    #[props(default)] on_nodes_change: Option<EventHandler<Vec<crate::types::NodeChange<N>>>>,
    #[props(default)] on_edges_change: Option<EventHandler<Vec<crate::types::EdgeChange<E>>>>,
    #[props(default)] on_connect: Option<EventHandler<crate::types::Connection>>,
    #[props(default)] on_edge_update_start: Option<EventHandler<crate::types::Edge<E>>>,
    #[props(default)] on_edge_update: Option<EventHandler<crate::types::EdgeUpdateEvent<E>>>,
    #[props(default)] on_edge_update_end: Option<EventHandler<crate::types::EdgeUpdateEndEvent<E>>>,
    #[props(default)] on_selection_change: Option<
        EventHandler<crate::types::SelectionChange<N, E>>,
    >,
    #[props(default)] on_node_drag_start: Option<EventHandler<crate::types::NodeDragEvent<N>>>,
    #[props(default)] on_node_drag: Option<EventHandler<crate::types::NodeDragEvent<N>>>,
    #[props(default)] on_node_drag_stop: Option<EventHandler<crate::types::NodeDragEvent<N>>>,
    #[props(default)] on_move: Option<EventHandler<crate::types::Viewport>>,
    #[props(default)] on_move_start: Option<EventHandler<crate::types::Viewport>>,
    #[props(default)] on_move_end: Option<EventHandler<crate::types::Viewport>>,
    #[props(default)] on_selection_start: Option<EventHandler<crate::types::SelectionStartEvent>>,
    #[props(default)] on_selection_end: Option<
        EventHandler<crate::types::SelectionEndEvent<N, E>>,
    >,
    #[props(default)] on_node_click: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_node_double_click: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_node_mouse_enter: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_node_mouse_leave: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_edge_click: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
    #[props(default)] on_edge_double_click: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
    #[props(default)] on_edge_mouse_enter: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
    #[props(default)] on_edge_mouse_leave: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();

    let viewport = *state.viewport.read();
    let transform = format!(
        "transform: translate({}px, {}px) scale({});",
        viewport.x, viewport.y, viewport.zoom
    );

    let mut last_zoom = use_signal(|| viewport.zoom);
    let mut last_handle_bounds_zoom = use_signal(|| viewport.zoom);
    let mut state_zoom = state.clone();
    use_effect(move || {
        let zoom = state_zoom.viewport.read().zoom;
        if (zoom - *last_zoom.read()).abs() < 0.0001 {
            return;
        }
        last_zoom.set(zoom);
        let connecting = state_zoom.connection.read().in_progress;
        if !connecting {
            return;
        }
        if (zoom - *last_handle_bounds_zoom.read()).abs() < 0.02 {
            return;
        }
        last_handle_bounds_zoom.set(zoom);
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Ok(node_elements) = document.query_selector_all(".dioxus-flow__node") else {
            return;
        };
        for idx in 0..node_elements.length() {
            let Some(element) = node_elements
                .get(idx)
                .and_then(|el| el.dyn_into::<web_sys::Element>().ok())
            else {
                continue;
            };
            let Some(node_id) = element.get_attribute("data-id") else {
                continue;
            };
            if let Some(bounds) = compute_handle_bounds_for_zoom(&element, zoom.max(0.0001)) {
                state_zoom.update_handle_bounds(&node_id, bounds);
            }
        }
    });

    let selection = state.user_selection_rect.read().clone();
    let selection_style = selection.as_ref().map(|rect| {
        let x = rect.x * viewport.zoom + viewport.x;
        let y = rect.y * viewport.zoom + viewport.y;
        let width = rect.width * viewport.zoom;
        let height = rect.height * viewport.zoom;
        format!(
            "transform: translate({}px, {}px); width: {}px; height: {}px;",
            x, y, width, height
        )
    });

    rsx! {
        PanZoomPane::<N, E> {
            on_move,
            on_move_start,
            on_move_end,
            on_selection_start,
            on_selection_end,
            on_nodes_change,
            on_edges_change,
            on_connect,
            on_edge_update,
            on_edge_update_end,
            on_node_drag,
            on_node_drag_stop,

            div {
                class: "dioxus-flow__viewport",
                style: "{transform}",

                {connection_line_element(&state)}

                EdgeRenderer::<N, E> {
                    edge_types,
                    on_nodes_change,
                    on_edges_change,
                    on_connect,
                    on_edge_update_start,
                    on_edge_click,
                    on_edge_double_click,
                    on_edge_mouse_enter,
                    on_edge_mouse_leave,
                }
                div { class: "dioxus-flow__edgelabel-renderer" }

                NodeRenderer::<N, E> {
                    node_types,
                    on_nodes_change,
                    on_edges_change,
                    on_node_drag_start,
                    on_node_click,
                    on_node_double_click,
                    on_node_mouse_enter,
                    on_node_mouse_leave,
                }
                div { class: "dioxus-flow__viewport-portal" }
            }

            if state.user_selection_active.read().clone() {
                if let Some(style) = selection_style {
                    div {
                        class: "dioxus-flow__selection",
                        style: "{style}",
                    }
                }
            }
        }

        SelectionListener::<N, E> { on_selection_change }
    }
}

fn connection_line_element<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &FlowState<N, E>,
) -> Element {
    let connection = state.connection.read().clone();
    if !connection.in_progress {
        return rsx! {};
    }
    let Some(from_node_id) = connection.from_node.clone() else {
        return rsx! {};
    };
    let Some(from_pos) = connection.from_position else {
        return rsx! {};
    };
    let Some(to) = connection.to_position else {
        return rsx! {};
    };
    let Some(node) = state.node_lookup.read().get(&from_node_id).cloned() else {
        return rsx! {};
    };
    let (from_x, from_y) = handle_position_for_connection(
        &node,
        connection.from_type,
        connection.from_handle.as_deref(),
        from_pos,
    );
    let to_handle_type = connection.to_type.or_else(|| {
        connection
            .from_type
            .map(|handle_type| handle_type.opposite())
    });
    let to_position = if let (Some(handle_type), Some(to_node_id)) =
        (to_handle_type, connection.to_node.clone())
    {
        state
            .node_lookup
            .read()
            .get(&to_node_id)
            .and_then(|node| {
                node.handle_bounds.as_ref().and_then(|bounds| {
                    select_handle(bounds, handle_type, connection.to_handle.as_deref())
                        .map(|handle| handle.position)
                })
            })
            .unwrap_or(from_pos)
    } else {
        from_pos
    };
    let is_valid = connection.is_valid;
    let status_class = if is_valid {
        "dioxus-flow__connection valid"
    } else {
        "dioxus-flow__connection invalid"
    };
    let path_style = state
        .connection_line_style
        .read()
        .clone()
        .unwrap_or_default();

    if let Some(component) = state.connection_line_component.read().clone() {
        return component(crate::types::ConnectionLineProps {
            from_x,
            from_y,
            to_x: to.x,
            to_y: to.y,
            from_position: from_pos,
            to_position,
            connection_line_type: *state.connection_line_type.read(),
            from_node_id,
            from_handle_id: connection.from_handle.clone(),
            to_node_id: connection.to_node.clone(),
            to_handle_id: connection.to_handle.clone(),
            is_valid,
        });
    }

    let path = match *state.connection_line_type.read() {
        crate::types::ConnectionLineType::Straight => {
            get_straight_path(from_x, from_y, to.x, to.y).path
        }
        crate::types::ConnectionLineType::Step => {
            get_step_path(from_x, from_y, to.x, to.y, from_pos, to_position, None).path
        }
        crate::types::ConnectionLineType::SmoothStep => {
            get_smooth_step_path(from_x, from_y, to.x, to.y, from_pos, to_position, None, None, None).path
        }
        crate::types::ConnectionLineType::SimpleBezier => {
            get_simple_bezier_path(from_x, from_y, to.x, to.y, from_pos, to_position).path
        }
        crate::types::ConnectionLineType::Bezier => {
            get_bezier_path(from_x, from_y, to.x, to.y, from_pos, to_position, None).path
        }
    };

    rsx! {
        svg {
            class: "{status_class}",
            width: "100%",
            height: "100%",
            path {
                class: "dioxus-flow__connection-path",
                style: "{path_style}",
                d: "{path}",
            }
        }
    }
}

fn node_handle_position_internal<N: Clone + PartialEq + Default>(
    node: &crate::types::InternalNode<N>,
    position: Position,
) -> (f64, f64) {
    let dims = node.dimensions;
    let base = node.position_absolute;
    match position {
        Position::Left => (base.x, base.y + dims.height / 2.0),
        Position::Right => (base.x + dims.width, base.y + dims.height / 2.0),
        Position::Top => (base.x + dims.width / 2.0, base.y),
        Position::Bottom => (base.x + dims.width / 2.0, base.y + dims.height),
    }
}

fn handle_position_for_connection<N: Clone + PartialEq + Default>(
    node: &crate::types::InternalNode<N>,
    handle_type: Option<HandleType>,
    handle_id: Option<&str>,
    fallback_position: Position,
) -> (f64, f64) {
    if let Some(handle_type) = handle_type {
        if let Some(bounds) = &node.handle_bounds {
            if let Some(handle) = select_handle(bounds, handle_type, handle_id) {
                return (
                    node.position_absolute.x + handle.x + handle.width / 2.0,
                    node.position_absolute.y + handle.y + handle.height / 2.0,
                );
            }
        }
    }

    node_handle_position_internal(node, fallback_position)
}

fn select_handle<'a>(
    bounds: &'a HandleBounds,
    handle_type: HandleType,
    handle_id: Option<&str>,
) -> Option<&'a HandleBound> {
    let handles = match handle_type {
        HandleType::Source => &bounds.source,
        HandleType::Target => &bounds.target,
    };
    if let Some(id) = handle_id {
        if let Some(found) = handles
            .iter()
            .find(|handle| handle.id.as_deref() == Some(id))
        {
            return Some(found);
        }
    }
    handles.first()
}

fn compute_handle_bounds_for_zoom(element: &web_sys::Element, zoom: f64) -> Option<HandleBounds> {
    let node_rect = element.get_bounding_client_rect();
    let handles = element.query_selector_all(".dioxus-flow__handle").ok()?;
    let mut bounds = HandleBounds::default();

    for index in 0..handles.length() {
        let handle = handles
            .get(index)
            .and_then(|h| h.dyn_into::<web_sys::Element>().ok())?;
        let rect = handle.get_bounding_client_rect();
        let x = (rect.x() - node_rect.x()) / zoom;
        let y = (rect.y() - node_rect.y()) / zoom;
        let width = rect.width() / zoom;
        let height = rect.height() / zoom;
        let id = handle
            .get_attribute("data-handle-id")
            .filter(|v| !v.is_empty());
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
