//! Hooks for Dioxus Flow

mod flow_instance;
mod key_press;

pub use flow_instance::*;
pub use key_press::*;

use crate::state::FlowState;
use crate::types::{Connection, Edge, EdgeChange, Node, NodeChange, Viewport};
use dioxus::prelude::*;
use dioxus::prelude::{ReadableExt, WritableExt};

pub fn use_nodes_state<N>(
    initial_nodes: Vec<Node<N>>,
) -> (Signal<Vec<Node<N>>>, impl FnMut(Vec<NodeChange<N>>))
where
    N: Clone + PartialEq + Default + 'static,
{
    let mut nodes = use_signal(|| initial_nodes);
    let on_nodes_change = move |changes: Vec<NodeChange<N>>| {
        let next = crate::types::apply_node_changes(changes, nodes.read().clone());
        nodes.set(next);
    };
    (nodes, on_nodes_change)
}

pub fn use_dioxus_flow<N, E>() -> FlowInstance<N, E>
where
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
{
    let state = use_context::<FlowState<N, E>>();
    FlowInstance::new(state)
}

pub fn use_store<N, E, R>(selector: impl Fn(&FlowState<N, E>) -> R + 'static) -> R
where
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
    R: Clone + PartialEq + 'static,
{
    use_store_with_equality(selector, |a: &R, b: &R| a == b)
}

pub fn use_store_with_equality<N, E, R>(
    selector: impl Fn(&FlowState<N, E>) -> R + 'static,
    equality: impl Fn(&R, &R) -> bool + 'static,
) -> R
where
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
    R: Clone + 'static,
{
    let state = use_context::<FlowState<N, E>>();
    let mut value = use_signal(|| selector(&state));
    let equality = std::rc::Rc::new(equality);

    use_effect(move || {
        let next = selector(&state);
        let current = value.read().clone();
        if !(equality)(&current, &next) {
            value.set(next);
        }
    });

    let result = {
        let current = value.read();
        current.clone()
    };
    result
}

pub fn use_edges_state<E>(
    initial_edges: Vec<Edge<E>>,
) -> (Signal<Vec<Edge<E>>>, impl FnMut(Vec<EdgeChange<E>>))
where
    E: Clone + PartialEq + Default + 'static,
{
    let mut edges = use_signal(|| initial_edges);
    let on_edges_change = move |changes: Vec<EdgeChange<E>>| {
        let next = crate::types::apply_edge_changes(changes, edges.read().clone());
        edges.set(next);
    };
    (edges, on_edges_change)
}

pub fn use_viewport<N, E>() -> Viewport
where
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
{
    let state = use_context::<FlowState<N, E>>();
    let viewport = *state.viewport.read();
    viewport
}

pub fn use_on_viewport_change<N, E>(callback: impl Fn(Viewport) + 'static)
where
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
{
    let state = use_context::<FlowState<N, E>>();

    use_effect(move || {
        let viewport = *state.viewport.read();
        callback(viewport);
    });
}

pub fn use_nodes_data<N, T>(
    node_ids: impl IntoIterator<Item = String>,
    selector: impl Fn(&Node<N>) -> T + 'static,
) -> Vec<T>
where
    N: Clone + PartialEq + Default + 'static,
    T: Clone + PartialEq + 'static,
{
    let state = use_context::<FlowState<N, ()>>();
    let node_ids: Vec<String> = node_ids.into_iter().collect();

    let memo: Memo<Vec<T>> = use_memo(move || {
        let nodes = state.nodes.read();
        node_ids
            .iter()
            .filter_map(|id| nodes.iter().find(|n| &n.id == id))
            .map(&selector)
            .collect()
    });

    let value = memo.read().clone();
    value
}

pub fn use_handle_connections<N, E>(
    handle_type: crate::types::HandleType,
    handle_id: Option<String>,
    node_id: String,
) -> Vec<Connection>
where
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
{
    let state = use_context::<FlowState<N, E>>();

    let memo: Memo<Vec<Connection>> = use_memo(move || {
        let edges = state.edges.read();
        edges
            .iter()
            .filter(|edge| match handle_type {
                crate::types::HandleType::Source => {
                    edge.source == node_id && edge.source_handle == handle_id
                }
                crate::types::HandleType::Target => {
                    edge.target == node_id && edge.target_handle == handle_id
                }
            })
            .map(|edge| Connection {
                source: edge.source.clone(),
                target: edge.target.clone(),
                source_handle: edge.source_handle.clone(),
                target_handle: edge.target_handle.clone(),
            })
            .collect()
    });

    let value = memo.read().clone();
    value
}

pub fn use_connection<N, E>() -> crate::types::ConnectionState
where
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
{
    let state = use_context::<FlowState<N, E>>();
    let connection = state.connection.read().clone();
    connection
}

#[derive(Clone)]
pub struct SelectionChangeSubscription<N, E>
where
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
{
    id: usize,
    state: FlowState<N, E>,
}

impl<N, E> Drop for SelectionChangeSubscription<N, E>
where
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
{
    fn drop(&mut self) {
        self.state.remove_selection_change_handler(self.id);
    }
}

pub fn use_on_selection_change<N, E>(
    handler: EventHandler<crate::types::SelectionChange<N, E>>,
) -> SelectionChangeSubscription<N, E>
where
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
{
    let mut state = use_context::<FlowState<N, E>>();
    use_hook(move || {
        let id = state.add_selection_change_handler(handler);
        SelectionChangeSubscription { id, state }
    })
}

pub fn use_update_node_internals<N, E>() -> impl FnMut(Vec<String>)
where
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
{
    let mut state = use_context::<FlowState<N, E>>();
    move |node_ids: Vec<String>| {
        state.update_node_internals(node_ids);
    }
}
