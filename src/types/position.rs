//! Position and geometry types

use serde::{Deserialize, Serialize};

/// 2D position with x and y coordinates
#[derive(Clone, Copy, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct XYPosition {
    pub x: f64,
    pub y: f64,
}

impl XYPosition {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &XYPosition) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl std::ops::Add for XYPosition {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for XYPosition {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Mul<f64> for XYPosition {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

/// Dimensions with width and height
#[derive(Clone, Copy, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct Dimensions {
    pub width: f64,
    pub height: f64,
}

impl Dimensions {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

/// Rectangle with position and dimensions
#[derive(Clone, Copy, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_position_and_dimensions(position: XYPosition, dimensions: Dimensions) -> Self {
        Self {
            x: position.x,
            y: position.y,
            width: dimensions.width,
            height: dimensions.height,
        }
    }

    pub fn center(&self) -> XYPosition {
        XYPosition {
            x: self.x + self.width / 2.0,
            y: self.y + self.height / 2.0,
        }
    }

    pub fn contains(&self, point: &XYPosition) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    pub fn contains_rect(&self, other: &Rect) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.x + other.width <= self.x + self.width
            && other.y + other.height <= self.y + self.height
    }

    pub fn union(&self, other: &Rect) -> Rect {
        let min_x = self.x.min(other.x);
        let min_y = self.y.min(other.y);
        let max_x = (self.x + self.width).max(other.x + other.width);
        let max_y = (self.y + self.height).max(other.y + other.height);

        Rect {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

/// Position enum for handle placement
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, Serialize, Deserialize)]
pub enum Position {
    Left,
    Right,
    #[default]
    Top,
    Bottom,
}

impl Position {
    pub fn opposite(&self) -> Self {
        match self {
            Position::Left => Position::Right,
            Position::Right => Position::Left,
            Position::Top => Position::Bottom,
            Position::Bottom => Position::Top,
        }
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(self, Position::Left | Position::Right)
    }

    pub fn is_vertical(&self) -> bool {
        matches!(self, Position::Top | Position::Bottom)
    }
}

/// Transform as [x, y, zoom]
pub type Transform = [f64; 3];

/// Coordinate extent as [[minX, minY], [maxX, maxY]]
pub type CoordinateExtent = [[f64; 2]; 2];

/// Represents the node extent constraint
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum NodeExtent {
    /// Constrain to specific coordinates
    CoordinateExtent(CoordinateExtent),
    /// Constrain to parent node
    Parent,
}

impl Default for NodeExtent {
    fn default() -> Self {
        NodeExtent::CoordinateExtent([
            [f64::NEG_INFINITY, f64::NEG_INFINITY],
            [f64::INFINITY, f64::INFINITY],
        ])
    }
}

/// Alignment for toolbars
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ToolbarAlign {
    Start,
    Center,
    End,
}

impl Default for ToolbarAlign {
    fn default() -> Self {
        ToolbarAlign::Center
    }
}
