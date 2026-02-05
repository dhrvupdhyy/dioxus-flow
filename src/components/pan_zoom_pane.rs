//! Pan and zoom pane component

use crate::state::FlowState;
use crate::types::{
    ConnectionMode, CoordinateExtent, HandleType, NodeExtent, Rect, SelectionMode, Viewport,
    XYPosition,
};
use dioxus::prelude::dioxus_elements::geometry::WheelDelta;
use dioxus::prelude::dioxus_elements::input_data::MouseButton;
use dioxus::prelude::*;
use dioxus::prelude::{
    InteractionLocation, ModifiersInteraction, PointerInteraction, ReadableExt, WritableExt,
};
use dioxus_web::WebEventExt;
use wasm_bindgen::JsCast;
use std::collections::{HashMap, HashSet};
use web_sys::console;

#[derive(Clone, PartialEq)]
struct PinchState {
    start_distance: f64,
    start_viewport: Viewport,
    center: XYPosition,
}

fn pinch_metrics(pointers: &HashMap<i32, XYPosition>) -> Option<(f64, XYPosition)> {
    let mut iter = pointers.values();
    let first = iter.next()?;
    let second = iter.next()?;
    let distance = first.distance_to(second);
    let center = XYPosition::new((first.x + second.x) / 2.0, (first.y + second.y) / 2.0);
    Some((distance, center))
}

#[component]
pub fn PanZoomPane<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    children: Element,
    #[props(default)] on_move: Option<EventHandler<Viewport>>,
    #[props(default)] on_move_start: Option<EventHandler<Viewport>>,
    #[props(default)] on_move_end: Option<EventHandler<Viewport>>,
    #[props(default)] on_nodes_change: Option<EventHandler<Vec<crate::types::NodeChange<N>>>>,
    #[props(default)] on_edges_change: Option<EventHandler<Vec<crate::types::EdgeChange<E>>>>,
    #[props(default)] on_connect: Option<EventHandler<crate::types::Connection>>,
    #[props(default)] on_edge_update: Option<EventHandler<crate::types::EdgeUpdateEvent<E>>>,
    #[props(default)] on_edge_update_end: Option<EventHandler<crate::types::EdgeUpdateEndEvent<E>>>,
    #[props(default)] on_node_drag: Option<EventHandler<crate::types::NodeDragEvent<N>>>,
    #[props(default)] on_node_drag_stop: Option<EventHandler<crate::types::NodeDragEvent<N>>>,
    #[props(default)] on_selection_start: Option<EventHandler<crate::types::SelectionStartEvent>>,
    #[props(default)] on_selection_end: Option<
        EventHandler<crate::types::SelectionEndEvent<N, E>>,
    >,
    #[props(default)] _marker: std::marker::PhantomData<(N, E)>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();
    let mut pan_start = use_signal(|| None::<(f64, f64)>);
    let mut start_viewport = use_signal(|| Viewport::identity());
    let mut selection_start = use_signal(|| None::<XYPosition>);
    let mut selection_additive = use_signal(|| false);
    let mut initialized_size = use_signal(|| false);
    let mut active_pointers = use_signal(HashMap::<i32, XYPosition>::new);
    let mut pinch_state = use_signal(|| None::<PinchState>);
    let mut pane_rect = use_signal(|| None::<web_sys::DomRect>);

    let mut state_size = state.clone();
    use_effect(move || {
        if *initialized_size.read() {
            return;
        }
        if let Some(window) = web_sys::window() {
            if let Ok(width) = window.inner_width() {
                if let Some(width) = width.as_f64() {
                    state_size.width.set(width);
                }
            }
            if let Ok(height) = window.inner_height() {
                if let Some(height) = height.as_f64() {
                    state_size.height.set(height);
                }
            }
        }
        initialized_size.set(true);
    });

    let mut state_down = state.clone();
    let on_pointer_down = move |evt: PointerEvent| {
        if let Some(web_evt) = evt.data.try_as_web_event() {
            if let Some(target) = web_evt.target().and_then(|t| {
                let element: Option<web_sys::Element> = t.dyn_into::<web_sys::Element>().ok();
                element
            }) {
            let no_pan_class = state_down.no_pan_class_name.read().clone();
            if !no_pan_class.is_empty()
                && target.closest(&format!(".{}", no_pan_class)).ok().flatten().is_some()
            {
                return;
            }
            }
        }
        let trigger_button = evt.data.trigger_button();
        let is_primary = trigger_button == Some(MouseButton::Primary);
        let button_code = trigger_button
            .map(|b| match b {
                MouseButton::Primary => 0,
                MouseButton::Auxiliary => 1,
                MouseButton::Secondary => 2,
                MouseButton::Fourth => 3,
                MouseButton::Fifth => 4,
                MouseButton::Unknown => 0,
            })
            .unwrap_or(0);
        let modifiers = evt.data.modifiers();
        if evt.data.pointer_type() == "touch" && *state_down.zoom_on_pinch.read() {
            let pointer_id = evt.data.pointer_id();
            let coords = evt.data.client_coordinates();
            active_pointers
                .write()
                .insert(pointer_id, XYPosition::new(coords.x, coords.y));
            if active_pointers.read().len() == 2 {
                if let Some((distance, center)) = pinch_metrics(&active_pointers.read()) {
                    pinch_state.set(Some(PinchState {
                        start_distance: distance,
                        start_viewport: *state_down.viewport.read(),
                        center,
                    }));
                }
            }
        }
        let coords = evt.data.client_coordinates();
        let flow_pos = state_down.screen_to_flow_position(XYPosition::new(coords.x, coords.y));

        let selection_enabled = is_primary
            && (*state_down.selection_on_drag.read()
            || *state_down.selection_key_pressed.read()
            || modifiers.shift())
            && !*state_down.pan_activation_key_pressed.read();
        if selection_enabled && *state_down.elements_selectable.read() {
            selection_start.set(Some(flow_pos));
            selection_additive.set(
                *state_down.multi_selection_key_pressed.read()
                    || modifiers.shift()
                    || modifiers.meta()
                    || modifiers.ctrl(),
            );
            state_down.user_selection_active.set(true);
            state_down.user_selection_rect.set(Some(Rect {
                x: flow_pos.x,
                y: flow_pos.y,
                width: 0.0,
                height: 0.0,
            }));
            state_down
                .multi_selection_active
                .set(*selection_additive.read());
            if let Some(handler) = &on_selection_start {
                handler.call(crate::types::SelectionStartEvent {
                    position: flow_pos,
                });
            }
            if *state_down.debug.read() {
                console::log_1(&"selection start".into());
            }
            return;
        }

        if is_primary && *state_down.elements_selectable.read() {
            state_down.deselect_all();
        }

        let mut allow_pan = *state_down.pan_on_drag.read()
            || *state_down.pan_activation_key_pressed.read();
        if let Some(buttons) = state_down.pan_on_drag_buttons.read().clone() {
            allow_pan = allow_pan && buttons.contains(&button_code);
        }

        if !allow_pan {
            return;
        }

        let viewport = *state_down.viewport.read();
        start_viewport.set(viewport);
        pan_start.set(Some((coords.x, coords.y)));
        state_down.panning.set(true);
        if let Some(handler) = &on_move_start {
            handler.call(viewport);
        }
    };

    let mut state_move = state.clone();
    let on_pointer_move = move |evt: PointerEvent| {
        if evt.data.pointer_type() == "touch" && *state_move.zoom_on_pinch.read() {
            let pointer_id = evt.data.pointer_id();
            if active_pointers.read().contains_key(&pointer_id) {
                let coords = evt.data.client_coordinates();
                active_pointers
                    .write()
                    .insert(pointer_id, XYPosition::new(coords.x, coords.y));

                if active_pointers.read().len() == 2 {
                    if pinch_state.read().is_none() {
                        if let Some((distance, center)) = pinch_metrics(&active_pointers.read()) {
                            pinch_state.set(Some(PinchState {
                                start_distance: distance,
                                start_viewport: *state_move.viewport.read(),
                                center,
                            }));
                        }
                    }

                    if let Some(pinch) = pinch_state.read().clone() {
                        if let Some((distance, _center)) = pinch_metrics(&active_pointers.read()) {
                            let min_zoom = *state_move.min_zoom.read();
                            let max_zoom = *state_move.max_zoom.read();
                            let zoom = (pinch.start_viewport.zoom * distance
                                / pinch.start_distance)
                                .clamp(min_zoom, max_zoom);
                            let flow_x = (pinch.center.x - pinch.start_viewport.x)
                                / pinch.start_viewport.zoom;
                            let flow_y = (pinch.center.y - pinch.start_viewport.y)
                                / pinch.start_viewport.zoom;
                            let next = Viewport {
                                x: pinch.center.x - flow_x * zoom,
                                y: pinch.center.y - flow_y * zoom,
                                zoom,
                            };
                            let clamped = state_move.clamp_viewport(next);
                            state_move.set_viewport(clamped, None);
                            refresh_connection_position(&mut state_move);
                        }
                    }
                }
                return;
            }
        }

        if state_move.connection.read().in_progress {
            let coords = evt.data.client_coordinates();
            let screen_pos = XYPosition::new(coords.x, coords.y);
            let flow_pos = state_move.screen_to_flow_position(screen_pos);
            let mut connection = state_move.connection.read().clone();
            let threshold = *state_move.connection_drag_threshold.read();
            if !connection.dragging {
                if let Some(start) = connection.start_screen {
                    if start.distance_to(&screen_pos) < threshold {
                        return;
                    }
                }
                connection.dragging = true;
            }
            update_connection_target(&mut state_move, &mut connection, screen_pos, flow_pos);
            state_move.connection.set(connection);
            if *state_move.auto_pan_on_connect.read() {
                if let Some(rect) = pane_rect.read().as_ref() {
                    auto_pan_if_needed(&mut state_move, screen_pos, rect);
                }
            }
            return;
        }

        let drag_state = state_move.node_drag.read().clone();
        if let Some(mut drag_state) = drag_state {
            let coords = evt.data.client_coordinates();
            let flow_pos = state_move.screen_to_flow_position(XYPosition::new(coords.x, coords.y));
            let delta = XYPosition {
                x: flow_pos.x - drag_state.start_pointer.x,
                y: flow_pos.y - drag_state.start_pointer.y,
            };
            let threshold = *state_move.node_drag_threshold.read();
            if !drag_state.started && delta.distance_to(&XYPosition::new(0.0, 0.0)) < threshold {
                return;
            }
            if !drag_state.started {
                drag_state.started = true;
                state_move.node_drag.set(Some(drag_state.clone()));
            }
            let mut changes = Vec::new();
            let snap = *state_move.snap_to_grid.read();
            let grid = *state_move.snap_grid.read();
            {
                let node_lookup = state_move.node_lookup.read();
                for (node_id, start_pos) in drag_state.nodes.iter() {
                    let mut next = XYPosition {
                        x: start_pos.x + delta.x,
                        y: start_pos.y + delta.y,
                    };
                    if snap {
                        next.x = (next.x / grid.0).round() * grid.0;
                        next.y = (next.y / grid.1).round() * grid.1;
                    }
                    if let Some(internal) = node_lookup.get(node_id) {
                        next = clamp_node_position(&state_move, internal, next);
                    }
                    changes.push(crate::types::NodeChange::Position {
                        id: node_id.clone(),
                        position: Some(next),
                        dragging: true,
                    });
                }
            }

            let next_nodes =
                apply_node_changes_with_next(&mut state_move, &on_nodes_change, changes);
            if let Some(handler) = &on_node_drag {
                if let Some(origin_id) = state_move
                    .node_drag
                    .read()
                    .as_ref()
                    .map(|drag| drag.origin_node_id.clone())
                {
                    if let Some(origin) = next_nodes.iter().find(|n| n.id == origin_id).cloned() {
                        handler.call(crate::types::NodeDragEvent {
                            node: origin,
                            nodes: next_nodes,
                        });
                    }
                }
            }
            if *state_move.auto_pan_on_node_drag.read() {
                let screen_pos = XYPosition::new(coords.x, coords.y);
                if let Some(rect) = pane_rect.read().as_ref() {
                    auto_pan_if_needed(&mut state_move, screen_pos, rect);
                }
            }
            return;
        }

        if let Some(start) = *selection_start.read() {
            let coords = evt.data.client_coordinates();
            let flow_pos = state_move.screen_to_flow_position(XYPosition::new(coords.x, coords.y));
            let (min_x, max_x) = if flow_pos.x < start.x {
                (flow_pos.x, start.x)
            } else {
                (start.x, flow_pos.x)
            };
            let (min_y, max_y) = if flow_pos.y < start.y {
                (flow_pos.y, start.y)
            } else {
                (start.y, flow_pos.y)
            };
            state_move.user_selection_rect.set(Some(Rect {
                x: min_x,
                y: min_y,
                width: max_x - min_x,
                height: max_y - min_y,
            }));
            return;
        }

        if let Some((sx, sy)) = *pan_start.read() {
            let coords = evt.data.client_coordinates();
            let dx = coords.x - sx;
            let dy = coords.y - sy;
            let base = *start_viewport.read();
            let next = Viewport {
                x: base.x + dx,
                y: base.y + dy,
                zoom: base.zoom,
            };
            state_move.set_viewport(next, None);
            refresh_connection_position(&mut state_move);
            if let Some(handler) = &on_move {
                handler.call(next);
            }
        }
    };

    let mut state_up = state.clone();
    let mut pan_start_up = pan_start.clone();
    let mut selection_start_up = selection_start.clone();
    let mut selection_additive_up = selection_additive.clone();
    let on_pointer_up = move |evt: PointerEvent| {
        if evt.data.pointer_type() == "touch" {
            let pointer_id = evt.data.pointer_id();
            active_pointers.write().remove(&pointer_id);
            if active_pointers.read().len() < 2 {
                pinch_state.set(None);
            }
        }
        end_interaction(
            &mut state_up,
            &on_nodes_change,
            &on_edges_change,
            &on_move_end,
            &on_connect,
            &on_edge_update,
            &on_edge_update_end,
            &on_node_drag_stop,
            &on_selection_end,
            &mut pan_start_up,
            &mut selection_start_up,
            &mut selection_additive_up,
        );
    };

    let mut state_leave = state.clone();
    let mut pan_start_leave = pan_start.clone();
    let mut selection_start_leave = selection_start.clone();
    let mut selection_additive_leave = selection_additive.clone();
    let on_pointer_leave = move |evt: PointerEvent| {
        if evt.data.pointer_type() == "touch" {
            let pointer_id = evt.data.pointer_id();
            active_pointers.write().remove(&pointer_id);
            if active_pointers.read().len() < 2 {
                pinch_state.set(None);
            }
        }
        end_interaction(
            &mut state_leave,
            &on_nodes_change,
            &on_edges_change,
            &on_move_end,
            &on_connect,
            &on_edge_update,
            &on_edge_update_end,
            &on_node_drag_stop,
            &on_selection_end,
            &mut pan_start_leave,
            &mut selection_start_leave,
            &mut selection_additive_leave,
        );
    };

    let mut state_wheel = state.clone();
    let on_wheel = move |evt: WheelEvent| {
        if let Some(web_evt) = evt.data.try_as_web_event() {
            if let Some(target) = web_evt.target().and_then(|t| {
                let element: Option<web_sys::Element> = t.dyn_into::<web_sys::Element>().ok();
                element
            }) {
                let no_wheel_class = state_wheel.no_wheel_class_name.read().clone();
                if !no_wheel_class.is_empty()
                    && target.closest(&format!(".{}", no_wheel_class)).ok().flatten().is_some()
                {
                    if evt.data.modifiers().ctrl() {
                        evt.prevent_default();
                    }
                    return;
                }
            }
        }
        let (delta_x, delta_y) = match evt.data.delta() {
            WheelDelta::Pixels(v) => (v.x, v.y),
            WheelDelta::Lines(v) => (v.x * 16.0, v.y * 16.0),
            WheelDelta::Pages(v) => {
                let page = (*state_wheel.height.read()).max(1.0);
                (v.x * page, v.y * page)
            }
        };
        if delta_x == 0.0 && delta_y == 0.0 {
            return;
        }
        let coords = evt.data.client_coordinates();
        let modifiers = evt.data.modifiers();

        let zoom_key = *state_wheel.zoom_activation_key_pressed.read();
        let pan_key = *state_wheel.pan_activation_key_pressed.read();
        let zoom_on_scroll = *state_wheel.zoom_on_scroll.read() || zoom_key;
        let is_pan_on_scroll = *state_wheel.pan_on_scroll.read()
            && !zoom_key
            && !*state_wheel.user_selection_active.read();

        if *state_wheel.prevent_scrolling.read() || zoom_on_scroll || is_pan_on_scroll || pan_key {
            evt.prevent_default();
        } else if !modifiers.ctrl() {
            return;
        }

        if is_pan_on_scroll && !zoom_on_scroll {
            let zoom = state_wheel.viewport.read().zoom.max(0.0001);
            let speed = *state_wheel.pan_on_scroll_speed.read();
            let (mut dx, mut dy) = (-delta_x / zoom * speed, -delta_y / zoom * speed);
            match *state_wheel.pan_on_scroll_mode.read() {
                crate::types::PanOnScrollMode::Horizontal => dy = 0.0,
                crate::types::PanOnScrollMode::Vertical => dx = 0.0,
                crate::types::PanOnScrollMode::Free => {}
            }
            state_wheel.pan_by(XYPosition { x: dx, y: dy });
            refresh_connection_position(&mut state_wheel);
            let viewport = *state_wheel.viewport.read();
            if let Some(handler) = &on_move {
                handler.call(viewport);
            }
            return;
        }

        if !zoom_on_scroll {
            return;
        }

        if modifiers.ctrl() && !*state_wheel.zoom_on_pinch.read() {
            return;
        }
        let base: f64 = if modifiers.ctrl() { 1.001 } else { 1.002 };
        let smooth_factor = base.powf(-delta_y).clamp(0.1, 10.0);
        zoom_at_point(&mut state_wheel, coords.x, coords.y, smooth_factor);
        refresh_connection_position(&mut state_wheel);
        let viewport = *state_wheel.viewport.read();
        if let Some(handler) = &on_move {
            handler.call(viewport);
        }
    };

    let mut state_double = state.clone();
    let on_double_click = move |evt: MouseEvent| {
        if !*state_double.zoom_on_double_click.read() {
            return;
        }
        let coords = evt.data.client_coordinates();
        zoom_at_point(&mut state_double, coords.x, coords.y, 1.2);
        let viewport = *state_double.viewport.read();
        if let Some(handler) = &on_move {
            handler.call(viewport);
        }
    };

    rsx! {
        div {
            class: "dioxus-flow__panzoom",
            onpointerdown: on_pointer_down,
            onpointermove: on_pointer_move,
            onpointerup: on_pointer_up,
            onpointerleave: on_pointer_leave,
            onwheel: on_wheel,
            ondoubleclick: on_double_click,
            onmounted: move |evt| {
                let element: web_sys::Element = evt.as_web_event();
                let rect = element.get_bounding_client_rect();
                pane_rect.set(Some(rect));
            },
            {children}
        }
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

fn apply_node_changes_with_next<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &mut FlowState<N, E>,
    handler: &Option<EventHandler<Vec<crate::types::NodeChange<N>>>>,
    changes: Vec<crate::types::NodeChange<N>>,
) -> Vec<crate::types::Node<N>> {
    if changes.is_empty() {
        return state.nodes.read().clone();
    }

    let current = state.nodes.read().clone();
    let next_nodes = crate::types::apply_node_changes(changes.clone(), current);

    if let Some(handler) = handler {
        handler.call(changes);
    } else {
        state.set_nodes(next_nodes.clone());
    }

    next_nodes
}

fn end_interaction<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &mut FlowState<N, E>,
    on_nodes_change: &Option<EventHandler<Vec<crate::types::NodeChange<N>>>>,
    on_edges_change: &Option<EventHandler<Vec<crate::types::EdgeChange<E>>>>,
    on_move_end: &Option<EventHandler<Viewport>>,
    on_connect: &Option<EventHandler<crate::types::Connection>>,
    on_edge_update: &Option<EventHandler<crate::types::EdgeUpdateEvent<E>>>,
    on_edge_update_end: &Option<EventHandler<crate::types::EdgeUpdateEndEvent<E>>>,
    on_node_drag_stop: &Option<EventHandler<crate::types::NodeDragEvent<N>>>,
    on_selection_end: &Option<EventHandler<crate::types::SelectionEndEvent<N, E>>>,
    pan_start: &mut Signal<Option<(f64, f64)>>,
    selection_start: &mut Signal<Option<XYPosition>>,
    selection_additive: &mut Signal<bool>,
) {
    if state.connection.read().in_progress {
        let mut connection = state.connection.read().clone();
        let reconnect_edge = connection.reconnect_edge_id.clone();
        let reconnect_end = connection.reconnect_end;
        let result = if connection.is_valid {
            connection.to_connection()
        } else {
            None
        };
        let edge_before = reconnect_edge
            .as_ref()
            .and_then(|id| state.edge_lookup.read().get(id).cloned());
        if let Some(handler) = state.on_connect_end.read().clone() {
            handler.call(crate::types::ConnectionEndEvent {
                connection: result.clone(),
                is_valid: connection.is_valid,
            });
        }
        connection.reset();
        state.connection.set(connection);

        if let Some(conn) = result {
            if let Some(edge_id) = reconnect_edge {
                if let Some(end) = reconnect_end {
                    let mut edges = state.edges.read().clone();
                    if let Some(edge) = edges.iter_mut().find(|e| e.id == edge_id) {
                        let old_edge = edge_before.clone().unwrap_or_else(|| edge.clone());
                        match end {
                            HandleType::Source => {
                                edge.source = conn.source.clone();
                                edge.source_handle = conn.source_handle.clone();
                            }
                            HandleType::Target => {
                                edge.target = conn.target.clone();
                                edge.target_handle = conn.target_handle.clone();
                            }
                        }
                        let updated = edge.clone();
                        if let Some(handler) = on_edge_update {
                            handler.call(crate::types::EdgeUpdateEvent {
                                edge: old_edge.clone(),
                                connection: conn.clone(),
                            });
                        }
                        apply_edge_changes(
                            state,
                            on_edges_change,
                            vec![crate::types::EdgeChange::Replace {
                                id: edge_id.clone(),
                                edge: updated,
                            }],
                        );
                        if let Some(handler) = on_edge_update_end {
                            handler.call(crate::types::EdgeUpdateEndEvent {
                                edge: old_edge,
                                connection: Some(conn),
                            });
                        }
                    }
                }
            } else if let Some(handler) = on_connect {
                handler.call(conn);
            } else {
                let edge = crate::state::connection_to_edge::<E>(&conn, None);
                state.apply_edge_changes(vec![crate::types::EdgeChange::Add { edge }]);
            }
        } else if let Some(edge_before) = edge_before {
            if let Some(handler) = on_edge_update_end {
                handler.call(crate::types::EdgeUpdateEndEvent {
                    edge: edge_before,
                    connection: None,
                });
            }
        }
    }
    let drag_state = state.node_drag.read().clone();
    let pending_click = state.pending_node_click.read().as_ref().cloned();
    if let Some(pending) = pending_click {
        let allow_apply = match &drag_state {
            Some(drag) => !drag.started,
            None => true,
        };
        if allow_apply {
            let nodes = state.nodes.read().clone();
            let mut changes = Vec::new();

            if pending.multi {
                if let Some(node) = nodes.iter().find(|n| n.id == pending.node_id) {
                    let next = !node.selected;
                    changes.push(crate::types::NodeChange::Selection {
                        id: node.id.clone(),
                        selected: next,
                    });
                }
            } else {
                for node in nodes.iter() {
                    let should_select = node.id == pending.node_id;
                    if node.selected != should_select {
                        changes.push(crate::types::NodeChange::Selection {
                            id: node.id.clone(),
                            selected: should_select,
                        });
                    }
                }
                let edge_changes: Vec<_> = state
                    .edges
                    .read()
                    .iter()
                    .filter(|edge| edge.selected)
                    .map(|edge| crate::types::EdgeChange::Selection {
                        id: edge.id.clone(),
                        selected: false,
                    })
                    .collect();
                apply_edge_changes(state, on_edges_change, edge_changes);
            }
            apply_node_changes(state, on_nodes_change, changes);
        }
        state.pending_node_click.set(None);
    }
    if let Some(drag_state) = drag_state {
        let mut changes = Vec::new();
        for (node_id, _) in drag_state.nodes.iter() {
            changes.push(crate::types::NodeChange::Position {
                id: node_id.clone(),
                position: None,
                dragging: false,
            });
        }
        let next_nodes = apply_node_changes_with_next(state, on_nodes_change, changes);
        if let Some(handler) = on_node_drag_stop {
            if let Some(origin) = next_nodes
                .iter()
                .find(|n| n.id == drag_state.origin_node_id)
                .cloned()
            {
                handler.call(crate::types::NodeDragEvent {
                    node: origin,
                    nodes: next_nodes,
                });
            }
        }
        state.node_drag.set(None);
        return;
    }

    if selection_start.read().is_some() {
        let selection = state.user_selection_rect.read().clone();
        let nodes = state.nodes.read().clone();
        if let Some(rect) = selection {
            let selection_mode = *state.selection_mode.read();
            let selected_ids = {
                let internal_lookup = state.node_lookup.read();
                let mut selected_ids = HashSet::new();
                for node in nodes.iter() {
                    if node.hidden || !node.selectable.unwrap_or(true) {
                        continue;
                    }
                    let internal = internal_lookup.get(&node.id);
                    let dims = internal
                        .map(|i| i.dimensions)
                        .unwrap_or_else(|| node.get_dimensions());
                    let position = internal
                        .map(|i| i.position_absolute)
                        .unwrap_or(node.position);
                    let node_rect = Rect {
                        x: position.x,
                        y: position.y,
                        width: dims.width,
                        height: dims.height,
                    };
                    let is_selected = match selection_mode {
                        SelectionMode::Full => rect.contains_rect(&node_rect),
                        SelectionMode::Partial => rect.intersects(&node_rect),
                    };
                    if is_selected {
                        selected_ids.insert(node.id.clone());
                    }
                }
                selected_ids
            };

            let additive = *selection_additive.read();
            let mut changes = Vec::new();
            for node in nodes.iter() {
                let mut should_select = selected_ids.contains(&node.id);
                if additive && node.selected {
                    should_select = true;
                }
                if node.selected != should_select {
                    changes.push(crate::types::NodeChange::Selection {
                        id: node.id.clone(),
                        selected: should_select,
                    });
                }
            }
            let next_nodes_for_event = if on_selection_end.is_some() {
                Some(crate::types::apply_node_changes(changes.clone(), nodes.clone()))
            } else {
                None
            };
            apply_node_changes(state, on_nodes_change, changes);
            if let Some(handler) = on_selection_end {
                let next_nodes = next_nodes_for_event.unwrap_or_else(|| state.nodes.read().clone());
                let selected_nodes = next_nodes
                    .iter()
                    .filter(|n| n.selected)
                    .cloned()
                    .collect();
                let selected_edges = state
                    .edges
                    .read()
                    .iter()
                    .filter(|e| e.selected)
                    .cloned()
                    .collect();
                handler.call(crate::types::SelectionEndEvent {
                    selection_rect: Some(rect),
                    nodes: selected_nodes,
                    edges: selected_edges,
                });
            }
            if *state.debug.read() {
                console::log_1(&"selection end".into());
            }
        }

        state.user_selection_active.set(false);
        state.user_selection_rect.set(None);
        selection_start.set(None);
        selection_additive.set(false);
    }

    if *state.panning.read() {
        state.panning.set(false);
        pan_start.set(None);
        let viewport = *state.viewport.read();
        if let Some(handler) = on_move_end {
            handler.call(viewport);
        }
    }
}

fn clamp_node_position<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &FlowState<N, E>,
    internal: &crate::types::InternalNode<N>,
    next_position: XYPosition,
) -> XYPosition {
    let dims = internal.node.get_dimensions();
    let extent = internal.node.extent.clone().or_else(|| {
        state
            .node_extent
            .read()
            .as_ref()
            .map(|extent| NodeExtent::CoordinateExtent(*extent))
    });

    match extent {
        Some(NodeExtent::Parent) => {
            if let Some(parent_id) = &internal.node.parent_id {
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
            let parent_abs = if let Some(parent_id) = internal.node.parent_id.as_ref() {
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

fn zoom_at_point<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &mut FlowState<N, E>,
    screen_x: f64,
    screen_y: f64,
    factor: f64,
) {
    let viewport = *state.viewport.read();
    let min_zoom = *state.min_zoom.read();
    let max_zoom = *state.max_zoom.read();
    let next_zoom = (viewport.zoom * factor).clamp(min_zoom, max_zoom);
    if (next_zoom - viewport.zoom).abs() < f64::EPSILON {
        return;
    }

    let scale = next_zoom / viewport.zoom;
    let next_x = screen_x - (screen_x - viewport.x) * scale;
    let next_y = screen_y - (screen_y - viewport.y) * scale;

    let clamped = state.clamp_viewport(Viewport {
        x: next_x,
        y: next_y,
        zoom: next_zoom,
    });
    state.set_viewport(clamped, None);
}

#[derive(Clone)]
struct ClosestHandle {
    node_id: String,
    handle_id: Option<String>,
    handle_type: HandleType,
    flow_pos: XYPosition,
    screen_pos: XYPosition,
    distance: f64,
}

fn update_connection_target<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &mut FlowState<N, E>,
    connection: &mut crate::types::ConnectionState,
    screen_pos: XYPosition,
    flow_pos: XYPosition,
) {
    let candidate = find_closest_handle(state, connection, screen_pos);
    if let Some(target) = candidate {
        let base_valid = match (*state.connection_mode.read(), connection.from_type) {
            (ConnectionMode::Strict, Some(from_type)) => from_type != target.handle_type,
            (ConnectionMode::Loose, _) => true,
            _ => false,
        };

        connection.set_target(
            target.node_id,
            target.handle_id,
            target.handle_type,
            base_valid,
        );

        let mut is_valid = base_valid;
        if base_valid {
            if let Some(conn) = connection.to_connection() {
                if let Some(validator) = *state.is_valid_connection.read() {
                    is_valid = validator(&conn);
                }
            } else {
                is_valid = false;
            }
        }
        connection.is_valid = is_valid;
        connection.update_screen_position(target.screen_pos, target.flow_pos);
    } else {
        connection.clear_target();
        connection.update_screen_position(screen_pos, flow_pos);
    }
}

fn find_closest_handle<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &FlowState<N, E>,
    connection: &crate::types::ConnectionState,
    screen_pos: XYPosition,
) -> Option<ClosestHandle> {
    let radius = *state.connection_radius.read();
    if radius <= 0.0 {
        return None;
    }
    let from_node = connection.from_node.as_ref()?;
    let from_handle = connection.from_handle.as_ref();
    let from_type = connection.from_type?;
    let mode = *state.connection_mode.read();

    let node_lookup = state.node_lookup.read();
    let mut best: Option<ClosestHandle> = None;

    for (node_id, internal) in node_lookup.iter() {
        if internal.node.hidden {
            continue;
        }
        let Some(bounds) = &internal.handle_bounds else {
            continue;
        };

        for handle_type in [HandleType::Source, HandleType::Target] {
            if mode == ConnectionMode::Strict && handle_type == from_type {
                continue;
            }
            let handles = match handle_type {
                HandleType::Source => &bounds.source,
                HandleType::Target => &bounds.target,
            };
            for handle in handles {
                if !handle.is_connectable {
                    continue;
                }
                if node_id == from_node
                    && handle_type == from_type
                    && handle.id.as_ref() == from_handle
                {
                    continue;
                }
                let flow_pos = XYPosition::new(
                    internal.position_absolute.x + handle.x + handle.width / 2.0,
                    internal.position_absolute.y + handle.y + handle.height / 2.0,
                );
                let handle_screen = state.flow_to_screen_position(flow_pos);
                let distance = handle_screen.distance_to(&screen_pos);
                if distance <= radius {
                    let candidate = ClosestHandle {
                        node_id: node_id.clone(),
                        handle_id: handle.id.clone(),
                        handle_type,
                        flow_pos,
                        screen_pos: handle_screen,
                        distance,
                    };
                    match &best {
                        Some(best_value) if best_value.distance <= distance => {}
                        _ => best = Some(candidate),
                    }
                }
            }
        }
    }

    best
}

fn refresh_connection_position<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &mut FlowState<N, E>,
) {
    state.refresh_connection_position();
}

fn auto_pan_if_needed<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &mut FlowState<N, E>,
    screen_pos: XYPosition,
    rect: &web_sys::DomRect,
) {
    let margin = 40.0;
    let speed = *state.auto_pan_speed.read();
    let mut dx = 0.0;
    let mut dy = 0.0;

    if screen_pos.x - rect.x() < margin {
        dx = speed;
    } else if rect.x() + rect.width() - screen_pos.x < margin {
        dx = -speed;
    }

    if screen_pos.y - rect.y() < margin {
        dy = speed;
    } else if rect.y() + rect.height() - screen_pos.y < margin {
        dy = -speed;
    }

    if dx != 0.0 || dy != 0.0 {
        state.pan_by(XYPosition { x: dx, y: dy });
    }
}
