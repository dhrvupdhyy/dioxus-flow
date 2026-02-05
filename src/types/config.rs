//! Configuration types for Dioxus Flow

use serde::{Deserialize, Serialize};

/// Node origin used for positioning nodes
pub type NodeOrigin = (f64, f64);

/// Color mode for styling
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ColorMode {
    Light,
    Dark,
    System,
}

impl Default for ColorMode {
    fn default() -> Self {
        ColorMode::Light
    }
}

/// Z-index behavior for selections
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ZIndexMode {
    Basic,
    Auto,
    Manual,
}

impl Default for ZIndexMode {
    fn default() -> Self {
        ZIndexMode::Basic
    }
}

/// Configurable labels for accessibility
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct AriaLabelConfig {
    pub controls_zoom_in: Option<String>,
    pub controls_zoom_out: Option<String>,
    pub controls_fit_view: Option<String>,
    pub minimap: Option<String>,
    pub attribution: Option<String>,
    pub edge: Option<String>,
    pub node: Option<String>,
}

/// Pro configuration options
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct ProOptions {
    pub hide_attribution: bool,
}

/// Error handler for runtime issues
pub type OnError = fn(String);
