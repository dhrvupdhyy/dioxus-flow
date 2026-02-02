//! Flow provider component

use crate::state::FlowState;
use crate::types::{Edge, Node};
use dioxus::prelude::*;

#[component]
pub fn FlowProvider<N, E>(
    children: Element,
    #[props(default)] initial_nodes: Vec<Node<N>>,
    #[props(default)] initial_edges: Vec<Edge<E>>,
) -> Element
where
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
{
    let mut state = use_context_provider(|| FlowState::<N, E>::new());
    let mut initialized = use_signal(|| false);

    use_effect(move || {
        if *initialized.read() {
            return;
        }
        state.init(initial_nodes.clone(), initial_edges.clone());
        initialized.set(true);
    });

    children
}
