//! Viewport utility helpers

use crate::types::{Rect, Viewport};

pub fn get_viewport_for_bounds(
    bounds: Rect,
    width: f64,
    height: f64,
    min_zoom: f64,
    max_zoom: f64,
    padding: f64,
) -> Viewport {
    if bounds.width <= 0.0 || bounds.height <= 0.0 || width <= 0.0 || height <= 0.0 {
        return Viewport::identity();
    }

    let x_zoom = width / bounds.width / (1.0 + padding * 2.0);
    let y_zoom = height / bounds.height / (1.0 + padding * 2.0);
    let zoom = x_zoom.min(y_zoom).clamp(min_zoom, max_zoom);

    let x = (width - bounds.width * zoom) / 2.0 - bounds.x * zoom;
    let y = (height - bounds.height * zoom) / 2.0 - bounds.y * zoom;

    Viewport { x, y, zoom }
}
