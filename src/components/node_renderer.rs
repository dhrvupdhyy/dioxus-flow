//! Node renderer component

use crate::components::{Handle, NodeProps};
use crate::state::FlowState;
use crate::types::{HandleBound, HandleBounds, HandleType, Node, Position, XYPosition};
use dioxus::prelude::dioxus_elements::input_data::MouseButton;
use dioxus::prelude::*;
use dioxus::prelude::{InteractionLocation, ModifiersInteraction, PointerInteraction, ReadableExt};
use dioxus_web::WebEventExt;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

#[component]
pub fn NodeRenderer<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    #[props(default)] node_types: Option<HashMap<String, Component<NodeProps<N, E>>>>,
    #[props(default)] on_nodes_change: Option<EventHandler<Vec<crate::types::NodeChange<N>>>>,
    #[props(default)] on_edges_change: Option<EventHandler<Vec<crate::types::EdgeChange<E>>>>,
    #[props(default)] on_node_drag_start: Option<EventHandler<crate::types::NodeDragEvent<N>>>,
    #[props(default)] on_node_click: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_node_double_click: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_node_mouse_enter: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_node_mouse_leave: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] _marker: std::marker::PhantomData<E>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();
    let nodes_memo: Memo<Vec<Node<N>>> = use_memo(move || {
        if *state.only_render_visible_elements.read() {
            state.get_visible_nodes()
        } else {
            state
                .nodes
                .read()
                .iter()
                .filter(|node| !node.hidden)
                .cloned()
                .collect()
        }
    });
    let nodes = nodes_memo.read().clone();

    rsx! {
        div {
            class: "dioxus-flow__nodes",
            for node in nodes {
                NodeWrapper::<N, E> {
                    key: "{node.id}",
                    node,
                    node_types: node_types.clone(),
                    on_nodes_change: on_nodes_change.clone(),
                    on_edges_change: on_edges_change.clone(),
                    on_node_drag_start: on_node_drag_start.clone(),
                    on_node_click: on_node_click.clone(),
                    on_node_double_click: on_node_double_click.clone(),
                    on_node_mouse_enter: on_node_mouse_enter.clone(),
                    on_node_mouse_leave: on_node_mouse_leave.clone(),
                }
            }
        }
    }
}

#[component]
fn NodeWrapper<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    node: Node<N>,
    #[props(default)] node_types: Option<HashMap<String, Component<NodeProps<N, E>>>>,
    #[props(default)] on_nodes_change: Option<EventHandler<Vec<crate::types::NodeChange<N>>>>,
    #[props(default)] on_edges_change: Option<EventHandler<Vec<crate::types::EdgeChange<E>>>>,
    #[props(default)] on_node_drag_start: Option<EventHandler<crate::types::NodeDragEvent<N>>>,
    #[props(default)] on_node_click: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_node_double_click: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_node_mouse_enter: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] on_node_mouse_leave: Option<EventHandler<crate::types::NodeMouseEvent<N>>>,
    #[props(default)] _marker: std::marker::PhantomData<E>,
) -> Element {
    let _node_id_context = use_context_provider(|| crate::state::NodeIdContext(node.id.clone()));
    let state = use_context::<FlowState<N, E>>();
    let mut resize_observer = use_signal(|| None::<ResizeObserverCleanup>);

    let dims = node.get_dimensions();
    let position = state
        .node_lookup
        .read()
        .get(&node.id)
        .map(|internal| internal.position_absolute)
        .unwrap_or_else(|| {
            let origin = *state.node_origin.read();
            XYPosition {
                x: node.position.x - dims.width * origin.0,
                y: node.position.y - dims.height * origin.1,
            }
        });
    let mut style = format!(
        "transform: translate({}px, {}px); width: {}px; height: {}px;",
        position.x, position.y, dims.width, dims.height
    );
    let base_z = node.z_index.unwrap_or(0);
    let z_mode = *state.z_index_mode.read();
    let elevate = *state.elevate_nodes_on_select.read();
    let z_index = if elevate && node.selected && z_mode != crate::types::ZIndexMode::Manual {
        base_z + 1000
    } else {
        base_z
    };
    if z_index != 0 {
        style.push_str(&format!(" z-index: {};", z_index));
    }
    if let Some(extra) = &node.style {
        style.push_str(&format!(" {}", extra));
    }

    let selected = node.selected;
    let draggable = node.draggable.unwrap_or(true);
    let connectable = node.connectable.unwrap_or(true);
    let selectable = node.selectable.unwrap_or(true);

    let node_id = node.id.clone();
    let node_for_drag = node.clone();
    let drag_handle_selector = node.drag_handle.clone();
    let mut state_down = state.clone();
    let on_pointer_down = move |evt: PointerEvent| {
        if evt.data.trigger_button() != Some(MouseButton::Primary) {
            return;
        }
        evt.stop_propagation();

        let web_evt = evt.data.as_web_event();
        if let Some(target) = web_evt
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        {
            let no_drag_class = state_down.no_drag_class_name.read().clone();
            if !no_drag_class.is_empty()
                && target.closest(&format!(".{}", no_drag_class)).ok().flatten().is_some()
            {
                return;
            }
        }

        if let Some(selector) = &drag_handle_selector {
            let web_evt = evt.data.as_web_event();
            let Some(target) = web_evt
                .target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            else {
                return;
            };
            if target.closest(selector).ok().flatten().is_none() {
                return;
            }
        }

        if selectable && *state_down.elements_selectable.read() {
            let modifiers = evt.data.modifiers();
            let multi = *state_down.multi_selection_key_pressed.read()
                || modifiers.shift()
                || modifiers.meta()
                || modifiers.ctrl();
            if *state_down.select_nodes_on_drag.read() {
                let nodes = state_down.nodes.read().clone();
                let mut changes = Vec::new();

                if multi {
                    let next = !selected;
                    changes.push(crate::types::NodeChange::Selection {
                        id: node_id.clone(),
                        selected: next,
                    });
                } else {
                    for other in nodes.iter() {
                        let should_select = other.id == node_id;
                        if other.selected != should_select {
                            changes.push(crate::types::NodeChange::Selection {
                                id: other.id.clone(),
                                selected: should_select,
                            });
                        }
                    }

                    let edges = state_down.edges.read().clone();
                    let mut edge_changes = Vec::new();
                    for edge in edges.iter() {
                        if edge.selected {
                            edge_changes.push(crate::types::EdgeChange::Selection {
                                id: edge.id.clone(),
                                selected: false,
                            });
                        }
                    }
                    apply_edge_changes(&mut state_down, &on_edges_change, edge_changes);
                }

                apply_node_changes(&mut state_down, &on_nodes_change, changes);
            } else {
                state_down.pending_node_click.set(Some(crate::state::PendingNodeClick {
                    node_id: node_id.clone(),
                    multi,
                }));
            }
        }

        if !draggable || !*state_down.nodes_draggable.read() {
            return;
        }

        let coords = evt.data.client_coordinates();
        let start_pointer = state_down.screen_to_flow_position(XYPosition::new(coords.x, coords.y));
        let drag_nodes: Vec<Node<N>> = if node_for_drag.selected {
            state_down.get_selected_nodes()
        } else {
            vec![node_for_drag.clone()]
        };
        let drag_positions = drag_nodes
            .iter()
            .map(|n| (n.id.clone(), n.position))
            .collect();
        state_down.node_drag.set(Some(crate::state::NodeDragState {
            origin_node_id: node_id.clone(),
            start_pointer,
            nodes: drag_positions,
            started: false,
        }));

        if let Some(handler) = &on_node_drag_start {
            handler.call(crate::types::NodeDragEvent {
                node: node_for_drag.clone(),
                nodes: drag_nodes,
            });
        }
    };

    let node_click = node.clone();
    let on_click = move |_| {
        if let Some(handler) = &on_node_click {
            handler.call(crate::types::NodeMouseEvent {
                node: node_click.clone(),
            });
        }
    };
    let node_double_click = node.clone();
    let on_double_click = move |_| {
        if let Some(handler) = &on_node_double_click {
            handler.call(crate::types::NodeMouseEvent {
                node: node_double_click.clone(),
            });
        }
    };
    let node_enter = node.clone();
    let on_mouse_enter = move |_| {
        if let Some(handler) = &on_node_mouse_enter {
            handler.call(crate::types::NodeMouseEvent {
                node: node_enter.clone(),
            });
        }
    };
    let node_leave = node.clone();
    let on_mouse_leave = move |_| {
        if let Some(handler) = &on_node_mouse_leave {
            handler.call(crate::types::NodeMouseEvent {
                node: node_leave.clone(),
            });
        }
    };

    let node_component = node
        .node_type
        .as_ref()
        .and_then(|t| node_types.as_ref().and_then(|map| map.get(t)))
        .cloned();

    let content = if let Some(component) = node_component {
        component(NodeProps {
            node: node.clone(),
            selected,
            draggable,
            connectable,
            selectable,
            _marker: std::marker::PhantomData,
        })
    } else {
        rsx! {
            div {
                class: "dioxus-flow__node-default",
                "{node.id}"
                Handle::<N, E> { position: Position::Left, handle_type: crate::types::HandleType::Target, node_id: node.id.clone(), is_connectable: connectable }
                Handle::<N, E> { position: Position::Right, handle_type: crate::types::HandleType::Source, node_id: node.id.clone(), is_connectable: connectable }
            }
        }
    };

    let mut base_class = String::from("dioxus-flow__node");
    if selectable {
        base_class.push_str(" selectable");
    }
    if draggable {
        base_class.push_str(" draggable");
    }
    if selected {
        base_class.push_str(" selected");
    }
    let class = if let Some(extra) = &node.class_name {
        format!("{} {}", base_class, extra)
    } else {
        base_class.to_string()
    };

    let aria_config = state.aria_label_config.read().clone();
    let aria_label = node.aria_label.clone().or(aria_config.node).unwrap_or_else(|| {
        format!("Node {}", node.id)
    });

    let tab_index = if *state.nodes_focusable.read() && node.focusable.unwrap_or(true) {
        "0"
    } else {
        "-1"
    };

    rsx! {
        div {
            class: "{class}",
            style: "{style}",
            "data-id": "{node.id}",
            "aria-label": "{aria_label}",
            role: "group",
            tabindex: "{tab_index}",
            onfocus: {
                let mut state_focus = state.clone();
                let node_id = node.id.clone();
                move |_| {
                    state_focus.focused_node_id.set(Some(node_id.clone()));
                    state_focus.focused_edge_id.set(None);
                }
            },
            onblur: {
                let mut state_blur = state.clone();
                let node_id = node.id.clone();
                move |_| {
                    if state_blur.focused_node_id.read().as_ref() == Some(&node_id) {
                        state_blur.focused_node_id.set(None);
                    }
                }
            },
            onpointerdown: on_pointer_down,
            onclick: on_click,
            ondoubleclick: on_double_click,
            onmouseenter: on_mouse_enter,
            onmouseleave: on_mouse_leave,
            onmounted: move |evt| {
                if resize_observer.read().is_some() {
                    return;
                }
                let element: web_sys::Element = evt.as_web_event();
                let element_for_cb = element.clone();
                let node_id = node.id.clone();
                let mut state_resize = state.clone();
                let handler = on_nodes_change.clone();
                let node_id_for_bounds = node.id.clone();

                let zoom = state_resize.viewport.read().zoom.max(0.0001);
                if let Some(bounds) = compute_handle_bounds(&element, zoom) {
                    state_resize.update_handle_bounds(&node_id_for_bounds, bounds);
                }

                let callback = Closure::<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>::wrap(Box::new(
                    move |entries, _observer| {
                        if entries.length() == 0 {
                            return;
                        }
                        let entry = entries.get(0).unchecked_into::<web_sys::ResizeObserverEntry>();
                        let rect = entry.content_rect();
                        let dims = crate::types::Dimensions {
                            width: rect.width(),
                            height: rect.height(),
                        };
                        let change = crate::types::NodeChange::Dimensions {
                            id: node_id.clone(),
                            dimensions: Some(dims),
                            resizing: false,
                        };
                        apply_node_changes(&mut state_resize, &handler, vec![change]);

                        let zoom = state_resize.viewport.read().zoom.max(0.0001);
                        if let Some(bounds) = compute_handle_bounds(&element_for_cb, zoom) {
                            state_resize.update_handle_bounds(&node_id_for_bounds, bounds);
                        }
                    },
                ));

                if let Ok(observer) = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()) {
                    observer.observe(&element);
                    resize_observer.set(Some(ResizeObserverCleanup {
                        observer,
                        callback: Some(callback),
                    }));
                }
            },
            {content}
        }
    }
}

struct ResizeObserverCleanup {
    observer: web_sys::ResizeObserver,
    callback: Option<Closure<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>>,
}

impl Drop for ResizeObserverCleanup {
    fn drop(&mut self) {
        self.observer.disconnect();
        self.callback.take();
    }
}

fn compute_handle_bounds(element: &web_sys::Element, zoom: f64) -> Option<HandleBounds> {
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

fn apply_edge_changes<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &mut FlowState<N, E>,
    handler: &Option<EventHandler<Vec<crate::types::EdgeChange<E>>>>,
    changes: Vec<crate::types::EdgeChange<E>>,
) {
    if changes.is_empty() {
        return;
    }
    if let Some(handler) = handler {
        handler.call(changes);
    } else {
        state.apply_edge_changes(changes);
    }
}

fn apply_node_changes<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &mut FlowState<N, E>,
    handler: &Option<EventHandler<Vec<crate::types::NodeChange<N>>>>,
    changes: Vec<crate::types::NodeChange<N>>,
) {
    if changes.is_empty() {
        return;
    }
    if let Some(handler) = handler {
        handler.call(changes);
    } else {
        state.apply_node_changes(changes);
    }
}
