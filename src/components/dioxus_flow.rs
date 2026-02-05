//! Main Dioxus Flow component

use crate::components::{FlowProvider, GraphView};
use crate::state::FlowState;
use crate::types::{
    ConnectionLineType, CoordinateExtent, Edge, Node, NodeExtent, PanOnScrollMode, SelectionMode,
    Viewport, XYPosition,
};
use dioxus::prelude::*;
use dioxus::prelude::{ReadableExt, WritableExt};
use std::collections::HashMap;
use js_sys::Function;
use js_sys::Reflect;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;

#[component]
pub fn DioxusFlow<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    children: Element,
    #[props(default)] nodes: Option<Signal<Vec<Node<N>>>>,
    #[props(default)] edges: Option<Signal<Vec<Edge<E>>>>,
    #[props(default)] default_nodes: Vec<Node<N>>,
    #[props(default)] default_edges: Vec<Edge<E>>,
    #[props(default)] node_types: Option<HashMap<String, Component<NodeProps<N, E>>>>,
    #[props(default)] edge_types: Option<HashMap<String, Component<EdgeComponentProps<E>>>>,
    #[props(default)] on_nodes_change: Option<EventHandler<Vec<crate::types::NodeChange<N>>>>,
    #[props(default)] on_edges_change: Option<EventHandler<Vec<crate::types::EdgeChange<E>>>>,
    #[props(default)] on_connect: Option<EventHandler<crate::types::Connection>>,
    #[props(default)] on_edge_update_start: Option<EventHandler<Edge<E>>>,
    #[props(default)] on_edge_update: Option<EventHandler<crate::types::EdgeUpdateEvent<E>>>,
    #[props(default)] on_edge_update_end: Option<EventHandler<crate::types::EdgeUpdateEndEvent<E>>>,
    #[props(default)] on_selection_change: Option<
        EventHandler<crate::types::SelectionChange<N, E>>,
    >,
    #[props(default)] on_node_drag_start: Option<EventHandler<crate::types::NodeDragEvent<N>>>,
    #[props(default)] on_node_drag: Option<EventHandler<crate::types::NodeDragEvent<N>>>,
    #[props(default)] on_node_drag_stop: Option<EventHandler<crate::types::NodeDragEvent<N>>>,
    #[props(default)] on_init: Option<EventHandler<crate::hooks::FlowInstance<N, E>>>,
    #[props(default)] on_move: Option<EventHandler<Viewport>>,
    #[props(default)] on_move_start: Option<EventHandler<Viewport>>,
    #[props(default)] on_move_end: Option<EventHandler<Viewport>>,
    #[props(default = 0.5)] min_zoom: f64,
    #[props(default = 2.0)] max_zoom: f64,
    #[props(default)] default_viewport: Option<Viewport>,
    #[props(default)] viewport: Option<Signal<Viewport>>,
    #[props(default)] on_viewport_change: Option<EventHandler<Viewport>>,
    #[props(default)] translate_extent: Option<CoordinateExtent>,
    #[props(default = (0.0, 0.0))] node_origin: crate::types::NodeOrigin,
    #[props(default = false)] fit_view: bool,
    #[props(default)] fit_view_options: Option<crate::types::FitViewOptions>,
    #[props(default = true)] zoom_on_scroll: bool,
    #[props(default = true)] zoom_on_pinch: bool,
    #[props(default = true)] zoom_on_double_click: bool,
    #[props(default = true)] pan_on_drag: bool,
    #[props(default)] pan_on_drag_buttons: Option<Vec<i32>>,
    #[props(default)] pan_on_scroll: bool,
    #[props(default = 0.5)] pan_on_scroll_speed: f64,
    #[props(default = PanOnScrollMode::Free)] pan_on_scroll_mode: PanOnScrollMode,
    #[props(default = true)] prevent_scrolling: bool,
    #[props(default)] pan_activation_key_code: Option<Vec<String>>,
    #[props(default)] zoom_activation_key_code: Option<Vec<String>>,
    #[props(default = true)] auto_pan_on_node_drag: bool,
    #[props(default = true)] auto_pan_on_connect: bool,
    #[props(default = 15.0)] auto_pan_speed: f64,
    #[props(default = true)] auto_pan_on_node_focus: bool,
    #[props(default = true)] nodes_draggable: bool,
    #[props(default = false)] snap_to_grid: bool,
    #[props(default = (15.0, 15.0))] snap_grid: (f64, f64),
    #[props(default = true)] nodes_connectable: bool,
    #[props(default = true)] nodes_focusable: bool,
    #[props(default = true)] edges_focusable: bool,
    #[props(default = true)] edges_reconnectable: bool,
    #[props(default = true)] elements_selectable: bool,
    #[props(default = true)] select_nodes_on_drag: bool,
    #[props(default = false)] only_render_visible_elements: bool,
    #[props(default = 0.2)] visible_area_padding: f64,
    #[props(default = false)] selection_on_drag: bool,
    #[props(default = SelectionMode::Partial)] selection_mode: SelectionMode,
    #[props(default)] node_extent: Option<crate::types::CoordinateExtent>,
    #[props(default)] connection_mode: Option<crate::types::ConnectionMode>,
    #[props(default)] connection_line_type: Option<ConnectionLineType>,
    #[props(default)] connection_line_component: Option<
        Component<crate::types::ConnectionLineProps>,
    >,
    #[props(default)] connection_line_style: Option<String>,
    #[props(default)] is_valid_connection: Option<crate::types::IsValidConnection>,
    #[props(default = 20.0)] connection_radius: f64,
    #[props(default = 10.0)] reconnect_radius: f64,
    #[props(default = 1.0)] node_drag_threshold: f64,
    #[props(default = 1.0)] connection_drag_threshold: f64,
    #[props(default = true)] connect_on_click: bool,
    #[props(default)] default_marker_color: Option<String>,
    #[props(default = "nodrag".to_string())] no_drag_class_name: String,
    #[props(default = "nowheel".to_string())] no_wheel_class_name: String,
    #[props(default = "nopan".to_string())] no_pan_class_name: String,
    #[props(default)] delete_key_code: Option<Vec<String>>,
    #[props(default)] selection_key_code: Option<Vec<String>>,
    #[props(default)] multi_selection_key_code: Option<Vec<String>>,
    #[props(default = true)] elevate_nodes_on_select: bool,
    #[props(default = false)] elevate_edges_on_select: bool,
    #[props(default = crate::types::ZIndexMode::Basic)] z_index_mode: crate::types::ZIndexMode,
    #[props(default = false)] disable_keyboard_a11y: bool,
    #[props(default)] width: Option<f64>,
    #[props(default)] height: Option<f64>,
    #[props(default = crate::types::ColorMode::Light)] color_mode: crate::types::ColorMode,
    #[props(default = false)] debug: bool,
    #[props(default)] aria_label_config: Option<crate::types::AriaLabelConfig>,
    #[props(default)] attribution_position: Option<String>,
    #[props(default)] pro_options: Option<crate::types::ProOptions>,
    #[props(default)] on_connect_start: Option<EventHandler<crate::types::ConnectionStartEvent>>,
    #[props(default)] on_connect_end: Option<EventHandler<crate::types::ConnectionEndEvent>>,
    #[props(default)] on_selection_start: Option<EventHandler<crate::types::SelectionStartEvent>>,
    #[props(default)] on_selection_end: Option<
        EventHandler<crate::types::SelectionEndEvent<N, E>>,
    >,
    #[props(default)] on_nodes_delete: Option<EventHandler<Vec<Node<N>>>>,
    #[props(default)] on_edges_delete: Option<EventHandler<Vec<Edge<E>>>>,
    #[props(default)] on_before_delete: Option<crate::types::OnBeforeDelete<N, E>>,
    #[props(default)] on_node_click: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_node_double_click: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_node_mouse_enter: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_node_mouse_leave: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_edge_click: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
    #[props(default)] on_edge_double_click: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
    #[props(default)] on_edge_mouse_enter: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
    #[props(default)] on_edge_mouse_leave: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
    #[props(default)] on_error: Option<crate::types::OnError>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
) -> Element {
    rsx! {
        FlowProvider {
            initial_nodes: default_nodes.clone(),
            initial_edges: default_edges.clone(),

            FlowBody {
                nodes,
                edges,
                node_types,
                edge_types,
                on_nodes_change,
                on_edges_change,
                on_connect,
                on_edge_update_start,
                on_edge_update,
                on_edge_update_end,
                on_selection_change,
                on_node_drag_start,
                on_node_drag,
                on_node_drag_stop,
                on_init,
                on_move,
                on_move_start,
                on_move_end,
                min_zoom,
                max_zoom,
                default_viewport,
                viewport,
                on_viewport_change,
                translate_extent,
                node_origin,
                fit_view,
                fit_view_options,
                zoom_on_scroll,
                zoom_on_pinch,
                zoom_on_double_click,
                pan_on_drag,
                pan_on_drag_buttons,
                pan_on_scroll,
                pan_on_scroll_speed,
                pan_on_scroll_mode,
                prevent_scrolling,
                pan_activation_key_code,
                zoom_activation_key_code,
                auto_pan_on_node_drag,
                auto_pan_on_connect,
                auto_pan_speed,
                auto_pan_on_node_focus,
                nodes_draggable,
                snap_to_grid,
                snap_grid,
                nodes_connectable,
                nodes_focusable,
                edges_focusable,
                edges_reconnectable,
                elements_selectable,
                select_nodes_on_drag,
                only_render_visible_elements,
                visible_area_padding,
                selection_on_drag,
                selection_mode,
                node_extent,
                connection_mode,
                connection_line_type,
                connection_line_component,
                connection_line_style,
                is_valid_connection,
                connection_radius,
                reconnect_radius,
                node_drag_threshold,
                connection_drag_threshold,
                connect_on_click,
                default_marker_color,
                no_drag_class_name,
                no_wheel_class_name,
                no_pan_class_name,
                delete_key_code,
                selection_key_code,
                multi_selection_key_code,
                elevate_nodes_on_select,
                elevate_edges_on_select,
                z_index_mode,
                disable_keyboard_a11y,
                width,
                height,
                color_mode,
                debug,
                aria_label_config,
                attribution_position,
                pro_options,
                on_connect_start,
                on_connect_end,
                on_selection_start,
                on_selection_end,
                on_nodes_delete,
                on_edges_delete,
                on_before_delete,
                on_node_click,
                on_node_double_click,
                on_node_mouse_enter,
                on_node_mouse_leave,
                on_edge_click,
                on_edge_double_click,
                on_edge_mouse_enter,
                on_edge_mouse_leave,
                on_error,
                class,
                style,
                children,
            }
        }
    }
}

#[component]
fn FlowBody<N: Clone + PartialEq + Default + 'static, E: Clone + PartialEq + Default + 'static>(
    children: Element,
    #[props(default)] nodes: Option<Signal<Vec<Node<N>>>>,
    #[props(default)] edges: Option<Signal<Vec<Edge<E>>>>,
    #[props(default)] node_types: Option<HashMap<String, Component<NodeProps<N, E>>>>,
    #[props(default)] edge_types: Option<HashMap<String, Component<EdgeComponentProps<E>>>>,
    #[props(default)] on_nodes_change: Option<EventHandler<Vec<crate::types::NodeChange<N>>>>,
    #[props(default)] on_edges_change: Option<EventHandler<Vec<crate::types::EdgeChange<E>>>>,
    #[props(default)] on_connect: Option<EventHandler<crate::types::Connection>>,
    #[props(default)] on_edge_update_start: Option<EventHandler<Edge<E>>>,
    #[props(default)] on_edge_update: Option<EventHandler<crate::types::EdgeUpdateEvent<E>>>,
    #[props(default)] on_edge_update_end: Option<EventHandler<crate::types::EdgeUpdateEndEvent<E>>>,
    #[props(default)] on_selection_change: Option<
        EventHandler<crate::types::SelectionChange<N, E>>,
    >,
    #[props(default)] on_node_drag_start: Option<EventHandler<crate::types::NodeDragEvent<N>>>,
    #[props(default)] on_node_drag: Option<EventHandler<crate::types::NodeDragEvent<N>>>,
    #[props(default)] on_node_drag_stop: Option<EventHandler<crate::types::NodeDragEvent<N>>>,
    #[props(default)] on_init: Option<EventHandler<crate::hooks::FlowInstance<N, E>>>,
    #[props(default)] on_move: Option<EventHandler<Viewport>>,
    #[props(default)] on_move_start: Option<EventHandler<Viewport>>,
    #[props(default)] on_move_end: Option<EventHandler<Viewport>>,
    #[props(default = 0.5)] min_zoom: f64,
    #[props(default = 2.0)] max_zoom: f64,
    #[props(default)] default_viewport: Option<Viewport>,
    #[props(default)] viewport: Option<Signal<Viewport>>,
    #[props(default)] on_viewport_change: Option<EventHandler<Viewport>>,
    #[props(default)] translate_extent: Option<CoordinateExtent>,
    #[props(default = (0.0, 0.0))] node_origin: crate::types::NodeOrigin,
    #[props(default = false)] fit_view: bool,
    #[props(default)] fit_view_options: Option<crate::types::FitViewOptions>,
    #[props(default = true)] zoom_on_scroll: bool,
    #[props(default = true)] zoom_on_pinch: bool,
    #[props(default = true)] zoom_on_double_click: bool,
    #[props(default = true)] pan_on_drag: bool,
    #[props(default)] pan_on_drag_buttons: Option<Vec<i32>>,
    #[props(default)] pan_on_scroll: bool,
    #[props(default = 0.5)] pan_on_scroll_speed: f64,
    #[props(default = PanOnScrollMode::Free)] pan_on_scroll_mode: PanOnScrollMode,
    #[props(default = true)] prevent_scrolling: bool,
    #[props(default)] pan_activation_key_code: Option<Vec<String>>,
    #[props(default)] zoom_activation_key_code: Option<Vec<String>>,
    #[props(default = true)] auto_pan_on_node_drag: bool,
    #[props(default = true)] auto_pan_on_connect: bool,
    #[props(default = 15.0)] auto_pan_speed: f64,
    #[props(default = true)] auto_pan_on_node_focus: bool,
    #[props(default = true)] nodes_draggable: bool,
    #[props(default = false)] snap_to_grid: bool,
    #[props(default = (15.0, 15.0))] snap_grid: (f64, f64),
    #[props(default = true)] nodes_connectable: bool,
    #[props(default = true)] nodes_focusable: bool,
    #[props(default = true)] edges_focusable: bool,
    #[props(default = true)] edges_reconnectable: bool,
    #[props(default = true)] elements_selectable: bool,
    #[props(default = true)] select_nodes_on_drag: bool,
    #[props(default = false)] only_render_visible_elements: bool,
    #[props(default = 0.2)] visible_area_padding: f64,
    #[props(default = false)] selection_on_drag: bool,
    #[props(default = SelectionMode::Partial)] selection_mode: SelectionMode,
    #[props(default)] node_extent: Option<crate::types::CoordinateExtent>,
    #[props(default)] connection_mode: Option<crate::types::ConnectionMode>,
    #[props(default)] connection_line_type: Option<ConnectionLineType>,
    #[props(default)] connection_line_component: Option<
        Component<crate::types::ConnectionLineProps>,
    >,
    #[props(default)] connection_line_style: Option<String>,
    #[props(default)] is_valid_connection: Option<crate::types::IsValidConnection>,
    #[props(default = 20.0)] connection_radius: f64,
    #[props(default = 10.0)] reconnect_radius: f64,
    #[props(default = 1.0)] node_drag_threshold: f64,
    #[props(default = 1.0)] connection_drag_threshold: f64,
    #[props(default = true)] connect_on_click: bool,
    #[props(default)] default_marker_color: Option<String>,
    #[props(default = "nodrag".to_string())] no_drag_class_name: String,
    #[props(default = "nowheel".to_string())] no_wheel_class_name: String,
    #[props(default = "nopan".to_string())] no_pan_class_name: String,
    #[props(default)] delete_key_code: Option<Vec<String>>,
    #[props(default)] selection_key_code: Option<Vec<String>>,
    #[props(default)] multi_selection_key_code: Option<Vec<String>>,
    #[props(default = true)] elevate_nodes_on_select: bool,
    #[props(default = false)] elevate_edges_on_select: bool,
    #[props(default = crate::types::ZIndexMode::Basic)] z_index_mode: crate::types::ZIndexMode,
    #[props(default = false)] disable_keyboard_a11y: bool,
    #[props(default)] width: Option<f64>,
    #[props(default)] height: Option<f64>,
    #[props(default = crate::types::ColorMode::Light)] color_mode: crate::types::ColorMode,
    #[props(default = false)] debug: bool,
    #[props(default)] aria_label_config: Option<crate::types::AriaLabelConfig>,
    #[props(default)] attribution_position: Option<String>,
    #[props(default)] pro_options: Option<crate::types::ProOptions>,
    #[props(default)] on_connect_start: Option<EventHandler<crate::types::ConnectionStartEvent>>,
    #[props(default)] on_connect_end: Option<EventHandler<crate::types::ConnectionEndEvent>>,
    #[props(default)] on_selection_start: Option<EventHandler<crate::types::SelectionStartEvent>>,
    #[props(default)] on_selection_end: Option<
        EventHandler<crate::types::SelectionEndEvent<N, E>>,
    >,
    #[props(default)] on_nodes_delete: Option<EventHandler<Vec<Node<N>>>>,
    #[props(default)] on_edges_delete: Option<EventHandler<Vec<Edge<E>>>>,
    #[props(default)] on_before_delete: Option<crate::types::OnBeforeDelete<N, E>>,
    #[props(default)] on_node_click: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_node_double_click: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_node_mouse_enter: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_node_mouse_leave: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_edge_click: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
    #[props(default)] on_edge_double_click: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
    #[props(default)] on_edge_mouse_enter: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
    #[props(default)] on_edge_mouse_leave: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
    #[props(default)] on_error: Option<crate::types::OnError>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();

    let mut state_config = state.clone();
    let aria_label_config_state = aria_label_config.clone();
    use_effect(move || {
        state_config.min_zoom.set(min_zoom);
        state_config.max_zoom.set(max_zoom);
        state_config.translate_extent.set(translate_extent);
        state_config.node_origin.set(node_origin);
        state_config.color_mode.set(color_mode);
        state_config.default_marker_color.set(default_marker_color.clone());
        state_config.z_index_mode.set(z_index_mode);
        state_config.elevate_nodes_on_select.set(elevate_nodes_on_select);
        state_config.elevate_edges_on_select.set(elevate_edges_on_select);
        state_config.disable_keyboard_a11y.set(disable_keyboard_a11y);
        state_config.debug.set(debug);
        if let Some(config) = aria_label_config_state.clone() {
            state_config.aria_label_config.set(config);
        }
        state_config.zoom_on_scroll.set(zoom_on_scroll);
        state_config.zoom_on_pinch.set(zoom_on_pinch);
        state_config.zoom_on_double_click.set(zoom_on_double_click);
        state_config.pan_on_drag.set(pan_on_drag);
        state_config.pan_on_drag_buttons.set(pan_on_drag_buttons.clone());
        state_config.pan_on_scroll.set(pan_on_scroll);
        state_config.pan_on_scroll_speed.set(pan_on_scroll_speed);
        state_config.pan_on_scroll_mode.set(pan_on_scroll_mode);
        state_config.prevent_scrolling.set(prevent_scrolling);
        state_config.auto_pan_on_node_drag.set(auto_pan_on_node_drag);
        state_config.auto_pan_on_connect.set(auto_pan_on_connect);
        state_config.auto_pan_speed.set(auto_pan_speed);
        state_config.auto_pan_on_node_focus.set(auto_pan_on_node_focus);
        state_config.nodes_draggable.set(nodes_draggable);
        state_config.snap_to_grid.set(snap_to_grid);
        state_config.snap_grid.set(snap_grid);
        state_config.nodes_connectable.set(nodes_connectable);
        state_config.nodes_focusable.set(nodes_focusable);
        state_config.edges_focusable.set(edges_focusable);
        state_config.edges_reconnectable.set(edges_reconnectable);
        state_config.elements_selectable.set(elements_selectable);
        state_config.select_nodes_on_drag.set(select_nodes_on_drag);
        state_config
            .only_render_visible_elements
            .set(only_render_visible_elements);
        state_config.visible_area_padding.set(visible_area_padding);
        state_config.selection_on_drag.set(selection_on_drag);
        state_config.selection_mode.set(selection_mode);
        state_config.node_extent.set(node_extent);
        if let Some(mode) = connection_mode {
            state_config.connection_mode.set(mode);
        }
        if let Some(line_type) = connection_line_type {
            state_config.connection_line_type.set(line_type);
        }
        state_config
            .connection_line_component
            .set(connection_line_component);
        state_config
            .connection_line_style
            .set(connection_line_style.clone());
        state_config.is_valid_connection.set(is_valid_connection);
        state_config.connection_radius.set(connection_radius);
        state_config.reconnect_radius.set(reconnect_radius);
        state_config.node_drag_threshold.set(node_drag_threshold);
        state_config
            .connection_drag_threshold
            .set(connection_drag_threshold);
        state_config.connect_on_click.set(connect_on_click);
        state_config.no_drag_class_name.set(no_drag_class_name.clone());
        state_config
            .no_wheel_class_name
            .set(no_wheel_class_name.clone());
        state_config.no_pan_class_name.set(no_pan_class_name.clone());
        state_config.on_connect_start.set(on_connect_start.clone());
        state_config.on_connect_end.set(on_connect_end.clone());
        state_config.on_error.set(on_error);
        state_config.on_viewport_change.set(on_viewport_change.clone());

        if let Some(width) = width {
            state_config.width.set(width);
        }
        if let Some(height) = height {
            state_config.height.set(height);
        }

        if let Some(viewport) = default_viewport {
            state_config.viewport.set(viewport);
        }
    });

    let mut state_sync = state.clone();
    use_effect(move || {
        if let Some(nodes_signal) = &nodes {
            let next_nodes = nodes_signal.read().clone();
            if *state_sync.nodes.read() != next_nodes {
                state_sync.set_nodes(next_nodes);
            }
        }
        if let Some(edges_signal) = &edges {
            let next_edges = edges_signal.read().clone();
            if *state_sync.edges.read() != next_edges {
                state_sync.set_edges(next_edges);
            }
        }
        if let Some(viewport_signal) = &viewport {
            let next_viewport = *viewport_signal.read();
            if *state_sync.viewport.read() != next_viewport {
                state_sync.viewport.set(next_viewport);
                state_sync.refresh_connection_position();
            }
        }
    });

    let mut init_done = use_signal(|| false);
    let state_init = state.clone();
    use_effect(move || {
        if !*init_done.read() {
            if let Some(on_init) = &on_init {
                on_init.call(crate::hooks::FlowInstance::new(state_init.clone()));
            }
            init_done.set(true);
        }
    });

    let mut fit_done = use_signal(|| false);
    let mut state_fit = state.clone();
    let viewport_controlled = viewport.is_some();
    use_effect(move || {
        if viewport_controlled || !fit_view || *fit_done.read() {
            return;
        }
        if state_fit.nodes.read().is_empty() {
            return;
        }
        if *state_fit.width.read() <= 0.0 || *state_fit.height.read() <= 0.0 {
            return;
        }
        state_fit.fit_view(fit_view_options.clone());
        fit_done.set(true);
    });

    let class = class.unwrap_or_default();
    let mut style = style.unwrap_or_default();
    if let Some(width) = width {
        style.push_str(&format!(" width: {}px;", width));
    }
    if let Some(height) = height {
        style.push_str(&format!(" height: {}px;", height));
    }

    let mut is_dark_mode = use_signal(|| matches!(color_mode, crate::types::ColorMode::Dark));
    use_effect(move || {
        match color_mode {
            crate::types::ColorMode::Dark => is_dark_mode.set(true),
            crate::types::ColorMode::Light => is_dark_mode.set(false),
            crate::types::ColorMode::System => {
                if let Some(window) = web_sys::window() {
                    let func = js_sys::Reflect::get(&window, &JsValue::from_str("matchMedia"))
                        .ok()
                        .and_then(|value| value.dyn_into::<js_sys::Function>().ok());
                    if let Some(func) = func {
                        if let Ok(result) =
                            func.call1(&window, &JsValue::from_str("(prefers-color-scheme: dark)"))
                        {
                            let matches = js_sys::Reflect::get(
                                &result,
                                &JsValue::from_str("matches"),
                            )
                            .ok()
                            .and_then(|value| value.as_bool())
                            .unwrap_or(false);
                            is_dark_mode.set(matches);
                        }
                    }
                }
            }
        }
    });

    let connection_active = state.connection.read().in_progress;
    let mut flow_class = if connection_active {
        format!("dioxus-flow dioxus-flow--connecting {class}")
    } else {
        format!("dioxus-flow {class}")
    };
    if *is_dark_mode.read() {
        flow_class.push_str(" dioxus-flow--dark");
    }

    let delete_keys = if disable_keyboard_a11y {
        Vec::new()
    } else {
        delete_key_code
            .unwrap_or_else(|| vec!["Backspace".to_string(), "Delete".to_string()])
    };
    let delete_pressed = crate::hooks::use_key_press_multi(delete_keys);
    let mut delete_latched = use_signal(|| false);
    let mut state_delete = state.clone();
    let on_nodes_change_delete = on_nodes_change.clone();
    let on_edges_change_delete = on_edges_change.clone();
    use_effect(move || {
        if disable_keyboard_a11y {
            return;
        }
        let pressed = *delete_pressed.read();
        if pressed && !*delete_latched.read() {
            let selected_nodes: Vec<Node<N>> = state_delete
                .nodes
                .read()
                .iter()
                .filter(|n| n.selected && n.deletable.unwrap_or(true))
                .cloned()
                .collect();
            let selected_node_ids: std::collections::HashSet<&str> =
                selected_nodes.iter().map(|n| n.id.as_str()).collect();
            let mut selected_edge_ids: std::collections::HashSet<String> = state_delete
                .edges
                .read()
                .iter()
                .filter(|e| e.selected && e.deletable.unwrap_or(true))
                .map(|e| e.id.clone())
                .collect();
            let selected_edges: Vec<Edge<E>> = {
                let edges = state_delete.edges.read();
                for edge in edges.iter() {
                    if selected_node_ids.contains(edge.source.as_str())
                        || selected_node_ids.contains(edge.target.as_str())
                    {
                        selected_edge_ids.insert(edge.id.clone());
                    }
                }
                edges
                    .iter()
                    .filter(|e| selected_edge_ids.contains(&e.id))
                    .cloned()
                    .collect()
            };

            if let Some(check) = on_before_delete {
                let event = crate::types::BeforeDeleteEvent {
                    nodes: selected_nodes.clone(),
                    edges: selected_edges.clone(),
                };
                if !check(&event) {
                    delete_latched.set(true);
                    return;
                }
            }

            if let Some(handler) = &on_nodes_delete {
                handler.call(selected_nodes.clone());
            }
            if let Some(handler) = &on_edges_delete {
                handler.call(selected_edges.clone());
            }

            let node_changes: Vec<crate::types::NodeChange<N>> = selected_nodes
                .iter()
                .map(|n| crate::types::NodeChange::remove(n.id.clone()))
                .collect();
            let edge_changes: Vec<crate::types::EdgeChange<E>> = selected_edges
                .iter()
                .map(|e| crate::types::EdgeChange::remove(e.id.clone()))
                .collect();

            if let Some(handler) = &on_nodes_change_delete {
                handler.call(node_changes);
            } else {
                state_delete.apply_node_changes(node_changes);
            }
            if let Some(handler) = &on_edges_change_delete {
                handler.call(edge_changes);
            } else {
                state_delete.apply_edge_changes(edge_changes);
            }
            delete_latched.set(true);
        } else if !pressed && *delete_latched.read() {
            delete_latched.set(false);
        }
    });

    let selection_keys = if disable_keyboard_a11y {
        Vec::new()
    } else {
        selection_key_code.unwrap_or_else(|| vec!["Shift".to_string()])
    };
    let selection_pressed = crate::hooks::use_key_press_multi(selection_keys);
    let mut state_selection = state.clone();
    use_effect(move || {
        if disable_keyboard_a11y {
            state_selection.selection_key_pressed.set(false);
            return;
        }
        state_selection
            .selection_key_pressed
            .set(*selection_pressed.read());
    });

    let multi_keys = if disable_keyboard_a11y {
        Vec::new()
    } else {
        multi_selection_key_code
            .unwrap_or_else(|| vec!["Meta".to_string(), "Control".to_string()])
    };
    let multi_pressed = crate::hooks::use_key_press_multi(multi_keys);
    let mut state_multi = state.clone();
    use_effect(move || {
        if disable_keyboard_a11y {
            state_multi.multi_selection_key_pressed.set(false);
            return;
        }
        state_multi
            .multi_selection_key_pressed
            .set(*multi_pressed.read());
    });

    let pan_keys = pan_activation_key_code.unwrap_or_else(|| vec![" ".to_string(), "Space".to_string()]);
    let pan_pressed = crate::hooks::use_key_press_multi(pan_keys);
    let mut state_pan_key = state.clone();
    use_effect(move || {
        state_pan_key
            .pan_activation_key_pressed
            .set(*pan_pressed.read());
    });

    let zoom_keys = zoom_activation_key_code
        .unwrap_or_else(|| vec!["Meta".to_string(), "Control".to_string()]);
    let zoom_pressed = crate::hooks::use_key_press_multi(zoom_keys);
    let mut state_zoom_key = state.clone();
    use_effect(move || {
        state_zoom_key
            .zoom_activation_key_pressed
            .set(*zoom_pressed.read());
    });

    let state_keyboard = state.clone();
    let on_nodes_change_keyboard = on_nodes_change.clone();
    let on_edges_change_keyboard = on_edges_change.clone();
    let _keyboard_listener = use_hook(move || {
        let mut state_keyboard_event = state_keyboard.clone();
        let on_nodes_change_keyboard = on_nodes_change_keyboard.clone();
        let on_edges_change_keyboard = on_edges_change_keyboard.clone();
        Rc::new(WindowListener::new(
            "keydown",
            move |evt: web_sys::KeyboardEvent| {
                if disable_keyboard_a11y {
                    return;
                }
                if evt.default_prevented() {
                    return;
                }
                if let Some(target) = evt
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                {
                    let tag = target.tag_name().to_uppercase();
                    if tag == "INPUT"
                        || tag == "TEXTAREA"
                        || tag == "SELECT"
                        || target.has_attribute("contenteditable")
                    {
                        return;
                    }
                }

                let key = evt.key();
                let step = if evt.shift_key() { 10.0 } else { 1.0 };
                let mut dx = 0.0;
                let mut dy = 0.0;
                match key.as_str() {
                    "ArrowUp" => dy = -step,
                    "ArrowDown" => dy = step,
                    "ArrowLeft" => dx = -step,
                    "ArrowRight" => dx = step,
                    "Tab" => {
                        evt.prevent_default();
                        focus_next_element(&mut state_keyboard_event, evt.shift_key());
                        return;
                    }
                    " " | "Enter" => {
                        toggle_focused_selection(
                            &mut state_keyboard_event,
                            &on_nodes_change_keyboard,
                            &on_edges_change_keyboard,
                        );
                        evt.prevent_default();
                        return;
                    }
                    "a" | "A" => {
                        if evt.meta_key() || evt.ctrl_key() {
                            state_keyboard_event.select_all();
                            evt.prevent_default();
                        }
                        return;
                    }
                    _ => return,
                }

                if dx == 0.0 && dy == 0.0 {
                    return;
                }
                if !*state_keyboard_event.nodes_draggable.read() {
                    return;
                }

                let selected = state_keyboard_event.get_selected_nodes();
                if selected.is_empty() {
                    return;
                }

                let snap = *state_keyboard_event.snap_to_grid.read();
                let grid = *state_keyboard_event.snap_grid.read();
                let mut changes = Vec::new();
                for node in selected.iter() {
                    let mut next = XYPosition {
                        x: node.position.x + dx,
                        y: node.position.y + dy,
                    };
                    if snap {
                        next.x = (next.x / grid.0).round() * grid.0;
                        next.y = (next.y / grid.1).round() * grid.1;
                    }
                    next = clamp_keyboard_position(&state_keyboard_event, node, next);
                    changes.push(crate::types::NodeChange::Position {
                        id: node.id.clone(),
                        position: Some(next),
                        dragging: false,
                    });
                }

                if let Some(handler) = &on_nodes_change_keyboard {
                    handler.call(changes);
                } else {
                    state_keyboard_event.apply_node_changes(changes);
                }
                evt.prevent_default();
            },
        ))
    });

    let show_attribution =
        !pro_options.as_ref().map(|p| p.hide_attribution).unwrap_or(false);
    let attribution_label = aria_label_config
        .as_ref()
        .and_then(|config| config.attribution.clone());

    rsx! {
        div {
            class: "{flow_class}",
            style: "{style}",

            GraphView {
                node_types,
                edge_types,
                on_nodes_change,
                on_edges_change,
                on_connect,
                on_edge_update_start,
                on_edge_update,
                on_edge_update_end,
                on_selection_change,
                on_node_drag_start,
                on_node_drag,
                on_node_drag_stop,
                on_move,
                on_move_start,
                on_move_end,
                on_selection_start,
                on_selection_end,
                on_node_click,
                on_node_double_click,
                on_node_mouse_enter,
                on_node_mouse_leave,
                on_edge_click,
                on_edge_double_click,
                on_edge_mouse_enter,
                on_edge_mouse_leave,
            }

            if show_attribution {
                crate::components::Attribution {
                    position: attribution_position,
                    aria_label: attribution_label,
                }
            }

            {children}
        }
    }
}

struct WindowListener {
    event_type: String,
    closure: Option<Closure<dyn FnMut(web_sys::KeyboardEvent)>>,
}

impl WindowListener {
    fn new(event_type: &str, handler: impl FnMut(web_sys::KeyboardEvent) + 'static) -> Self {
        let Some(window) = web_sys::window() else {
            return Self {
                event_type: event_type.to_string(),
                closure: None,
            };
        };
        let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
        window
            .add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())
            .ok();
        Self {
            event_type: event_type.to_string(),
            closure: Some(closure),
        }
    }
}

impl Drop for WindowListener {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            if let Some(closure) = &self.closure {
                window
                    .remove_event_listener_with_callback(
                        &self.event_type,
                        closure.as_ref().unchecked_ref(),
                    )
                    .ok();
            }
        }
    }
}

fn clamp_keyboard_position<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &FlowState<N, E>,
    node: &Node<N>,
    next_position: XYPosition,
) -> XYPosition {
    let dims = node.get_dimensions();
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
                    let max_x = (parent.dimensions.width - dims.width).max(0.0);
                    let max_y = (parent.dimensions.height - dims.height).max(0.0);
                    return XYPosition {
                        x: next_position.x.clamp(0.0, max_x),
                        y: next_position.y.clamp(0.0, max_y),
                    };
                }
            }
            next_position
        }
        Some(NodeExtent::CoordinateExtent(extent)) => {
            let parent_abs = if let Some(parent_id) = node.parent_id.as_ref() {
                state
                    .node_lookup
                    .read()
                    .get(parent_id)
                    .map(|p| p.position_absolute)
                    .unwrap_or_else(|| XYPosition::new(0.0, 0.0))
            } else {
                XYPosition::new(0.0, 0.0)
            };
            let abs = XYPosition {
                x: next_position.x + parent_abs.x,
                y: next_position.y + parent_abs.y,
            };
            let clamped_abs = clamp_to_extent(extent, abs, dims);
            XYPosition {
                x: clamped_abs.x - parent_abs.x,
                y: clamped_abs.y - parent_abs.y,
            }
        }
        None => next_position,
    }
}

fn focus_next_element<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &mut FlowState<N, E>,
    reverse: bool,
) {
    let nodes_enabled = *state.nodes_focusable.read();
    let edges_enabled = *state.edges_focusable.read();
    if !nodes_enabled && !edges_enabled {
        return;
    }
    let mut focusable: Vec<(bool, String)> = Vec::new();
    if nodes_enabled {
        let nodes = state.nodes.read();
        for node in nodes.iter() {
            if !node.hidden && node.focusable.unwrap_or(true) {
                focusable.push((true, node.id.clone()));
            }
        }
    }
    if edges_enabled {
        let edges = state.edges.read();
        for edge in edges.iter() {
            if edge.focusable.unwrap_or(true) {
                focusable.push((false, edge.id.clone()));
            }
        }
    }
    if focusable.is_empty() {
        return;
    }
    let current_node = state.focused_node_id.read().clone();
    let current_edge = state.focused_edge_id.read().clone();
    let next_index = current_node
        .as_ref()
        .and_then(|id| focusable.iter().position(|(is_node, v)| *is_node && v == id))
        .or_else(|| {
            current_edge
                .as_ref()
                .and_then(|id| focusable.iter().position(|(is_node, v)| !*is_node && v == id))
        })
        .map(|index| {
            if reverse {
                if index == 0 {
                    focusable.len() - 1
                } else {
                    index - 1
                }
            } else {
                (index + 1) % focusable.len()
            }
        })
        .unwrap_or(0);
    let (is_node, next_id) = focusable[next_index].clone();
    if is_node {
        state.focused_node_id.set(Some(next_id.clone()));
        state.focused_edge_id.set(None);
        if *state.auto_pan_on_node_focus.read() {
            state.ensure_node_visible(&next_id);
        }
    } else {
        state.focused_edge_id.set(Some(next_id.clone()));
        state.focused_node_id.set(None);
    }

    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            let selector = if is_node {
                format!("[data-id=\"{}\"]", next_id.replace('\"', "\\\""))
            } else {
                format!("[data-edge-id=\"{}\"]", next_id.replace('\"', "\\\""))
            };
            if let Ok(Some(element)) = document.query_selector(&selector) {
                focus_dom_element(&element);
            }
        }
    }
}

fn toggle_focused_selection<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &mut FlowState<N, E>,
    on_nodes_change: &Option<EventHandler<Vec<crate::types::NodeChange<N>>>>,
    on_edges_change: &Option<EventHandler<Vec<crate::types::EdgeChange<E>>>>,
) {
    let multi = *state.multi_selection_key_pressed.read();
    let focused_node_id = state.focused_node_id.read().clone();
    if let Some(focused_id) = focused_node_id {
        let nodes = state.nodes.read().clone();
        let mut changes = Vec::new();
        for node in nodes.iter() {
            let should_select = if node.id == focused_id {
                if multi { !node.selected } else { true }
            } else if multi {
                node.selected
            } else {
                false
            };
            if node.selected != should_select {
                changes.push(crate::types::NodeChange::Selection {
                    id: node.id.clone(),
                    selected: should_select,
                });
            }
        }
        if !multi {
            let edges = state.edges.read().clone();
            let mut edge_changes = Vec::new();
            for edge in edges.iter() {
                if edge.selected {
                    edge_changes.push(crate::types::EdgeChange::Selection {
                        id: edge.id.clone(),
                        selected: false,
                    });
                }
            }
            if let Some(handler) = on_edges_change {
                handler.call(edge_changes);
            } else {
                state.apply_edge_changes(edge_changes);
            }
        }

        if let Some(handler) = on_nodes_change {
            handler.call(changes);
        } else {
            state.apply_node_changes(changes);
        }
        return;
    }

    let focused_edge_id = state.focused_edge_id.read().clone();
    let Some(focused_edge_id) = focused_edge_id else {
        return;
    };

    let edges = state.edges.read().clone();
    let mut edge_changes = Vec::new();
    for edge in edges.iter() {
        let should_select = if edge.id == focused_edge_id {
            if multi { !edge.selected } else { true }
        } else if multi {
            edge.selected
        } else {
            false
        };
        if edge.selected != should_select {
            edge_changes.push(crate::types::EdgeChange::Selection {
                id: edge.id.clone(),
                selected: should_select,
            });
        }
    }

    if !multi {
        let nodes = state.nodes.read().clone();
        let mut node_changes = Vec::new();
        for node in nodes.iter() {
            if node.selected {
                node_changes.push(crate::types::NodeChange::Selection {
                    id: node.id.clone(),
                    selected: false,
                });
            }
        }
        if let Some(handler) = on_nodes_change {
            handler.call(node_changes);
        } else {
            state.apply_node_changes(node_changes);
        }
    }

    if let Some(handler) = on_edges_change {
        handler.call(edge_changes);
    } else {
        state.apply_edge_changes(edge_changes);
    }
}

fn focus_dom_element(element: &web_sys::Element) {
    if let Ok(focus_fn) = Reflect::get(element, &JsValue::from_str("focus")) {
        if let Some(func) = focus_fn.dyn_ref::<Function>() {
            let _ = func.call0(element);
        }
    }
}

fn clamp_to_extent(
    extent: CoordinateExtent,
    position: XYPosition,
    dims: crate::types::Dimensions,
) -> XYPosition {
    let min_x = extent[0][0];
    let min_y = extent[0][1];
    let max_x = extent[1][0];
    let max_y = extent[1][1];

    let max_x = if max_x.is_finite() {
        max_x - dims.width
    } else {
        max_x
    };
    let max_y = if max_y.is_finite() {
        max_y - dims.height
    } else {
        max_y
    };

    XYPosition {
        x: position.x.clamp(min_x, max_x),
        y: position.y.clamp(min_y, max_y),
    }
}

/// Props passed to custom node components
#[derive(Clone, PartialEq, Props)]
pub struct NodeProps<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
> {
    pub node: Node<N>,
    #[props(default)]
    pub selected: bool,
    #[props(default)]
    pub draggable: bool,
    #[props(default)]
    pub connectable: bool,
    #[props(default)]
    pub selectable: bool,
    #[props(default)]
    pub _marker: std::marker::PhantomData<E>,
}

/// Props passed to custom edge components
#[derive(Clone, PartialEq, Props)]
pub struct EdgeComponentProps<T: Clone + PartialEq + Default + 'static> {
    pub edge: Edge<T>,
    pub source_x: f64,
    pub source_y: f64,
    pub target_x: f64,
    pub target_y: f64,
    pub source_position: crate::types::Position,
    pub target_position: crate::types::Position,
}
