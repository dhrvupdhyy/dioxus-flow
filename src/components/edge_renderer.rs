//! Edge renderer component

use crate::components::EdgeComponentProps;
use crate::state::FlowState;
use crate::types::{
    Edge, EdgeMarker, HandleBound, HandleBounds, HandleType, MarkerType, Position,
    ReconnectableValue, XYPosition,
};
use crate::utils::{
    get_bezier_path, get_simple_bezier_path, get_smooth_step_path, get_step_path, get_straight_path,
};
use dioxus::prelude::dioxus_elements::input_data::MouseButton;
use dioxus::prelude::*;
use dioxus::prelude::{ModifiersInteraction, PointerInteraction, ReadableExt};
use std::collections::{HashMap, HashSet};

#[component]
#[allow(unused_variables)]
pub fn EdgeRenderer<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    #[props(default)] edge_types: Option<HashMap<String, Component<EdgeComponentProps<E>>>>,
    #[props(default)] on_nodes_change: Option<EventHandler<Vec<crate::types::NodeChange<N>>>>,
    #[props(default)] on_edges_change: Option<EventHandler<Vec<crate::types::EdgeChange<E>>>>,
    #[props(default)] on_connect: Option<EventHandler<crate::types::Connection>>,
    #[props(default)] on_edge_update_start: Option<EventHandler<Edge<E>>>,
    #[props(default)] on_edge_click: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
    #[props(default)] on_edge_double_click: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
    #[props(default)] on_edge_mouse_enter: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
    #[props(default)] on_edge_mouse_leave: Option<EventHandler<crate::types::EdgeMouseEvent<E>>>,
    #[props(default)] _marker: std::marker::PhantomData<N>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();
    let state_visible = state.clone();
    let render_edges_memo: Memo<Vec<EdgeRender<E>>> = use_memo(move || {
        let edges = state_visible.edges.read();
        let nodes = state_visible.node_lookup.read();
        let visible_ids = if *state_visible.only_render_visible_elements.read() {
            Some(
                state_visible
                    .get_visible_nodes()
                    .into_iter()
                    .map(|node| node.id)
                    .collect::<HashSet<String>>(),
            )
        } else {
            None
        };
        let z_index_mode = *state_visible.z_index_mode.read();
        let elevate = *state_visible.elevate_edges_on_select.read();
        let mut items: Vec<EdgeRender<E>> = edges
            .iter()
            .filter_map(|edge| {
                if let Some(visible) = &visible_ids {
                    if !visible.contains(&edge.source) && !visible.contains(&edge.target) {
                        return None;
                    }
                }
                let source_node = nodes.get(&edge.source)?;
                let target_node = nodes.get(&edge.target)?;

                let source_pos = source_node.node.source_position.unwrap_or(Position::Right);
                let target_pos = target_node.node.target_position.unwrap_or(Position::Left);

                let (source_x, source_y) = handle_position_for_edge(
                    &source_node,
                    HandleType::Source,
                    edge.source_handle.as_deref(),
                    source_pos,
                );
                let (target_x, target_y) = handle_position_for_edge(
                    &target_node,
                    HandleType::Target,
                    edge.target_handle.as_deref(),
                    target_pos,
                );

                Some(EdgeRender {
                    edge: edge.clone(),
                    source_x,
                    source_y,
                    target_x,
                    target_y,
                    source_pos,
                    target_pos,
                })
            })
            .collect();

        items.sort_by_key(|item| {
            let base = item.edge.z_index.unwrap_or(0);
            if elevate && item.edge.selected && z_index_mode != crate::types::ZIndexMode::Manual {
                base + 1000
            } else {
                base
            }
        });
        items
    });
    let render_edges = render_edges_memo.read();

    let mut marker_defs: Vec<(String, EdgeMarker)> = Vec::new();
    let mut marker_ids: HashMap<String, String> = HashMap::new();
    let default_marker_color = state.default_marker_color.read().clone();
    for item in render_edges.iter() {
        if let Some(marker) = item.edge.marker_start.clone() {
            let marker = if marker.color.is_none() {
                if let Some(color) = default_marker_color.clone() {
                    let mut m = marker;
                    m.color = Some(color);
                    m
                } else {
                    marker
                }
            } else {
                marker
            };
            resolve_marker_id(&marker, &mut marker_ids, &mut marker_defs);
        }
        if let Some(marker) = item.edge.marker_end.clone() {
            let marker = if marker.color.is_none() {
                if let Some(color) = default_marker_color.clone() {
                    let mut m = marker;
                    m.color = Some(color);
                    m
                } else {
                    marker
                }
            } else {
                marker
            };
            resolve_marker_id(&marker, &mut marker_ids, &mut marker_defs);
        }
    }

    let edge_elements: Vec<Element> = render_edges
        .iter()
        .map(|item| {
            let edge = &item.edge;
            let source_x = item.source_x;
            let source_y = item.source_y;
            let target_x = item.target_x;
            let target_y = item.target_y;
            let source_pos = item.source_pos;
            let target_pos = item.target_pos;

            let custom = edge
                .edge_type
                .as_ref()
                .and_then(|t| edge_types.as_ref().and_then(|map| map.get(t)))
                .cloned();

            if let Some(component) = custom {
                component(EdgeComponentProps {
                    edge: edge.clone(),
                    source_x,
                    source_y,
                    target_x,
                    target_y,
                    source_position: source_pos,
                    target_position: target_pos,
                })
            } else {
                let animated = edge.animated;
                let path_result = edge_path_for_type(
                    edge,
                    source_x,
                    source_y,
                    target_x,
                    target_y,
                    source_pos,
                    target_pos,
                );
                let base_class = match (edge.selected, animated) {
                    (true, true) => "dioxus-flow__edge-path animated",
                    (true, false) => "dioxus-flow__edge-path",
                    (false, true) => "dioxus-flow__edge-path animated",
                    (false, false) => "dioxus-flow__edge-path",
                };
                let class = if let Some(extra) = &edge.class_name {
                    format!("{} {}", base_class, extra)
                } else {
                    base_class.to_string()
                };
                let style = edge.style.clone().unwrap_or_default();
                let edge_id = edge.id.clone();
                let edge_id_for_select = edge_id.clone();
                let edge_selectable = edge.selectable.unwrap_or(true);
                let edge_class = if edge_selectable {
                    if edge.selected {
                        "dioxus-flow__edge selectable selected"
                    } else {
                        "dioxus-flow__edge selectable"
                    }
                } else if edge.selected {
                    "dioxus-flow__edge selected"
                } else {
                    "dioxus-flow__edge"
                };
                let edge_selected = edge.selected;
                let mut state_select = state.clone();
                let on_edges_change_select = on_edges_change.clone();
                let on_nodes_change_select = on_nodes_change.clone();
                let on_edge_pointer_down = move |evt: PointerEvent| {
                    if evt.data.trigger_button() != Some(MouseButton::Primary) {
                        return;
                    }
                    if !edge_selectable || !*state_select.elements_selectable.read() {
                        return;
                    }
                    evt.stop_propagation();
                    let modifiers = evt.data.modifiers();
                    let multi = *state_select.multi_selection_key_pressed.read()
                        || modifiers.shift()
                        || modifiers.meta()
                        || modifiers.ctrl();

                    let mut edge_changes = Vec::new();
                    if multi {
                        edge_changes.push(crate::types::EdgeChange::Selection {
                            id: edge_id_for_select.clone(),
                            selected: !edge_selected,
                        });
                    } else {
                        let edges = state_select.edges.read().clone();
                        for edge in edges.iter() {
                            let should_select = edge.id == edge_id_for_select;
                            if edge.selected != should_select {
                                edge_changes.push(crate::types::EdgeChange::Selection {
                                    id: edge.id.clone(),
                                    selected: should_select,
                                });
                            }
                        }

                        let nodes = state_select.nodes.read().clone();
                        let mut node_changes = Vec::new();
                        for node in nodes.iter() {
                            if node.selected {
                                node_changes.push(crate::types::NodeChange::Selection {
                                    id: node.id.clone(),
                                    selected: false,
                                });
                            }
                        }
                        apply_node_changes(
                            &mut state_select,
                            &on_nodes_change_select,
                            node_changes,
                        );
                    }

                    apply_edge_changes(&mut state_select, &on_edges_change_select, edge_changes);
                };
                let marker_start_attr = item
                    .edge
                    .marker_start
                    .as_ref()
                    .and_then(|marker| marker_id_for(marker, &marker_ids))
                    .map(|id| format!("url(#{})", id))
                    .unwrap_or_default();
                let marker_end_attr = item
                    .edge
                    .marker_end
                    .as_ref()
                    .and_then(|marker| marker_id_for(marker, &marker_ids))
                    .map(|id| format!("url(#{})", id))
                    .unwrap_or_default();
                let label = edge.label.clone();
                let show_label_bg = edge.label_show_bg.unwrap_or(false);
                let label_style = edge.label_style.clone().unwrap_or_default();
                let label_bg_style = edge.label_bg_style.clone().unwrap_or_default();
                let label_padding = edge.label_bg_padding.unwrap_or((6.0, 4.0));
                let label_radius = edge.label_bg_border_radius.unwrap_or(0.0);
                let label_x = path_result.label_x;
                let label_y = path_result.label_y;
                let label_metrics = label.as_ref().map(|text: &String| {
                    let text_len = text.chars().count() as f64;
                    let text_width = text_len * 6.0;
                    let text_height = 14.0;
                    let bg_width = text_width + label_padding.0 * 2.0;
                    let bg_height = text_height + label_padding.1 * 2.0;
                    (bg_width, bg_height)
                });
                let (bg_width, bg_height) = label_metrics.unwrap_or((0.0, 0.0));
                let reconnectable = edge.reconnectable.unwrap_or(ReconnectableValue::True);
                let edges_reconnectable = *state.edges_reconnectable.read();
                let allow_reconnect_source = edges_reconnectable
                    && matches!(
                        reconnectable,
                        ReconnectableValue::True | ReconnectableValue::Source
                    );
                let allow_reconnect_target = edges_reconnectable
                    && matches!(
                        reconnectable,
                        ReconnectableValue::True | ReconnectableValue::Target
                    );
                let reconnect_radius = *state.reconnect_radius.read();
                let (source_anchor_x, source_anchor_y) =
                    shift_edge_anchor(source_x, source_y, source_pos, reconnect_radius);
                let (target_anchor_x, target_anchor_y) =
                    shift_edge_anchor(target_x, target_y, target_pos, reconnect_radius);
                let mut state_reconnect_source = state.clone();
                let mut state_reconnect_target = state.clone();
                let reconnect_edge_id = edge.id.clone();
                let edge_source = edge.source.clone();
                let edge_target = edge.target.clone();
                let edge_source_handle = edge.source_handle.clone();
                let edge_target_handle = edge.target_handle.clone();
                let edge_focusable =
                    *state.edges_focusable.read() && edge.focusable.unwrap_or(true);
                let edge_tab_index = if edge_focusable { "0" } else { "-1" };
                let aria_config = state.aria_label_config.read().clone();
                let edge_aria_label = aria_config
                    .edge
                    .unwrap_or_else(|| format!("Edge {}", edge.id));
                let edge_id_for_focus = edge_id.clone();
                let edge_id_for_blur = edge_id.clone();
                let on_edge_update_start_source = on_edge_update_start.clone();
                let on_edge_update_start_target = on_edge_update_start.clone();
                let edge_for_update = edge.clone();
                let edge_for_update_target = edge.clone();
                let reconnect_edge_id_source = reconnect_edge_id.clone();
                let on_reconnect_source = move |evt: PointerEvent| {
                    if !allow_reconnect_source {
                        return;
                    }
                    if evt.data.trigger_button() != Some(MouseButton::Primary) {
                        return;
                    }
                    evt.stop_propagation();
                    if let Some(handler) = &on_edge_update_start_source {
                        handler.call(edge_for_update.clone());
                    }
                    let mut connection = crate::types::ConnectionState::start_reconnect(
                        reconnect_edge_id_source.clone(),
                        HandleType::Source,
                        edge_source.clone(),
                        edge_source_handle.clone(),
                        HandleType::Source,
                        source_pos,
                    );
                    let screen_pos = state_reconnect_source
                        .flow_to_screen_position(XYPosition::new(source_x, source_y));
                    connection.update_screen_position(
                        screen_pos,
                        XYPosition::new(source_x, source_y),
                    );
                    connection.start_screen = Some(screen_pos);
                    state_reconnect_source.connection.set(connection);
                    if let Some(handler) = state_reconnect_source.on_connect_start.read().clone() {
                        handler.call(crate::types::ConnectionStartEvent {
                            node_id: edge_source.clone(),
                            handle_id: edge_source_handle.clone(),
                            handle_type: HandleType::Source,
                            position: source_pos,
                        });
                    }
                };
                let reconnect_edge_id_target = reconnect_edge_id.clone();
                let on_reconnect_target = move |evt: PointerEvent| {
                    if !allow_reconnect_target {
                        return;
                    }
                    if evt.data.trigger_button() != Some(MouseButton::Primary) {
                        return;
                    }
                    evt.stop_propagation();
                    if let Some(handler) = &on_edge_update_start_target {
                        handler.call(edge_for_update_target.clone());
                    }
                    let mut connection = crate::types::ConnectionState::start_reconnect(
                        reconnect_edge_id_target.clone(),
                        HandleType::Target,
                        edge_target.clone(),
                        edge_target_handle.clone(),
                        HandleType::Target,
                        target_pos,
                    );
                    let screen_pos = state_reconnect_target
                        .flow_to_screen_position(XYPosition::new(target_x, target_y));
                    connection.update_screen_position(
                        screen_pos,
                        XYPosition::new(target_x, target_y),
                    );
                    connection.start_screen = Some(screen_pos);
                    state_reconnect_target.connection.set(connection);
                    if let Some(handler) = state_reconnect_target.on_connect_start.read().clone() {
                        handler.call(crate::types::ConnectionStartEvent {
                            node_id: edge_target.clone(),
                            handle_id: edge_target_handle.clone(),
                            handle_type: HandleType::Target,
                            position: target_pos,
                        });
                    }
                };

                let edge_click = edge.clone();
                let on_click = move |_| {
                    if let Some(handler) = &on_edge_click {
                        handler.call(crate::types::EdgeMouseEvent {
                            edge: edge_click.clone(),
                        });
                    }
                };
                let edge_double = edge.clone();
                let on_double_click = move |_| {
                    if let Some(handler) = &on_edge_double_click {
                        handler.call(crate::types::EdgeMouseEvent {
                            edge: edge_double.clone(),
                        });
                    }
                };
                let edge_enter = edge.clone();
                let on_mouse_enter = move |_| {
                    if let Some(handler) = &on_edge_mouse_enter {
                        handler.call(crate::types::EdgeMouseEvent {
                            edge: edge_enter.clone(),
                        });
                    }
                };
                let edge_leave = edge.clone();
                let on_mouse_leave = move |_| {
                    if let Some(handler) = &on_edge_mouse_leave {
                        handler.call(crate::types::EdgeMouseEvent {
                            edge: edge_leave.clone(),
                        });
                    }
                };

                rsx! {
                    g {
                        class: "{edge_class}",
                        role: if edge_selectable { "button" } else { "group" },
                        tabindex: "{edge_tab_index}",
                        "aria-label": "{edge_aria_label}",
                        "data-edge-id": "{edge_id}",
                        onclick: on_click,
                        ondoubleclick: on_double_click,
                        onmouseenter: on_mouse_enter,
                        onmouseleave: on_mouse_leave,
                        onfocus: {
                            let mut state_focus = state.clone();
                            let edge_id = edge_id_for_focus.clone();
                            move |_| {
                                state_focus.focused_edge_id.set(Some(edge_id.clone()));
                                state_focus.focused_node_id.set(None);
                            }
                        },
                        onblur: {
                            let mut state_blur = state.clone();
                            let edge_id = edge_id_for_blur.clone();
                            move |_| {
                                if state_blur.focused_edge_id.read().as_ref() == Some(&edge_id) {
                                    state_blur.focused_edge_id.set(None);
                                }
                            }
                        },
                        path {
                            class: "{class}",
                            style: "{style}",
                            d: "{path_result.path}",
                            marker_start: "{marker_start_attr}",
                            marker_end: "{marker_end_attr}",
                        }
                        path {
                            class: "dioxus-flow__edge-interaction",
                            d: "{path_result.path}",
                            stroke_width: "{edge.interaction_width.unwrap_or(20.0)}",
                            onpointerdown: on_edge_pointer_down,
                        }
                        if allow_reconnect_source {
                            circle {
                                class: "dioxus-flow__edge-reconnect dioxus-flow__edge-reconnect-source",
                                cx: "{source_anchor_x}",
                                cy: "{source_anchor_y}",
                                r: "{reconnect_radius}",
                                onpointerdown: on_reconnect_source,
                            }
                        }
                        if allow_reconnect_target {
                            circle {
                                class: "dioxus-flow__edge-reconnect dioxus-flow__edge-reconnect-target",
                                cx: "{target_anchor_x}",
                                cy: "{target_anchor_y}",
                                r: "{reconnect_radius}",
                                onpointerdown: on_reconnect_target,
                            }
                        }
                        if let Some(text) = label {
                            g {
                                class: "dioxus-flow__edge-label",
                                if show_label_bg {
                                    rect {
                                        x: "{label_x - bg_width / 2.0}",
                                        y: "{label_y - bg_height / 2.0}",
                                        rx: "{label_radius}",
                                        ry: "{label_radius}",
                                        width: "{bg_width}",
                                        height: "{bg_height}",
                                        class: "dioxus-flow__edge-label-bg",
                                        style: "{label_bg_style}",
                                    }
                                }
                                text {
                                    x: "{label_x}",
                                    y: "{label_y}",
                                    text_anchor: "middle",
                                    dominant_baseline: "middle",
                                    class: "dioxus-flow__edge-label-text",
                                    style: "{label_style}",
                                    "{text}"
                                }
                            }
                        }
                    }
                }
            }
        })
        .collect();

    rsx! {
        svg {
            class: "dioxus-flow__edges",
            width: "100%",
            height: "100%",
            if !marker_defs.is_empty() {
                defs {
                    for (id, marker) in marker_defs {
                        EdgeMarkerDef { id, marker }
                    }
                }
            }
            for edge_element in edge_elements {
                {edge_element}
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct EdgeRender<E: Clone + PartialEq + Default> {
    edge: Edge<E>,
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
    source_pos: Position,
    target_pos: Position,
}

fn edge_path_for_type<E: Clone + PartialEq + Default>(
    edge: &Edge<E>,
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
    source_position: Position,
    target_position: Position,
) -> crate::types::EdgePathResult {
    match edge.edge_type.as_deref() {
        Some("straight") => get_straight_path(source_x, source_y, target_x, target_y),
        Some("step") => get_step_path(
            source_x,
            source_y,
            target_x,
            target_y,
            source_position,
            target_position,
            None,
        ),
        Some("smoothstep") => get_smooth_step_path(
            source_x,
            source_y,
            target_x,
            target_y,
            source_position,
            target_position,
            None,
            None,
            None,
        ),
        Some("simplebezier") => get_simple_bezier_path(
            source_x,
            source_y,
            target_x,
            target_y,
            source_position,
            target_position,
        ),
        _ => get_bezier_path(
            source_x,
            source_y,
            target_x,
            target_y,
            source_position,
            target_position,
            None,
        ),
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

fn shift_edge_anchor(
    x: f64,
    y: f64,
    position: Position,
    radius: f64,
) -> (f64, f64) {
    if radius <= 0.0 {
        return (x, y);
    }
    match position {
        Position::Left => (x - radius, y),
        Position::Right => (x + radius, y),
        Position::Top => (x, y - radius),
        Position::Bottom => (x, y + radius),
    }
}

fn handle_position_for_edge<N: Clone + PartialEq + Default>(
    node: &crate::types::InternalNode<N>,
    handle_type: HandleType,
    handle_id: Option<&str>,
    fallback_position: Position,
) -> (f64, f64) {
    if let Some(bounds) = &node.handle_bounds {
        if let Some(handle) = select_handle(bounds, handle_type, handle_id) {
            return (
                node.position_absolute.x + handle.x + handle.width / 2.0,
                node.position_absolute.y + handle.y + handle.height / 2.0,
            );
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

fn marker_key(marker: &EdgeMarker) -> String {
    let marker_type = match marker.marker_type {
        MarkerType::Arrow => "arrow",
        MarkerType::ArrowClosed => "arrow-closed",
    };
    let color = marker.color.clone().unwrap_or_default();
    let width = marker.width.unwrap_or(0.0);
    let height = marker.height.unwrap_or(0.0);
    let stroke_width = marker.stroke_width.unwrap_or(0.0);
    format!("{marker_type}:{color}:{width}:{height}:{stroke_width}")
}

fn resolve_marker_id(
    marker: &EdgeMarker,
    ids: &mut HashMap<String, String>,
    defs: &mut Vec<(String, EdgeMarker)>,
) -> String {
    let key = marker_key(marker);
    if let Some(id) = ids.get(&key) {
        return id.clone();
    }
    let id = format!("df-marker-{}", ids.len() + 1);
    ids.insert(key, id.clone());
    defs.push((id.clone(), marker.clone()));
    id
}

fn marker_id_for(marker: &EdgeMarker, ids: &HashMap<String, String>) -> Option<String> {
    let key = marker_key(marker);
    ids.get(&key).cloned()
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

#[component]
fn EdgeMarkerDef(id: String, marker: EdgeMarker) -> Element {
    let (path, view_box) = match marker.marker_type {
        MarkerType::Arrow => ("M0,0 L10,5 L0,10", "0 0 10 10"),
        MarkerType::ArrowClosed => ("M0,0 L10,5 L0,10 z", "0 0 10 10"),
    };
    let color = marker
        .color
        .unwrap_or_else(|| "var(--df-edge-color)".to_string());
    let width = marker.width.unwrap_or(12.0);
    let height = marker.height.unwrap_or(12.0);
    let stroke_width = marker.stroke_width.unwrap_or(1.0);
    rsx! {
        marker {
            id: "{id}",
            marker_width: "{width}",
            marker_height: "{height}",
            ref_x: "10",
            ref_y: "5",
            orient: "auto",
            marker_units: "strokeWidth",
            view_box: "{view_box}",
            path {
                d: "{path}",
                fill: "{color}",
                stroke: "{color}",
                stroke_width: "{stroke_width}",
            }
        }
    }
}
