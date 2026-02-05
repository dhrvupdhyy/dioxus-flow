//! Event payload types

use crate::types::{Connection, Edge, HandleType, Node, Position, Rect, XYPosition};

#[derive(Clone, PartialEq, Debug)]
pub struct ConnectionStartEvent {
    pub node_id: String,
    pub handle_id: Option<String>,
    pub handle_type: HandleType,
    pub position: Position,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ConnectionEndEvent {
    pub connection: Option<Connection>,
    pub is_valid: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct NodeMouseEvent<N: Clone + PartialEq + Default = ()> {
    pub node: Node<N>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct EdgeMouseEvent<E: Clone + PartialEq + Default = ()> {
    pub edge: Edge<E>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SelectionStartEvent {
    pub position: XYPosition,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SelectionEndEvent<N: Clone + PartialEq + Default = (), E: Clone + PartialEq + Default = ()> {
    pub selection_rect: Option<Rect>,
    pub nodes: Vec<Node<N>>,
    pub edges: Vec<Edge<E>>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct BeforeDeleteEvent<N: Clone + PartialEq + Default = (), E: Clone + PartialEq + Default = ()> {
    pub nodes: Vec<Node<N>>,
    pub edges: Vec<Edge<E>>,
}

pub type OnBeforeDelete<N, E> = fn(&BeforeDeleteEvent<N, E>) -> bool;

#[derive(Clone, PartialEq, Debug)]
pub struct DeleteEvent<N: Clone + PartialEq + Default = (), E: Clone + PartialEq + Default = ()> {
    pub nodes: Vec<Node<N>>,
    pub edges: Vec<Edge<E>>,
    pub node_changes: Vec<crate::types::NodeChange<N>>,
    pub edge_changes: Vec<crate::types::EdgeChange<E>>,
}
