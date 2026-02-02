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
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

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
    #[props(default)] translate_extent: Option<CoordinateExtent>,
    #[props(default = true)] zoom_on_scroll: bool,
    #[props(default = true)] zoom_on_pinch: bool,
    #[props(default = true)] zoom_on_double_click: bool,
    #[props(default = true)] pan_on_drag: bool,
    #[props(default)] pan_on_scroll: bool,
    #[props(default = PanOnScrollMode::Free)] pan_on_scroll_mode: PanOnScrollMode,
    #[props(default = true)] nodes_draggable: bool,
    #[props(default = true)] nodes_connectable: bool,
    #[props(default = true)] nodes_focusable: bool,
    #[props(default = true)] elements_selectable: bool,
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
    #[props(default)] delete_key_code: Option<Vec<String>>,
    #[props(default)] selection_key_code: Option<Vec<String>>,
    #[props(default)] multi_selection_key_code: Option<Vec<String>>,
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
                translate_extent,
                zoom_on_scroll,
                zoom_on_pinch,
                zoom_on_double_click,
                pan_on_drag,
                pan_on_scroll,
                pan_on_scroll_mode,
                nodes_draggable,
                nodes_connectable,
                nodes_focusable,
                elements_selectable,
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
                delete_key_code,
                selection_key_code,
                multi_selection_key_code,
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
    #[props(default)] translate_extent: Option<CoordinateExtent>,
    #[props(default = true)] zoom_on_scroll: bool,
    #[props(default = true)] zoom_on_pinch: bool,
    #[props(default = true)] zoom_on_double_click: bool,
    #[props(default = true)] pan_on_drag: bool,
    #[props(default)] pan_on_scroll: bool,
    #[props(default = PanOnScrollMode::Free)] pan_on_scroll_mode: PanOnScrollMode,
    #[props(default = true)] nodes_draggable: bool,
    #[props(default = true)] nodes_connectable: bool,
    #[props(default = true)] nodes_focusable: bool,
    #[props(default = true)] elements_selectable: bool,
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
    #[props(default)] delete_key_code: Option<Vec<String>>,
    #[props(default)] selection_key_code: Option<Vec<String>>,
    #[props(default)] multi_selection_key_code: Option<Vec<String>>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();

    let mut state_config = state.clone();
    use_effect(move || {
        state_config.min_zoom.set(min_zoom);
        state_config.max_zoom.set(max_zoom);
        state_config.translate_extent.set(translate_extent);
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

    let class = class.unwrap_or_default();
    let style = style.unwrap_or_default();
    let connection_active = state.connection.read().in_progress;
    let flow_class = if connection_active {
        format!("dioxus-flow dioxus-flow--connecting {class}")
    } else {
        format!("dioxus-flow {class}")
    };

    let delete_keys =
        delete_key_code.unwrap_or_else(|| vec!["Backspace".to_string(), "Delete".to_string()]);
    let delete_pressed = crate::hooks::use_key_press_multi(delete_keys);
    let mut delete_latched = use_signal(|| false);
    let mut state_delete = state.clone();
    use_effect(move || {
        let pressed = *delete_pressed.read();
        if pressed && !*delete_latched.read() {
            state_delete.delete_selected();
            delete_latched.set(true);
        } else if !pressed && *delete_latched.read() {
            delete_latched.set(false);
        }
    });

    let selection_keys = selection_key_code.unwrap_or_else(|| vec!["Shift".to_string()]);
    let selection_pressed = crate::hooks::use_key_press_multi(selection_keys);
    let mut state_selection = state.clone();
    use_effect(move || {
        state_selection
            .selection_key_pressed
            .set(*selection_pressed.read());
    });

    let multi_keys =
        multi_selection_key_code.unwrap_or_else(|| vec!["Meta".to_string(), "Control".to_string()]);
    let multi_pressed = crate::hooks::use_key_press_multi(multi_keys);
    let mut state_multi = state.clone();
    use_effect(move || {
        state_multi
            .multi_selection_key_pressed
            .set(*multi_pressed.read());
    });

    let mut state_keyboard = state.clone();
    let on_nodes_change_keyboard = on_nodes_change.clone();
    let on_edges_change_keyboard = on_edges_change.clone();
    let _keyboard_listener = use_hook(move || {
        let mut state_keyboard_event = state_keyboard.clone();
        let on_nodes_change_keyboard = on_nodes_change_keyboard.clone();
        let on_edges_change_keyboard = on_edges_change_keyboard.clone();
        Rc::new(WindowListener::new(
            "keydown",
            move |evt: web_sys::KeyboardEvent| {
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
                        focus_next_node(&mut state_keyboard_event, evt.shift_key());
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
        let window = web_sys::window().expect("window not available");
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

fn focus_next_node<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &mut FlowState<N, E>,
    reverse: bool,
) {
    if !*state.nodes_focusable.read() {
        return;
    }
    let focusable: Vec<String> = {
        let nodes = state.nodes.read();
        nodes
            .iter()
            .filter(|node| !node.hidden && node.focusable.unwrap_or(true))
            .map(|node| node.id.clone())
            .collect()
    };
    if focusable.is_empty() {
        return;
    }
    let current = state.focused_node_id.read().clone();
    let next_index = current
        .as_ref()
        .and_then(|id| focusable.iter().position(|v| v == id))
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
    let next_id = focusable[next_index].clone();
    state.focused_node_id.set(Some(next_id.clone()));

    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            let selector = format!("[data-id=\"{}\"]", next_id.replace('\"', "\\\""));
            if let Ok(Some(element)) = document.query_selector(&selector) {
                if let Some(html) = element.dyn_into::<web_sys::HtmlElement>().ok() {
                    let _ = html.focus();
                }
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
    let Some(focused_id) = state.focused_node_id.read().clone() else {
        return;
    };
    let nodes = state.nodes.read().clone();
    let mut changes = Vec::new();
    let multi = *state.multi_selection_key_pressed.read();
    for node in nodes.iter() {
        let should_select = if node.id == focused_id {
            if multi {
                !node.selected
            } else {
                true
            }
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
