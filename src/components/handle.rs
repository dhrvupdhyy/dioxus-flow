//! Handle component

use crate::state::FlowState;
use crate::types::{ConnectionMode, HandleType, Position, XYPosition};
use dioxus::prelude::dioxus_elements::input_data::MouseButton;
use dioxus::prelude::*;
use dioxus::prelude::{PointerInteraction, ReadableExt, WritableExt};
use web_sys::console;

#[component]
pub fn Handle<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    node_id: String,
    handle_type: HandleType,
    position: Position,
    #[props(default)] id: Option<String>,
    #[props(default = true)] is_connectable: bool,
    #[props(default)] _marker: std::marker::PhantomData<(N, E)>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();
    let position_class = match position {
        Position::Left => "dioxus-flow__handle-left",
        Position::Right => "dioxus-flow__handle-right",
        Position::Top => "dioxus-flow__handle-top",
        Position::Bottom => "dioxus-flow__handle-bottom",
    };

    let handle_type_class = match handle_type {
        HandleType::Source => "dioxus-flow__handle-source",
        HandleType::Target => "dioxus-flow__handle-target",
    };

    let mut class = format!(
        "dioxus-flow__handle {} {}",
        position_class, handle_type_class
    );
    if is_connectable {
        class.push_str(" connectable connectablestart connectableend");
    }

    let connection = state.connection.read().clone();
    if connection.in_progress {
        class.push_str(" connecting");
        if connection.from_node.as_deref() == Some(&node_id)
            && connection.from_handle.as_deref() == id.as_deref()
            && connection.from_type == Some(handle_type)
        {
            class.push_str(" connectingfrom");
        }
        if connection.to_node.as_deref() == Some(&node_id)
            && connection.to_handle.as_deref() == id.as_deref()
            && connection.to_type == Some(handle_type)
        {
            class.push_str(" connectingto");
            if connection.is_valid {
                class.push_str(" valid");
            } else {
                class.push_str(" invalid");
            }
        }
        if connection.from_type.is_some() {
            let is_possible_end = match *state.connection_mode.read() {
                ConnectionMode::Strict => connection.from_type != Some(handle_type),
                ConnectionMode::Loose => {
                    connection.from_node.as_deref() != Some(&node_id)
                        || connection.from_handle.as_deref() != id.as_deref()
                }
            };
            if is_possible_end {
                class.push_str(" connectionindicator");
            }
        }
    } else if is_connectable {
        class.push_str(" connectionindicator");
    }

    let node_id_attr = node_id.clone();
    let handle_id_attr = id.clone();
    let node_id_down = node_id.clone();
    let handle_id_down = id.clone();
    let mut state_down = state.clone();
    let on_pointer_down = move |evt: PointerEvent| {
        if !is_connectable || !*state_down.nodes_connectable.read() {
            return;
        }
        if evt.data.trigger_button() != Some(MouseButton::Primary) {
            return;
        }
        evt.stop_propagation();

        let mut connection = state_down.connection.read().clone();
        if connection.in_progress && *state_down.connect_on_click.read() {
            let base_valid = match *state_down.connection_mode.read() {
                ConnectionMode::Strict => match connection.from_type {
                    Some(from_type) => from_type != handle_type,
                    None => false,
                },
                ConnectionMode::Loose => true,
            };
            connection.set_target(
                node_id_down.clone(),
                handle_id_down.clone(),
                handle_type,
                base_valid,
            );
            let is_valid = if base_valid {
                if let Some(conn) = connection.to_connection() {
                    if let Some(validator) = *state_down.is_valid_connection.read() {
                        validator(&conn)
                    } else {
                        true
                    }
                } else {
                    false
                }
            } else {
                false
            };
        connection.is_valid = is_valid;
            if let Some(flow_pos) = resolve_handle_flow_position(
                &state_down,
                &node_id_down,
                handle_type,
                handle_id_down.as_deref(),
                position,
            ) {
                let screen_pos = state_down.flow_to_screen_position(flow_pos);
                connection.update_screen_position(screen_pos, flow_pos);
            }
            state_down.connection.set(connection);
            return;
        }
        let coords = evt.data.client_coordinates();
        let mut connection = crate::types::ConnectionState::start(
            node_id_down.clone(),
            handle_id_down.clone(),
            handle_type,
            position,
        );
        connection.start_screen = Some(XYPosition::new(coords.x, coords.y));
        state_down.connection.set(connection);
        if *state_down.debug.read() {
            console::log_1(
                &format!("connect start: {} {:?}", node_id_down, handle_id_down).into(),
            );
        }
        if let Some(handler) = state_down.on_connect_start.read().clone() {
            handler.call(crate::types::ConnectionStartEvent {
                node_id: node_id_down.clone(),
                handle_id: handle_id_down.clone(),
                handle_type,
                position,
            });
        }
    };

    let node_id_enter = node_id.clone();
    let handle_id_enter = id.clone();
    let mut state_enter = state.clone();
    let on_pointer_enter = move |_evt: PointerEvent| {
        if !is_connectable || !*state_enter.nodes_connectable.read() {
            return;
        }
        let mut connection = state_enter.connection.read().clone();
        if !connection.in_progress {
            return;
        }
        let base_valid = match *state_enter.connection_mode.read() {
            ConnectionMode::Strict => match connection.from_type {
                Some(from_type) => from_type != handle_type,
                None => false,
            },
            ConnectionMode::Loose => true,
        };
        connection.set_target(
            node_id_enter.clone(),
            handle_id_enter.clone(),
            handle_type,
            base_valid,
        );
        let is_valid = if base_valid {
            if let Some(conn) = connection.to_connection() {
                if let Some(validator) = *state_enter.is_valid_connection.read() {
                    validator(&conn)
                } else {
                    true
                }
            } else {
                false
            }
        } else {
            false
        };
        connection.is_valid = is_valid;
        if let Some(flow_pos) = resolve_handle_flow_position(
            &state_enter,
            &node_id_enter,
            handle_type,
            handle_id_enter.as_deref(),
            position,
        ) {
            let screen_pos = state_enter.flow_to_screen_position(flow_pos);
            connection.update_screen_position(screen_pos, flow_pos);
        }
        state_enter.connection.set(connection);
    };

    let mut state_leave = state.clone();
    let on_pointer_leave = move |_evt: PointerEvent| {
        let mut connection = state_leave.connection.read().clone();
        if !connection.in_progress {
            return;
        }
        connection.clear_target();
        state_leave.connection.set(connection);
    };

    let mut state_up = state.clone();
    let on_pointer_up = move |_evt: PointerEvent| {
        let mut connection = state_up.connection.read().clone();
        if let Some(_conn) = connection.end() {
            state_up.connection.set(connection);
            // emit connect event if provided via state? handled in PanZoomPane end
        } else {
            state_up.connection.set(connection);
        }
    };

    let aria_label = match handle_type {
        HandleType::Source => "source handle",
        HandleType::Target => "target handle",
    };

    rsx! {
        div {
            class: "{class}",
            "data-node-id": "{node_id_attr}",
            "data-handle-id": "{handle_id_attr.clone().unwrap_or_default()}",
            "data-handle-pos": "{position:?}",
            "aria-label": "{aria_label}",
            onpointerdown: on_pointer_down,
            onpointerenter: on_pointer_enter,
            onpointerleave: on_pointer_leave,
            onpointerup: on_pointer_up,
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

fn resolve_handle_flow_position<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &FlowState<N, E>,
    node_id: &str,
    handle_type: HandleType,
    handle_id: Option<&str>,
    fallback_position: Position,
) -> Option<XYPosition> {
    let lookup = state.node_lookup.read();
    let internal = lookup.get(node_id)?;
    if let Some(bounds) = &internal.handle_bounds {
        let handles = match handle_type {
            HandleType::Source => &bounds.source,
            HandleType::Target => &bounds.target,
        };
        let handle = if let Some(id) = handle_id {
            handles.iter().find(|handle| handle.id.as_deref() == Some(id))
        } else {
            handles.first()
        };
        if let Some(handle) = handle {
            return Some(XYPosition::new(
                internal.position_absolute.x + handle.x + handle.width / 2.0,
                internal.position_absolute.y + handle.y + handle.height / 2.0,
            ));
        }
    }
    let (x, y) = node_handle_position_internal(internal, fallback_position);
    Some(XYPosition::new(x, y))
}
