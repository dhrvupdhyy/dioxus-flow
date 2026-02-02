//! Change types for nodes and edges

use super::{Dimensions, Edge, Node, XYPosition};

/// Changes that can be applied to nodes
#[derive(Clone, PartialEq, Debug)]
pub enum NodeChange<T: Clone + PartialEq + Default = ()> {
    /// Position change
    Position {
        id: String,
        position: Option<XYPosition>,
        dragging: bool,
    },
    /// Dimensions change
    Dimensions {
        id: String,
        dimensions: Option<Dimensions>,
        resizing: bool,
    },
    /// Selection change
    Selection { id: String, selected: bool },
    /// Remove node
    Remove { id: String },
    /// Add node
    Add { node: Node<T> },
    /// Replace node
    Replace { id: String, node: Node<T> },
}

impl<T: Clone + PartialEq + Default> NodeChange<T> {
    pub fn position(id: impl Into<String>, position: XYPosition, dragging: bool) -> Self {
        NodeChange::Position {
            id: id.into(),
            position: Some(position),
            dragging,
        }
    }

    pub fn dimensions(id: impl Into<String>, dimensions: Dimensions, resizing: bool) -> Self {
        NodeChange::Dimensions {
            id: id.into(),
            dimensions: Some(dimensions),
            resizing,
        }
    }

    pub fn select(id: impl Into<String>, selected: bool) -> Self {
        NodeChange::Selection {
            id: id.into(),
            selected,
        }
    }

    pub fn remove(id: impl Into<String>) -> Self {
        NodeChange::Remove { id: id.into() }
    }

    pub fn add(node: Node<T>) -> Self {
        NodeChange::Add { node }
    }

    pub fn replace(id: impl Into<String>, node: Node<T>) -> Self {
        NodeChange::Replace {
            id: id.into(),
            node,
        }
    }
}

/// Changes that can be applied to edges
#[derive(Clone, PartialEq, Debug)]
pub enum EdgeChange<T: Clone + PartialEq + Default = ()> {
    /// Selection change
    Selection { id: String, selected: bool },
    /// Remove edge
    Remove { id: String },
    /// Add edge
    Add { edge: Edge<T> },
    /// Replace edge
    Replace { id: String, edge: Edge<T> },
}

#[derive(Clone, PartialEq, Debug)]
pub struct SelectionChange<N: Clone + PartialEq + Default = (), E: Clone + PartialEq + Default = ()>
{
    pub nodes: Vec<Node<N>>,
    pub edges: Vec<Edge<E>>,
}

impl<T: Clone + PartialEq + Default> EdgeChange<T> {
    pub fn select(id: impl Into<String>, selected: bool) -> Self {
        EdgeChange::Selection {
            id: id.into(),
            selected,
        }
    }

    pub fn remove(id: impl Into<String>) -> Self {
        EdgeChange::Remove { id: id.into() }
    }

    pub fn add(edge: Edge<T>) -> Self {
        EdgeChange::Add { edge }
    }

    pub fn replace(id: impl Into<String>, edge: Edge<T>) -> Self {
        EdgeChange::Replace {
            id: id.into(),
            edge,
        }
    }
}

/// Apply node changes to a list of nodes
pub fn apply_node_changes<T: Clone + PartialEq + Default>(
    changes: Vec<NodeChange<T>>,
    mut nodes: Vec<Node<T>>,
) -> Vec<Node<T>> {
    for change in changes {
        match change {
            NodeChange::Position {
                id,
                position,
                dragging,
            } => {
                if let Some(node) = nodes.iter_mut().find(|n| n.id == id) {
                    if let Some(pos) = position {
                        node.position = pos;
                    }
                    node.dragging = dragging;
                }
            }
            NodeChange::Dimensions {
                id,
                dimensions,
                resizing,
            } => {
                if let Some(node) = nodes.iter_mut().find(|n| n.id == id) {
                    if let Some(dims) = dimensions {
                        node.measured_width = Some(dims.width);
                        node.measured_height = Some(dims.height);
                    }
                    node.resizing = resizing;
                }
            }
            NodeChange::Selection { id, selected } => {
                if let Some(node) = nodes.iter_mut().find(|n| n.id == id) {
                    node.selected = selected;
                }
            }
            NodeChange::Remove { id } => {
                nodes.retain(|n| n.id != id);
            }
            NodeChange::Add { node } => {
                nodes.push(node);
            }
            NodeChange::Replace { id, node } => {
                if let Some(idx) = nodes.iter().position(|n| n.id == id) {
                    nodes[idx] = node;
                }
            }
        }
    }
    nodes
}

/// Apply edge changes to a list of edges
pub fn apply_edge_changes<T: Clone + PartialEq + Default>(
    changes: Vec<EdgeChange<T>>,
    mut edges: Vec<Edge<T>>,
) -> Vec<Edge<T>> {
    for change in changes {
        match change {
            EdgeChange::Selection { id, selected } => {
                if let Some(edge) = edges.iter_mut().find(|e| e.id == id) {
                    edge.selected = selected;
                }
            }
            EdgeChange::Remove { id } => {
                edges.retain(|e| e.id != id);
            }
            EdgeChange::Add { edge } => {
                edges.push(edge);
            }
            EdgeChange::Replace { id, edge } => {
                if let Some(idx) = edges.iter().position(|e| e.id == id) {
                    edges[idx] = edge;
                }
            }
        }
    }
    edges
}

/// Get node position changes from drag
pub fn get_position_change<T: Clone + PartialEq + Default>(
    node: &Node<T>,
    position: XYPosition,
    dragging: bool,
) -> NodeChange<T> {
    NodeChange::Position {
        id: node.id.clone(),
        position: Some(position),
        dragging,
    }
}

/// Get selection changes for selecting/deselecting nodes
pub fn get_selection_changes<T: Clone + PartialEq + Default>(
    nodes: &[Node<T>],
    selected_ids: &[String],
) -> Vec<NodeChange<T>> {
    nodes
        .iter()
        .map(|node| {
            let should_be_selected = selected_ids.contains(&node.id);
            if node.selected != should_be_selected {
                Some(NodeChange::Selection {
                    id: node.id.clone(),
                    selected: should_be_selected,
                })
            } else {
                None
            }
        })
        .flatten()
        .collect()
}
