//! Graph utilities

use crate::types::{Edge, InternalNode, Node, Rect};
use std::collections::HashSet;

pub fn add_edge<E: Clone + PartialEq + Default>(
    edge: Edge<E>,
    mut edges: Vec<Edge<E>>,
) -> Vec<Edge<E>> {
    let exists = edges.iter().any(|e| {
        e.source == edge.source
            && e.target == edge.target
            && e.source_handle == edge.source_handle
            && e.target_handle == edge.target_handle
    });

    if !exists {
        edges.push(edge);
    }

    edges
}

pub fn get_nodes_bounds<N: Clone + PartialEq + Default>(nodes: &[Node<N>]) -> Rect {
    if nodes.is_empty() {
        return Rect::default();
    }

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for node in nodes {
        let dims = node.get_dimensions();
        min_x = min_x.min(node.position.x);
        min_y = min_y.min(node.position.y);
        max_x = max_x.max(node.position.x + dims.width);
        max_y = max_y.max(node.position.y + dims.height);
    }

    Rect {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    }
}

pub fn get_internal_nodes_bounds<N: Clone + PartialEq + Default>(
    nodes: impl IntoIterator<Item = InternalNode<N>>,
) -> Rect {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for node in nodes {
        let dims = node.dimensions;
        min_x = min_x.min(node.position_absolute.x);
        min_y = min_y.min(node.position_absolute.y);
        max_x = max_x.max(node.position_absolute.x + dims.width);
        max_y = max_y.max(node.position_absolute.y + dims.height);
    }

    if min_x == f64::MAX {
        return Rect::default();
    }

    Rect {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    }
}

pub fn get_nodes_inside<N: Clone + PartialEq + Default>(
    nodes: &[Node<N>],
    rect: Rect,
    fully_inside: bool,
) -> Vec<Node<N>> {
    nodes
        .iter()
        .filter(|node| {
            let dims = node.get_dimensions();
            let node_rect = Rect {
                x: node.position.x,
                y: node.position.y,
                width: dims.width,
                height: dims.height,
            };
            if fully_inside {
                rect.contains_rect(&node_rect)
            } else {
                rect.intersects(&node_rect)
            }
        })
        .cloned()
        .collect()
}

pub fn get_incomers<N, E>(node: &Node<N>, nodes: &[Node<N>], edges: &[Edge<E>]) -> Vec<Node<N>>
where
    N: Clone + PartialEq + Default,
    E: Clone + PartialEq + Default,
{
    let incoming_edges: Vec<_> = edges.iter().filter(|e| e.target == node.id).collect();

    incoming_edges
        .iter()
        .filter_map(|edge| nodes.iter().find(|n| n.id == edge.source))
        .cloned()
        .collect()
}

pub fn get_outgoers<N, E>(node: &Node<N>, nodes: &[Node<N>], edges: &[Edge<E>]) -> Vec<Node<N>>
where
    N: Clone + PartialEq + Default,
    E: Clone + PartialEq + Default,
{
    let outgoing_edges: Vec<_> = edges.iter().filter(|e| e.source == node.id).collect();

    outgoing_edges
        .iter()
        .filter_map(|edge| nodes.iter().find(|n| n.id == edge.target))
        .cloned()
        .collect()
}

pub fn get_connected_edges<N, E>(nodes: &[Node<N>], edges: &[Edge<E>]) -> Vec<Edge<E>>
where
    N: Clone + PartialEq + Default,
    E: Clone + PartialEq + Default,
{
    let node_ids: HashSet<_> = nodes.iter().map(|n| &n.id).collect();

    edges
        .iter()
        .filter(|edge| node_ids.contains(&edge.source) || node_ids.contains(&edge.target))
        .cloned()
        .collect()
}
