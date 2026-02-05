//! Connection types

use super::{HandleType, Position, XYPosition};
use serde::{Deserialize, Serialize};

/// A connection between two nodes
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Connection {
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Source handle ID
    #[serde(default)]
    pub source_handle: Option<String>,
    /// Target handle ID
    #[serde(default)]
    pub target_handle: Option<String>,
}

pub type IsValidConnection = fn(&Connection) -> bool;

impl Connection {
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            source_handle: None,
            target_handle: None,
        }
    }

    pub fn with_handles(
        mut self,
        source_handle: Option<String>,
        target_handle: Option<String>,
    ) -> Self {
        self.source_handle = source_handle;
        self.target_handle = target_handle;
        self
    }
}

/// Connection mode
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Serialize, Deserialize)]
pub enum ConnectionMode {
    /// Only connect matching handle types (source to target)
    #[default]
    Strict,
    /// Allow any connection
    Loose,
}

/// Connection line type
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Serialize, Deserialize)]
pub enum ConnectionLineType {
    #[default]
    Bezier,
    SmoothStep,
    Step,
    Straight,
    SimpleBezier,
}

/// State of the current connection being drawn
#[derive(Clone, PartialEq, Debug, Default)]
pub struct ConnectionState {
    /// Whether a connection is in progress
    pub in_progress: bool,
    /// Starting node ID
    pub from_node: Option<String>,
    /// Starting handle ID
    pub from_handle: Option<String>,
    /// Handle type being connected from
    pub from_type: Option<HandleType>,
    /// Starting position
    pub from_position: Option<Position>,
    /// Current mouse position
    pub to_position: Option<XYPosition>,
    /// Current mouse position in screen coordinates
    pub to_position_screen: Option<XYPosition>,
    /// Target node ID (if hovering over a valid target)
    pub to_node: Option<String>,
    /// Target handle ID
    pub to_handle: Option<String>,
    /// Target handle type
    pub to_type: Option<HandleType>,
    /// Edge being reconnected (if any)
    pub reconnect_edge_id: Option<String>,
    /// Which end of the edge is being reconnected
    pub reconnect_end: Option<HandleType>,
    /// Whether the current target is valid
    pub is_valid: bool,
    /// Whether the user has dragged far enough to start a connection
    pub dragging: bool,
    /// Initial screen position for drag threshold checks
    pub start_screen: Option<XYPosition>,
}

impl ConnectionState {
    pub fn start(
        from_node: String,
        from_handle: Option<String>,
        from_type: HandleType,
        from_position: Position,
    ) -> Self {
        Self {
            in_progress: true,
            from_node: Some(from_node),
            from_handle,
            from_type: Some(from_type),
            from_position: Some(from_position),
            to_position: None,
            to_position_screen: None,
            to_node: None,
            to_handle: None,
            to_type: None,
            reconnect_edge_id: None,
            reconnect_end: None,
            is_valid: false,
            dragging: false,
            start_screen: None,
        }
    }

    pub fn start_reconnect(
        edge_id: String,
        reconnect_end: HandleType,
        from_node: String,
        from_handle: Option<String>,
        from_type: HandleType,
        from_position: Position,
    ) -> Self {
        Self {
            in_progress: true,
            from_node: Some(from_node),
            from_handle,
            from_type: Some(from_type),
            from_position: Some(from_position),
            to_position: None,
            to_position_screen: None,
            to_node: None,
            to_handle: None,
            to_type: None,
            reconnect_edge_id: Some(edge_id),
            reconnect_end: Some(reconnect_end),
            is_valid: false,
            dragging: false,
            start_screen: None,
        }
    }

    pub fn update_position(&mut self, position: XYPosition) {
        self.to_position = Some(position);
    }

    pub fn update_screen_position(&mut self, screen: XYPosition, flow: XYPosition) {
        self.to_position_screen = Some(screen);
        self.to_position = Some(flow);
    }

    pub fn set_target(
        &mut self,
        node_id: String,
        handle_id: Option<String>,
        handle_type: HandleType,
        is_valid: bool,
    ) {
        self.to_node = Some(node_id);
        self.to_handle = handle_id;
        self.to_type = Some(handle_type);
        self.is_valid = is_valid;
    }

    pub fn clear_target(&mut self) {
        self.to_node = None;
        self.to_handle = None;
        self.to_type = None;
        self.is_valid = false;
    }

    pub fn end(&mut self) -> Option<Connection> {
        if self.in_progress && self.is_valid {
            let connection = self.to_connection();
            self.reset();
            connection
        } else {
            self.reset();
            None
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn to_connection(&self) -> Option<Connection> {
        if let (Some(from_node), Some(from_type)) = (&self.from_node, &self.from_type) {
            if let Some(to_node) = &self.to_node {
                let (source, target, source_handle, target_handle) = match from_type {
                    HandleType::Source => (
                        from_node.clone(),
                        to_node.clone(),
                        self.from_handle.clone(),
                        self.to_handle.clone(),
                    ),
                    HandleType::Target => (
                        to_node.clone(),
                        from_node.clone(),
                        self.to_handle.clone(),
                        self.from_handle.clone(),
                    ),
                };
                return Some(Connection {
                    source,
                    target,
                    source_handle,
                    target_handle,
                });
            }
        }
        None
    }
}

/// Props for connection line component
#[derive(Clone, PartialEq, Debug)]
pub struct ConnectionLineProps {
    pub from_x: f64,
    pub from_y: f64,
    pub to_x: f64,
    pub to_y: f64,
    pub from_position: Position,
    pub to_position: Position,
    pub connection_line_type: ConnectionLineType,
    pub from_node_id: String,
    pub from_handle_id: Option<String>,
    pub to_node_id: Option<String>,
    pub to_handle_id: Option<String>,
    pub is_valid: bool,
}
