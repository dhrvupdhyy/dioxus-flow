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
    #[props(default = true)] zoom_on_scroll: bool,
    #[props(default = true)] zoom_on_pinch: bool,
    #[props(default = true)] zoom_on_double_click: bool,
    #[props(default = true)] pan_on_drag: bool,
    #[props(default)] pan_on_scroll: bool,
    #[props(default = crate::types::PanOnScrollMode::Free)]
    pan_on_scroll_mode: crate::types::PanOnScrollMode,
    #[props(default = true)] nodes_draggable: bool,
    #[props(default = true)] nodes_connectable: bool,
    #[props(default = true)] nodes_focusable: bool,
    #[props(default = true)] elements_selectable: bool,
    #[props(default = false)] selection_on_drag: bool,
    #[props(default = SelectionMode::Partial)] selection_mode: SelectionMode,
    #[props(default)] connection_mode: Option<crate::types::ConnectionMode>,
    #[props(default)] connection_line_type: Option<ConnectionLineType>,
    #[props(default)] connection_line_style: Option<String>,
    #[props(default = 20.0)] connection_radius: f64,
    #[props(default = false)] only_render_visible_elements: bool,
    #[props(default = 0.2)] visible_area_padding: f64,
) -> Element {
    let state = use_context::<FlowState<N, E>>();

    let mut state_config = state.clone();
    use_effect(move || {
        state_config.min_zoom.set(min_zoom);
        state_config.max_zoom.set(max_zoom);
        state_config.translate_extent.set(translate_extent);
        state_config.node_extent.set(node_extent);
        state_config.zoom_on_scroll.set(zoom_on_scroll);
        state_config.zoom_on_pinch.set(zoom_on_pinch);
        state_config.zoom_on_double_click.set(zoom_on_double_click);
        state_config.pan_on_drag.set(pan_on_drag);
        state_config.pan_on_scroll.set(pan_on_scroll);
        state_config.pan_on_scroll_mode.set(pan_on_scroll_mode);
        state_config.nodes_draggable.set(nodes_draggable);
        state_config.nodes_connectable.set(nodes_connectable);
        state_config.nodes_focusable.set(nodes_focusable);
        state_config.elements_selectable.set(elements_selectable);
        state_config.selection_on_drag.set(selection_on_drag);
        state_config.selection_mode.set(selection_mode);
        state_config.connection_radius.set(connection_radius);
        state_config
            .only_render_visible_elements
            .set(only_render_visible_elements);
        state_config.visible_area_padding.set(visible_area_padding);

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
