//! Built-in node components

use crate::components::Handle;
use crate::types::{HandleType, Position};
use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn DefaultNode<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    node: crate::components::NodeProps<N, E>,
) -> Element {
    rsx! {
        div {
            class: "dioxus-flow__node-default",
            "{node.node.id}"
            Handle::<N, E> { position: Position::Left, handle_type: HandleType::Target, node_id: node.node.id.clone(), is_connectable: node.connectable }
            Handle::<N, E> { position: Position::Right, handle_type: HandleType::Source, node_id: node.node.id.clone(), is_connectable: node.connectable }
        }
    }
}

#[allow(non_snake_case)]
pub fn InputNode<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    node: crate::components::NodeProps<N, E>,
) -> Element {
    rsx! {
        div {
            class: "dioxus-flow__node-input",
            "{node.node.id}"
            Handle::<N, E> { position: Position::Right, handle_type: HandleType::Source, node_id: node.node.id.clone(), is_connectable: node.connectable }
        }
    }
}

#[allow(non_snake_case)]
pub fn OutputNode<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    node: crate::components::NodeProps<N, E>,
) -> Element {
    rsx! {
        div {
            class: "dioxus-flow__node-output",
            "{node.node.id}"
            Handle::<N, E> { position: Position::Left, handle_type: HandleType::Target, node_id: node.node.id.clone(), is_connectable: node.connectable }
        }
    }
}

#[allow(non_snake_case)]
pub fn GroupNode<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    node: crate::components::NodeProps<N, E>,
) -> Element {
    rsx! {
        div {
            class: "dioxus-flow__node-group",
            "{node.node.id}"
        }
    }
}
