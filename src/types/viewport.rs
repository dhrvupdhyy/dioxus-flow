//! Viewport types

use serde::{Deserialize, Serialize};

/// Viewport state representing the current view transformation
#[derive(Clone, Copy, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct Viewport {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

impl Viewport {
    pub fn new(x: f64, y: f64, zoom: f64) -> Self {
        Self { x, y, zoom }
    }

    pub fn identity() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        }
    }

    pub fn to_transform(&self) -> [f64; 3] {
        [self.x, self.y, self.zoom]
    }

    pub fn from_transform(transform: [f64; 3]) -> Self {
        Self {
            x: transform[0],
            y: transform[1],
            zoom: transform[2],
        }
    }
}

/// Options for fit view operation
#[derive(Clone, PartialEq, Default, Debug)]
pub struct FitViewOptions {
    /// Padding around the bounds (0.0 - 1.0)
    pub padding: Option<f64>,
    /// Include hidden nodes
    pub include_hidden_nodes: bool,
    /// Minimum zoom level
    pub min_zoom: Option<f64>,
    /// Maximum zoom level
    pub max_zoom: Option<f64>,
    /// Animation duration in ms (None = no animation)
    pub duration: Option<u32>,
    /// Node IDs to focus on (None = all nodes)
    pub nodes: Option<Vec<String>>,
}

/// Options for set center operation
#[derive(Clone, PartialEq, Default, Debug)]
pub struct SetCenterOptions {
    /// Zoom level (None = keep current)
    pub zoom: Option<f64>,
    /// Animation duration in ms (None = no animation)
    pub duration: Option<u32>,
}

/// Options for fit bounds operation
#[derive(Clone, PartialEq, Default, Debug)]
pub struct FitBoundsOptions {
    /// Padding around the bounds (0.0 - 1.0)
    pub padding: Option<f64>,
    /// Animation duration in ms (None = no animation)
    pub duration: Option<u32>,
}

/// Pan on scroll mode
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum PanOnScrollMode {
    #[default]
    Free,
    Horizontal,
    Vertical,
}

/// Selection mode
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SelectionMode {
    #[default]
    Partial,
    Full,
}
