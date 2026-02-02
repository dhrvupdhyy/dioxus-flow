//! Node types

use super::{Dimensions, NodeExtent, Position, XYPosition};
use serde::{Deserialize, Serialize};

/// A node in the flow graph
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Node<T: Clone + PartialEq + Default = ()> {
    /// Unique identifier for the node
    pub id: String,
    /// Position of the node
    pub position: XYPosition,
    /// Custom data associated with the node
    #[serde(default)]
    pub data: T,
    /// Type of node (used to select node component)
    #[serde(default)]
    pub node_type: Option<String>,
    /// Inline style string
    #[serde(default)]
    pub style: Option<String>,
    /// CSS class name
    #[serde(default)]
    pub class_name: Option<String>,
    /// Whether the node can be dragged
    #[serde(default)]
    pub draggable: Option<bool>,
    /// CSS selector for drag handle within the node
    #[serde(default)]
    pub drag_handle: Option<String>,
    /// Whether the node can be selected
    #[serde(default)]
    pub selectable: Option<bool>,
    /// Whether the node can be connected
    #[serde(default)]
    pub connectable: Option<bool>,
    /// Whether the node can be focused
    #[serde(default)]
    pub focusable: Option<bool>,
    /// Whether the node can be deleted
    #[serde(default)]
    pub deletable: Option<bool>,
    /// Whether the node is hidden
    #[serde(default)]
    pub hidden: bool,
    /// Whether the node is selected
    #[serde(default)]
    pub selected: bool,
    /// Whether the node is being dragged
    #[serde(default)]
    pub dragging: bool,
    /// Whether the node is being resized
    #[serde(default)]
    pub resizing: bool,
    /// Width of the node
    #[serde(default)]
    pub width: Option<f64>,
    /// Height of the node
    #[serde(default)]
    pub height: Option<f64>,
    /// Measured width (set by the system after render)
    #[serde(default)]
    pub measured_width: Option<f64>,
    /// Measured height (set by the system after render)
    #[serde(default)]
    pub measured_height: Option<f64>,
    /// Z-index for layering
    #[serde(default)]
    pub z_index: Option<i32>,
    /// Parent node ID for nesting
    #[serde(default)]
    pub parent_id: Option<String>,
    /// Whether to expand parent to fit this node
    #[serde(default)]
    pub expand_parent: bool,
    /// Extent constraint for node position
    #[serde(default)]
    pub extent: Option<NodeExtent>,
    /// Source handle position
    #[serde(default)]
    pub source_position: Option<Position>,
    /// Target handle position
    #[serde(default)]
    pub target_position: Option<Position>,
    /// Aria label for accessibility
    #[serde(default)]
    pub aria_label: Option<String>,
}

impl<T: Clone + PartialEq + Default> Default for Node<T> {
    fn default() -> Self {
        Self {
            id: String::new(),
            position: XYPosition::default(),
            data: T::default(),
            node_type: None,
            style: None,
            class_name: None,
            draggable: None,
            drag_handle: None,
            selectable: None,
            connectable: None,
            focusable: None,
            deletable: None,
            hidden: false,
            selected: false,
            dragging: false,
            resizing: false,
            width: None,
            height: None,
            measured_width: None,
            measured_height: None,
            z_index: None,
            parent_id: None,
            expand_parent: false,
            extent: None,
            source_position: None,
            target_position: None,
            aria_label: None,
        }
    }
}

impl<T: Clone + PartialEq + Default> Node<T> {
    pub fn new(id: impl Into<String>, position: XYPosition) -> Self {
        Self {
            id: id.into(),
            position,
            ..Default::default()
        }
    }

    pub fn with_data(mut self, data: T) -> Self {
        self.data = data;
        self
    }

    pub fn with_type(mut self, node_type: impl Into<String>) -> Self {
        self.node_type = Some(node_type.into());
        self
    }

    pub fn with_style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }

    pub fn with_class(mut self, class_name: impl Into<String>) -> Self {
        self.class_name = Some(class_name.into());
        self
    }

    pub fn with_dimensions(mut self, width: f64, height: f64) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn get_dimensions(&self) -> Dimensions {
        Dimensions {
            width: self.width.or(self.measured_width).unwrap_or(150.0),
            height: self.height.or(self.measured_height).unwrap_or(40.0),
        }
    }
}

/// Event payload for node dragging callbacks.
#[derive(Clone, PartialEq, Debug)]
pub struct NodeDragEvent<T: Clone + PartialEq + Default = ()> {
    pub node: Node<T>,
    pub nodes: Vec<Node<T>>,
}

pub type ShouldResize<T> = fn(&Node<T>, Dimensions) -> bool;

#[derive(Clone, PartialEq, Debug)]
pub struct NodeResizeEvent<T: Clone + PartialEq + Default = ()> {
    pub node: Node<T>,
    pub dimensions: Dimensions,
}

/// Internal node representation with computed values
#[derive(Clone, PartialEq, Debug)]
pub struct InternalNode<T: Clone + PartialEq + Default = ()> {
    /// The original node
    pub node: Node<T>,
    /// Absolute position (including parent offset)
    pub position_absolute: XYPosition,
    /// Computed dimensions
    pub dimensions: Dimensions,
    /// Handle bounds for connection detection
    pub handle_bounds: Option<HandleBounds>,
}

/// Handle bounds for a node
#[derive(Clone, PartialEq, Debug, Default)]
pub struct HandleBounds {
    pub source: Vec<HandleBound>,
    pub target: Vec<HandleBound>,
}

/// Individual handle bound
#[derive(Clone, PartialEq, Debug)]
pub struct HandleBound {
    pub id: Option<String>,
    pub position: Position,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub is_connectable: bool,
}
