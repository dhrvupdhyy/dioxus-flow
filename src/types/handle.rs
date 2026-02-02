//! Handle types

use super::Position;
use serde::{Deserialize, Serialize};

/// Handle type - source or target
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, Serialize, Deserialize)]
pub enum HandleType {
    #[default]
    Source,
    Target,
}

impl HandleType {
    pub fn opposite(&self) -> Self {
        match self {
            HandleType::Source => HandleType::Target,
            HandleType::Target => HandleType::Source,
        }
    }
}

/// Handle information
#[derive(Clone, PartialEq, Debug)]
pub struct Handle {
    /// Handle ID (optional)
    pub id: Option<String>,
    /// Handle position on the node
    pub position: Position,
    /// Handle type
    pub handle_type: HandleType,
    /// Node ID this handle belongs to
    pub node_id: String,
    /// X position relative to node
    pub x: f64,
    /// Y position relative to node
    pub y: f64,
    /// Width of the handle
    pub width: f64,
    /// Height of the handle
    pub height: f64,
}

impl Handle {
    pub fn new(node_id: impl Into<String>, handle_type: HandleType, position: Position) -> Self {
        Self {
            id: None,
            position,
            handle_type,
            node_id: node_id.into(),
            x: 0.0,
            y: 0.0,
            width: 8.0,
            height: 8.0,
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Get the center point of the handle
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// Handle element info for DOM references
#[derive(Clone, PartialEq, Debug)]
pub struct HandleElement {
    pub id: Option<String>,
    pub position: Position,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
