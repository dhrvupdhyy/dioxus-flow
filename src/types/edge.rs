//! Edge types

use super::Position;
use crate::types::Connection;
use serde::{Deserialize, Serialize};

/// An edge connecting two nodes
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Edge<T: Clone + PartialEq + Default = ()> {
    /// Unique identifier for the edge
    pub id: String,
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Source handle ID (optional)
    #[serde(default)]
    pub source_handle: Option<String>,
    /// Target handle ID (optional)
    #[serde(default)]
    pub target_handle: Option<String>,
    /// Custom data associated with the edge
    #[serde(default)]
    pub data: Option<T>,
    /// Type of edge (used to select edge component)
    #[serde(default)]
    pub edge_type: Option<String>,
    /// Whether the edge is animated
    #[serde(default)]
    pub animated: bool,
    /// Whether the edge is selected
    #[serde(default)]
    pub selected: bool,
    /// Whether the edge can be selected
    #[serde(default)]
    pub selectable: Option<bool>,
    /// Whether the edge can be deleted
    #[serde(default)]
    pub deletable: Option<bool>,
    /// Whether the edge can be focused
    #[serde(default)]
    pub focusable: Option<bool>,
    /// Whether the edge is hidden
    #[serde(default)]
    pub hidden: bool,
    /// Whether the edge can be reconnected
    #[serde(default)]
    pub reconnectable: Option<ReconnectableValue>,
    /// Z-index for layering
    #[serde(default)]
    pub z_index: Option<i32>,
    /// Label text
    #[serde(default)]
    pub label: Option<String>,
    /// Label style
    #[serde(default)]
    pub label_style: Option<String>,
    /// Whether to show label background
    #[serde(default)]
    pub label_show_bg: Option<bool>,
    /// Label background style
    #[serde(default)]
    pub label_bg_style: Option<String>,
    /// Label background padding [x, y]
    #[serde(default)]
    pub label_bg_padding: Option<(f64, f64)>,
    /// Label background border radius
    #[serde(default)]
    pub label_bg_border_radius: Option<f64>,
    /// Edge path style
    #[serde(default)]
    pub style: Option<String>,
    /// CSS class name
    #[serde(default)]
    pub class_name: Option<String>,
    /// Marker at the start of the edge
    #[serde(default)]
    pub marker_start: Option<EdgeMarker>,
    /// Marker at the end of the edge
    #[serde(default)]
    pub marker_end: Option<EdgeMarker>,
    /// Interaction width for easier selection
    #[serde(default)]
    pub interaction_width: Option<f64>,
}

impl<T: Clone + PartialEq + Default> Default for Edge<T> {
    fn default() -> Self {
        Self {
            id: String::new(),
            source: String::new(),
            target: String::new(),
            source_handle: None,
            target_handle: None,
            data: None,
            edge_type: None,
            animated: false,
            selected: false,
            selectable: None,
            deletable: None,
            focusable: None,
            hidden: false,
            reconnectable: None,
            z_index: None,
            label: None,
            label_style: None,
            label_show_bg: None,
            label_bg_style: None,
            label_bg_padding: None,
            label_bg_border_radius: None,
            style: None,
            class_name: None,
            marker_start: None,
            marker_end: None,
            interaction_width: None,
        }
    }
}

impl<T: Clone + PartialEq + Default> Edge<T> {
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            ..Default::default()
        }
    }

    pub fn with_source_handle(mut self, handle: impl Into<String>) -> Self {
        self.source_handle = Some(handle.into());
        self
    }

    pub fn with_target_handle(mut self, handle: impl Into<String>) -> Self {
        self.target_handle = Some(handle.into());
        self
    }

    pub fn with_type(mut self, edge_type: impl Into<String>) -> Self {
        self.edge_type = Some(edge_type.into());
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn with_style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }

    pub fn with_data(mut self, data: T) -> Self {
        self.data = Some(data);
        self
    }
}

/// Reconnectable value - can be true, false, or source/target only
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ReconnectableValue {
    True,
    False,
    Source,
    Target,
}

impl Default for ReconnectableValue {
    fn default() -> Self {
        ReconnectableValue::True
    }
}

/// Edge update event payload (reconnection).
#[derive(Clone, PartialEq, Debug)]
pub struct EdgeUpdateEvent<T: Clone + PartialEq + Default = ()> {
    pub edge: Edge<T>,
    pub connection: Connection,
}

/// Edge update end payload (success or cancel).
#[derive(Clone, PartialEq, Debug)]
pub struct EdgeUpdateEndEvent<T: Clone + PartialEq + Default = ()> {
    pub edge: Edge<T>,
    pub connection: Option<Connection>,
}

/// Edge marker configuration
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EdgeMarker {
    /// Marker type
    pub marker_type: MarkerType,
    /// Marker color
    #[serde(default)]
    pub color: Option<String>,
    /// Marker width
    #[serde(default)]
    pub width: Option<f64>,
    /// Marker height
    #[serde(default)]
    pub height: Option<f64>,
    /// Marker units
    #[serde(default)]
    pub marker_units: Option<String>,
    /// Orient
    #[serde(default)]
    pub orient: Option<String>,
    /// Stroke width
    #[serde(default)]
    pub stroke_width: Option<f64>,
}

impl EdgeMarker {
    pub fn arrow() -> Self {
        Self {
            marker_type: MarkerType::Arrow,
            color: None,
            width: None,
            height: None,
            marker_units: None,
            orient: None,
            stroke_width: None,
        }
    }

    pub fn arrow_closed() -> Self {
        Self {
            marker_type: MarkerType::ArrowClosed,
            color: None,
            width: None,
            height: None,
            marker_units: None,
            orient: None,
            stroke_width: None,
        }
    }

    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }
}

/// Marker type
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum MarkerType {
    #[default]
    Arrow,
    ArrowClosed,
}

/// Edge path options
#[derive(Clone, PartialEq, Debug, Default)]
pub struct EdgePathOptions {
    /// Curvature for bezier edges (0.0 - 1.0)
    pub curvature: Option<f64>,
    /// Border radius for smooth step edges
    pub border_radius: Option<f64>,
    /// Offset for step edges
    pub offset: Option<f64>,
}

/// Props passed to edge components
#[derive(Clone, PartialEq, Debug)]
pub struct EdgeProps<T: Clone + PartialEq + Default = ()> {
    pub id: String,
    pub source: String,
    pub target: String,
    pub source_x: f64,
    pub source_y: f64,
    pub target_x: f64,
    pub target_y: f64,
    pub source_position: Position,
    pub target_position: Position,
    pub source_handle_id: Option<String>,
    pub target_handle_id: Option<String>,
    pub data: Option<T>,
    pub label: Option<String>,
    pub label_style: Option<String>,
    pub label_show_bg: bool,
    pub label_bg_style: Option<String>,
    pub label_bg_padding: (f64, f64),
    pub label_bg_border_radius: f64,
    pub style: Option<String>,
    pub marker_start: Option<String>,
    pub marker_end: Option<String>,
    pub path_options: Option<EdgePathOptions>,
    pub interaction_width: f64,
    pub selected: bool,
    pub animated: bool,
}

/// Result from edge path calculation
#[derive(Clone, PartialEq, Debug)]
pub struct EdgePathResult {
    pub path: String,
    pub label_x: f64,
    pub label_y: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}
