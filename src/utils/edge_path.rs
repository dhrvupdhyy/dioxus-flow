//! Edge path utilities

use crate::types::{EdgePathResult, Position, XYPosition};

pub fn get_bezier_path(
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
    source_position: Position,
    target_position: Position,
    curvature: Option<f64>,
) -> EdgePathResult {
    let curvature = curvature.unwrap_or(0.25);
    let source = XYPosition {
        x: source_x,
        y: source_y,
    };
    let target = XYPosition {
        x: target_x,
        y: target_y,
    };

    let (c1, c2) =
        bezier_control_points(source, target, source_position, target_position, curvature);

    let path = format!(
        "M {} {} C {} {}, {} {}, {} {}",
        source_x, source_y, c1.x, c1.y, c2.x, c2.y, target_x, target_y
    );

    let label = cubic_bezier_point(source, c1, c2, target, 0.5);

    EdgePathResult {
        path,
        label_x: label.x,
        label_y: label.y,
        offset_x: 0.0,
        offset_y: 0.0,
    }
}

pub fn get_simple_bezier_path(
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
) -> EdgePathResult {
    let source = XYPosition {
        x: source_x,
        y: source_y,
    };
    let target = XYPosition {
        x: target_x,
        y: target_y,
    };
    let c1 = XYPosition {
        x: (source_x + target_x) / 2.0,
        y: source_y,
    };
    let c2 = XYPosition {
        x: (source_x + target_x) / 2.0,
        y: target_y,
    };

    let path = format!(
        "M {} {} C {} {}, {} {}, {} {}",
        source_x, source_y, c1.x, c1.y, c2.x, c2.y, target_x, target_y
    );

    let label = cubic_bezier_point(source, c1, c2, target, 0.5);

    EdgePathResult {
        path,
        label_x: label.x,
        label_y: label.y,
        offset_x: 0.0,
        offset_y: 0.0,
    }
}

pub fn get_straight_path(
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
) -> EdgePathResult {
    let path = format!("M {} {} L {} {}", source_x, source_y, target_x, target_y);
    let label_x = (source_x + target_x) / 2.0;
    let label_y = (source_y + target_y) / 2.0;
    EdgePathResult {
        path,
        label_x,
        label_y,
        offset_x: 0.0,
        offset_y: 0.0,
    }
}

pub fn get_step_path(
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
    source_position: Position,
    _target_position: Position,
    offset: Option<f64>,
) -> EdgePathResult {
    let mut path = String::new();
    let mut points = Vec::new();

    if source_position.is_horizontal() {
        let mid_x = (source_x + target_x) / 2.0;
        points.push((source_x, source_y));
        points.push((mid_x, source_y));
        points.push((mid_x, target_y));
        points.push((target_x, target_y));
    } else {
        let mid_y = (source_y + target_y) / 2.0;
        points.push((source_x, source_y));
        points.push((source_x, mid_y));
        points.push((target_x, mid_y));
        points.push((target_x, target_y));
    }

    if let Some((x, y)) = points.first() {
        path.push_str(&format!("M {} {}", x, y));
        for (x, y) in points.iter().skip(1) {
            path.push_str(&format!(" L {} {}", x, y));
        }
    }

    let (label_x, label_y) = label_position_along_polyline(&points);

    EdgePathResult {
        path,
        label_x,
        label_y,
        offset_x: 0.0,
        offset_y: 0.0,
    }
}

pub fn get_smooth_step_path(
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
    source_position: Position,
    target_position: Position,
    border_radius: Option<f64>,
) -> EdgePathResult {
    let radius = border_radius.unwrap_or(8.0);
    let mut path = String::new();

    let (mut p0x, mut p0y) = (source_x, source_y);
    let mut points = Vec::new();

    if source_position.is_horizontal() {
        let mid_x = (source_x + target_x) / 2.0;
        points.push((source_x, source_y));
        points.push((mid_x, source_y));
        points.push((mid_x, target_y));
        points.push((target_x, target_y));
    } else {
        let mid_y = (source_y + target_y) / 2.0;
        points.push((source_x, source_y));
        points.push((source_x, mid_y));
        points.push((target_x, mid_y));
        points.push((target_x, target_y));
    }

    if let Some((x, y)) = points.first() {
        p0x = *x;
        p0y = *y;
        path.push_str(&format!("M {} {}", p0x, p0y));
    }

    for window in points.windows(3) {
        let (x1, y1) = window[0];
        let (x2, y2) = window[1];
        let (x3, y3) = window[2];

        let dx1 = x2 - x1;
        let dy1 = y2 - y1;
        let dx2 = x3 - x2;
        let dy2 = y3 - y2;

        let len1 = (dx1 * dx1 + dy1 * dy1).sqrt();
        let len2 = (dx2 * dx2 + dy2 * dy2).sqrt();

        if len1 < f64::EPSILON || len2 < f64::EPSILON {
            continue;
        }

        let r1 = radius.min(len1 / 2.0);
        let r2 = radius.min(len2 / 2.0);

        let x1a = x2 - dx1 / len1 * r1;
        let y1a = y2 - dy1 / len1 * r1;
        let x2b = x2 + dx2 / len2 * r2;
        let y2b = y2 + dy2 / len2 * r2;

        path.push_str(&format!(" L {} {}", x1a, y1a));
        path.push_str(&format!(" Q {} {} {} {}", x2, y2, x2b, y2b));
    }

    if let Some((x, y)) = points.last() {
        path.push_str(&format!(" L {} {}", x, y));
    }

    let (label_x, label_y) = label_position_along_polyline(&points);

    EdgePathResult {
        path,
        label_x,
        label_y,
        offset_x: 0.0,
        offset_y: 0.0,
    }
}

fn label_position_along_polyline(points: &[(f64, f64)]) -> (f64, f64) {
    if points.len() < 2 {
        return (0.0, 0.0);
    }
    let mut lengths = Vec::with_capacity(points.len() - 1);
    let mut total = 0.0;
    for window in points.windows(2) {
        let dx = window[1].0 - window[0].0;
        let dy = window[1].1 - window[0].1;
        let len = (dx * dx + dy * dy).sqrt();
        lengths.push(len);
        total += len;
    }
    if total == 0.0 {
        return points[0];
    }
    let mut mid = total / 2.0;
    for (idx, len) in lengths.iter().enumerate() {
        if mid <= *len {
            let (x0, y0) = points[idx];
            let (x1, y1) = points[idx + 1];
            let t = if *len == 0.0 { 0.0 } else { mid / *len };
            return (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
        }
        mid -= *len;
    }
    *points.last().unwrap()
}

fn bezier_control_points(
    source: XYPosition,
    target: XYPosition,
    source_position: Position,
    target_position: Position,
    curvature: f64,
) -> (XYPosition, XYPosition) {
    let distance = (target.x - source.x).abs().max((target.y - source.y).abs());
    let (sx, sy) = direction_vector(source_position);
    let (tx, ty) = direction_vector(target_position);

    let c1 = XYPosition {
        x: source.x + sx * distance * curvature,
        y: source.y + sy * distance * curvature,
    };

    let c2 = XYPosition {
        x: target.x + tx * distance * curvature,
        y: target.y + ty * distance * curvature,
    };

    (c1, c2)
}

fn direction_vector(position: Position) -> (f64, f64) {
    match position {
        Position::Left => (-1.0, 0.0),
        Position::Right => (1.0, 0.0),
        Position::Top => (0.0, -1.0),
        Position::Bottom => (0.0, 1.0),
    }
}

fn cubic_bezier_point(
    p0: XYPosition,
    p1: XYPosition,
    p2: XYPosition,
    p3: XYPosition,
    t: f64,
) -> XYPosition {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;

    let mut p = XYPosition { x: 0.0, y: 0.0 };
    p.x = uuu * p0.x + 3.0 * uu * t * p1.x + 3.0 * u * tt * p2.x + ttt * p3.x;
    p.y = uuu * p0.y + 3.0 * uu * t * p1.y + 3.0 * u * tt * p2.y + ttt * p3.y;
    p
}
