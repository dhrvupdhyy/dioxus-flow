//! Store updater component

use dioxus::prelude::*;
use dioxus::prelude::{ReadableExt, WritableExt};

use crate::state::FlowState;
use crate::types::{ConnectionLineType, CoordinateExtent, Node, SelectionMode, Viewport};

#[component]
pub fn StoreUpdater<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    #[props(default)] nodes: Option<Signal<Vec<Node<N>>>>,
    #[props(default)] edges: Option<Signal<Vec<crate::types::Edge<E>>>>,
    #[props(default = 0.5)] min_zoom: f64,
    #[props(default = 2.0)] max_zoom: f64,
    #[props(default)] viewport: Option<Viewport>,
    #[props(default)] translate_extent: Option<CoordinateExtent>,
    #[props(default)] node_extent: Option<CoordinateExtent>,
    #[props(default = (0.0, 0.0))] node_origin: crate::types::NodeOrigin,
    #[props(default = true)] zoom_on_scroll: bool,
    #[props(default = true)] zoom_on_pinch: bool,
    #[props(default = true)] zoom_on_double_click: bool,
    #[props(default = true)] pan_on_drag: bool,
    #[props(default)] pan_on_drag_buttons: Option<Vec<i32>>,
    #[props(default)] pan_on_scroll: bool,
    #[props(default = 0.5)] pan_on_scroll_speed: f64,
    #[props(default = crate::types::PanOnScrollMode::Free)]
    pan_on_scroll_mode: crate::types::PanOnScrollMode,
    #[props(default = true)] prevent_scrolling: bool,
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
    #[props(default = false)] selection_on_drag: bool,
    #[props(default = SelectionMode::Partial)] selection_mode: SelectionMode,
    #[props(default)] connection_mode: Option<crate::types::ConnectionMode>,
    #[props(default)] connection_line_type: Option<ConnectionLineType>,
    #[props(default)] connection_line_style: Option<String>,
    #[props(default = 20.0)] connection_radius: f64,
    #[props(default = 10.0)] reconnect_radius: f64,
    #[props(default = 1.0)] node_drag_threshold: f64,
    #[props(default = 1.0)] connection_drag_threshold: f64,
    #[props(default = true)] connect_on_click: bool,
    #[props(default)] default_marker_color: Option<String>,
    #[props(default = "nodrag".to_string())] no_drag_class_name: String,
    #[props(default = "nowheel".to_string())] no_wheel_class_name: String,
    #[props(default = "nopan".to_string())] no_pan_class_name: String,
    #[props(default = false)] only_render_visible_elements: bool,
    #[props(default = 0.2)] visible_area_padding: f64,
    #[props(default = true)] elevate_nodes_on_select: bool,
    #[props(default = false)] elevate_edges_on_select: bool,
    #[props(default = crate::types::ZIndexMode::Basic)] z_index_mode: crate::types::ZIndexMode,
    #[props(default = false)] disable_keyboard_a11y: bool,
    #[props(default)] width: Option<f64>,
    #[props(default)] height: Option<f64>,
    #[props(default = crate::types::ColorMode::Light)] color_mode: crate::types::ColorMode,
    #[props(default = false)] debug: bool,
    #[props(default)] aria_label_config: Option<crate::types::AriaLabelConfig>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();

    let mut state_config = state.clone();
    use_effect(move || {
        state_config.min_zoom.set(min_zoom);
        state_config.max_zoom.set(max_zoom);
        state_config.translate_extent.set(translate_extent);
        state_config.node_extent.set(node_extent);
        state_config.node_origin.set(node_origin);
        state_config.color_mode.set(color_mode);
        state_config.default_marker_color.set(default_marker_color.clone());
        state_config.z_index_mode.set(z_index_mode);
        state_config.elevate_nodes_on_select.set(elevate_nodes_on_select);
        state_config.elevate_edges_on_select.set(elevate_edges_on_select);
        state_config.disable_keyboard_a11y.set(disable_keyboard_a11y);
        state_config.debug.set(debug);
        if let Some(config) = aria_label_config.clone() {
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
        state_config.selection_on_drag.set(selection_on_drag);
        state_config.selection_mode.set(selection_mode);
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
        state_config
            .only_render_visible_elements
            .set(only_render_visible_elements);
        state_config.visible_area_padding.set(visible_area_padding);

        if let Some(width) = width {
            state_config.width.set(width);
        }
        if let Some(height) = height {
            state_config.height.set(height);
        }

        if let Some(mode) = connection_mode {
            state_config.connection_mode.set(mode);
        }
        if let Some(line_type) = connection_line_type {
            state_config.connection_line_type.set(line_type);
        }
        state_config
            .connection_line_style
            .set(connection_line_style.clone());
        if let Some(viewport) = viewport {
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
    });

    rsx! {}
}
