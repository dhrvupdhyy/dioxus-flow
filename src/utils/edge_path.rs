//! Edge path utilities

use crate::types::{EdgePathResult, Position, XYPosition};

const DEFAULT_BEZIER_CURVATURE: f64 = 0.25;
const DEFAULT_SMOOTH_STEP_RADIUS: f64 = 5.0;
const DEFAULT_SMOOTH_STEP_OFFSET: f64 = 20.0;
const DEFAULT_STEP_POSITION: f64 = 0.5;

pub fn get_bezier_path(
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
    source_position: Position,
    target_position: Position,
    curvature: Option<f64>,
) -> EdgePathResult {
    let curvature = curvature.unwrap_or(DEFAULT_BEZIER_CURVATURE);

    let (source_control_x, source_control_y) = get_control_with_curvature(
        source_position,
        source_x,
        source_y,
        target_x,
        target_y,
        curvature,
    );
    let (target_control_x, target_control_y) = get_control_with_curvature(
        target_position,
        target_x,
        target_y,
        source_x,
        source_y,
        curvature,
    );
    let (label_x, label_y, offset_x, offset_y) = get_bezier_edge_center(
        source_x,
        source_y,
        target_x,
        target_y,
        source_control_x,
        source_control_y,
        target_control_x,
        target_control_y,
    );

    EdgePathResult {
        path: format!(
            "M{},{} C{},{} {},{} {},{}",
            source_x, source_y, source_control_x, source_control_y, target_control_x, target_control_y, target_x, target_y
        ),
        label_x,
        label_y,
        offset_x,
        offset_y,
    }
}

pub fn get_simple_bezier_path(
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
    source_position: Position,
    target_position: Position,
) -> EdgePathResult {
    let (source_control_x, source_control_y) =
        get_simple_control(source_position, source_x, source_y, target_x, target_y);
    let (target_control_x, target_control_y) =
        get_simple_control(target_position, target_x, target_y, source_x, source_y);
    let (label_x, label_y, offset_x, offset_y) = get_bezier_edge_center(
        source_x,
        source_y,
        target_x,
        target_y,
        source_control_x,
        source_control_y,
        target_control_x,
        target_control_y,
    );

    EdgePathResult {
        path: format!(
            "M{},{} C{},{} {},{} {},{}",
            source_x, source_y, source_control_x, source_control_y, target_control_x, target_control_y, target_x, target_y
        ),
        label_x,
        label_y,
        offset_x,
        offset_y,
    }
}

pub fn get_straight_path(
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
) -> EdgePathResult {
    let (label_x, label_y, offset_x, offset_y) =
        get_edge_center(source_x, source_y, target_x, target_y);
    EdgePathResult {
        path: format!("M {},{}L {},{}", source_x, source_y, target_x, target_y),
        label_x,
        label_y,
        offset_x,
        offset_y,
    }
}

pub fn get_step_path(
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
    source_position: Position,
    target_position: Position,
    offset: Option<f64>,
) -> EdgePathResult {
    smooth_step_path(
        source_x,
        source_y,
        target_x,
        target_y,
        source_position,
        target_position,
        Some(0.0),
        offset,
        None,
    )
}

pub fn get_smooth_step_path(
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
    source_position: Position,
    target_position: Position,
    border_radius: Option<f64>,
    offset: Option<f64>,
    step_position: Option<f64>,
) -> EdgePathResult {
    smooth_step_path(
        source_x,
        source_y,
        target_x,
        target_y,
        source_position,
        target_position,
        border_radius,
        offset,
        step_position,
    )
}

fn smooth_step_path(
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
    source_position: Position,
    target_position: Position,
    border_radius: Option<f64>,
    offset: Option<f64>,
    step_position: Option<f64>,
) -> EdgePathResult {
    let border_radius = border_radius.unwrap_or(DEFAULT_SMOOTH_STEP_RADIUS);
    let offset = offset.unwrap_or(DEFAULT_SMOOTH_STEP_OFFSET);
    let step_position = step_position.unwrap_or(DEFAULT_STEP_POSITION);

    let (points, label_x, label_y, offset_x, offset_y) = get_smooth_step_points(
        XYPosition::new(source_x, source_y),
        source_position,
        XYPosition::new(target_x, target_y),
        target_position,
        offset,
        step_position,
    );

    let path = points.iter().enumerate().fold(String::new(), |mut res, (i, p)| {
        let segment = if i > 0 && i < points.len() - 1 {
            get_bend(points[i - 1], *p, points[i + 1], border_radius)
        } else if i == 0 {
            format!("M{} {}", p.x, p.y)
        } else {
            format!("L{} {}", p.x, p.y)
        };
        res.push_str(&segment);
        res
    });

    EdgePathResult {
        path,
        label_x,
        label_y,
        offset_x,
        offset_y,
    }
}

fn get_edge_center(
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
) -> (f64, f64, f64, f64) {
    let offset_x = (target_x - source_x).abs() / 2.0;
    let center_x = if target_x < source_x {
        target_x + offset_x
    } else {
        target_x - offset_x
    };

    let offset_y = (target_y - source_y).abs() / 2.0;
    let center_y = if target_y < source_y {
        target_y + offset_y
    } else {
        target_y - offset_y
    };

    (center_x, center_y, offset_x, offset_y)
}

fn get_bezier_edge_center(
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
    source_control_x: f64,
    source_control_y: f64,
    target_control_x: f64,
    target_control_y: f64,
) -> (f64, f64, f64, f64) {
    let center_x =
        source_x * 0.125 + source_control_x * 0.375 + target_control_x * 0.375 + target_x * 0.125;
    let center_y =
        source_y * 0.125 + source_control_y * 0.375 + target_control_y * 0.375 + target_y * 0.125;
    let offset_x = (center_x - source_x).abs();
    let offset_y = (center_y - source_y).abs();
    (center_x, center_y, offset_x, offset_y)
}

fn calculate_control_offset(distance: f64, curvature: f64) -> f64 {
    if distance >= 0.0 {
        return 0.5 * distance;
    }
    curvature * 25.0 * (-distance).sqrt()
}

fn get_control_with_curvature(
    position: Position,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    curvature: f64,
) -> (f64, f64) {
    match position {
        Position::Left => (x1 - calculate_control_offset(x1 - x2, curvature), y1),
        Position::Right => (x1 + calculate_control_offset(x2 - x1, curvature), y1),
        Position::Top => (x1, y1 - calculate_control_offset(y1 - y2, curvature)),
        Position::Bottom => (x1, y1 + calculate_control_offset(y2 - y1, curvature)),
    }
}

fn get_simple_control(position: Position, x1: f64, y1: f64, x2: f64, y2: f64) -> (f64, f64) {
    if position == Position::Left || position == Position::Right {
        ((x1 + x2) * 0.5, y1)
    } else {
        (x1, (y1 + y2) * 0.5)
    }
}

fn get_smooth_step_points(
    source: XYPosition,
    source_position: Position,
    target: XYPosition,
    target_position: Position,
    offset: f64,
    step_position: f64,
) -> (Vec<XYPosition>, f64, f64, f64, f64) {
    let source_dir = handle_direction(source_position);
    let target_dir = handle_direction(target_position);
    let source_gapped = XYPosition::new(
        source.x + source_dir.x * offset,
        source.y + source_dir.y * offset,
    );
    let target_gapped = XYPosition::new(
        target.x + target_dir.x * offset,
        target.y + target_dir.y * offset,
    );
    let dir = get_direction(source_gapped, source_position, target_gapped);
    let (dir_accessor, curr_dir) = if dir.x != 0.0 {
        (Axis::X, dir.x)
    } else {
        (Axis::Y, dir.y)
    };

    let (_, _, default_offset_x, default_offset_y) =
        get_edge_center(source.x, source.y, target.x, target.y);

    let mut points: Vec<XYPosition>;
    let center_x: f64;
    let center_y: f64;
    let mut source_gap_offset = XYPosition::new(0.0, 0.0);
    let mut target_gap_offset = XYPosition::new(0.0, 0.0);

    if source_dir.axis(dir_accessor) * target_dir.axis(dir_accessor) == -1.0 {
        if dir_accessor == Axis::X {
            center_x = source_gapped.x + (target_gapped.x - source_gapped.x) * step_position;
            center_y = (source_gapped.y + target_gapped.y) / 2.0;
        } else {
            center_x = (source_gapped.x + target_gapped.x) / 2.0;
            center_y = source_gapped.y + (target_gapped.y - source_gapped.y) * step_position;
        }

        let vertical_split = vec![
            XYPosition::new(center_x, source_gapped.y),
            XYPosition::new(center_x, target_gapped.y),
        ];
        let horizontal_split = vec![
            XYPosition::new(source_gapped.x, center_y),
            XYPosition::new(target_gapped.x, center_y),
        ];

        if source_dir.axis(dir_accessor) == curr_dir {
            points = if dir_accessor == Axis::X { vertical_split } else { horizontal_split };
        } else {
            points = if dir_accessor == Axis::X { horizontal_split } else { vertical_split };
        }
    } else {
        let source_target = vec![XYPosition::new(source_gapped.x, target_gapped.y)];
        let target_source = vec![XYPosition::new(target_gapped.x, source_gapped.y)];

        if dir_accessor == Axis::X {
            points = if source_dir.x == curr_dir {
                target_source.clone()
            } else {
                source_target.clone()
            };
        } else {
            points = if source_dir.y == curr_dir {
                source_target.clone()
            } else {
                target_source.clone()
            };
        }

        if source_position == target_position {
            let diff = if dir_accessor == Axis::X {
                (source.x - target.x).abs()
            } else {
                (source.y - target.y).abs()
            };
            if diff <= offset {
                let gap_offset = (offset - 1.0).min(offset - diff);
                if dir_accessor == Axis::X {
                    if source_dir.x == curr_dir {
                        source_gap_offset.x = if source_gapped.x > source.x {
                            -gap_offset
                        } else {
                            gap_offset
                        };
                    } else {
                        target_gap_offset.x = if target_gapped.x > target.x {
                            -gap_offset
                        } else {
                            gap_offset
                        };
                    }
                } else if source_dir.y == curr_dir {
                    source_gap_offset.y = if source_gapped.y > source.y {
                        -gap_offset
                    } else {
                        gap_offset
                    };
                } else {
                    target_gap_offset.y = if target_gapped.y > target.y {
                        -gap_offset
                    } else {
                        gap_offset
                    };
                }
            }
        }

        if source_position != target_position {
            let dir_accessor_opp = if dir_accessor == Axis::X { Axis::Y } else { Axis::X };
            let is_same_dir = source_dir.axis(dir_accessor) == target_dir.axis(dir_accessor_opp);
            let source_gt_target_opp =
                source_gapped.axis(dir_accessor_opp) > target_gapped.axis(dir_accessor_opp);
            let source_lt_target_opp =
                source_gapped.axis(dir_accessor_opp) < target_gapped.axis(dir_accessor_opp);
            let flip_source_target = if source_dir.axis(dir_accessor) == 1.0 {
                (!is_same_dir && source_gt_target_opp) || (is_same_dir && source_lt_target_opp)
            } else {
                (!is_same_dir && source_lt_target_opp) || (is_same_dir && source_gt_target_opp)
            };

            if flip_source_target {
                points = if dir_accessor == Axis::X {
                    source_target
                } else {
                    target_source
                };
            }
        }

        let source_gap_point = XYPosition::new(
            source_gapped.x + source_gap_offset.x,
            source_gapped.y + source_gap_offset.y,
        );
        let target_gap_point = XYPosition::new(
            target_gapped.x + target_gap_offset.x,
            target_gapped.y + target_gap_offset.y,
        );
        let max_x_distance = (source_gap_point.x - points[0].x)
            .abs()
            .max((target_gap_point.x - points[0].x).abs());
        let max_y_distance = (source_gap_point.y - points[0].y)
            .abs()
            .max((target_gap_point.y - points[0].y).abs());

        if max_x_distance >= max_y_distance {
            center_x = (source_gap_point.x + target_gap_point.x) / 2.0;
            center_y = points[0].y;
        } else {
            center_x = points[0].x;
            center_y = (source_gap_point.y + target_gap_point.y) / 2.0;
        }
    }

    let mut path_points = Vec::with_capacity(2 + points.len() + 2);
    path_points.push(source);
    path_points.push(XYPosition::new(
        source_gapped.x + source_gap_offset.x,
        source_gapped.y + source_gap_offset.y,
    ));
    path_points.extend(points.iter().cloned());
    path_points.push(XYPosition::new(
        target_gapped.x + target_gap_offset.x,
        target_gapped.y + target_gap_offset.y,
    ));
    path_points.push(target);

    (path_points, center_x, center_y, default_offset_x, default_offset_y)
}

fn get_bend(a: XYPosition, b: XYPosition, c: XYPosition, size: f64) -> String {
    let bend_size = (distance(a, b) / 2.0)
        .min(distance(b, c) / 2.0)
        .min(size);
    let x = b.x;
    let y = b.y;

    if (a.x == x && x == c.x) || (a.y == y && y == c.y) {
        return format!("L{} {}", x, y);
    }

    if a.y == y {
        let x_dir = if a.x < c.x { -1.0 } else { 1.0 };
        let y_dir = if a.y < c.y { 1.0 } else { -1.0 };
        return format!(
            "L {} {}Q {} {} {} {}",
            x + bend_size * x_dir,
            y,
            x,
            y,
            x,
            y + bend_size * y_dir
        );
    }

    let x_dir = if a.x < c.x { 1.0 } else { -1.0 };
    let y_dir = if a.y < c.y { -1.0 } else { 1.0 };
    format!(
        "L {} {}Q {} {} {} {}",
        x,
        y + bend_size * y_dir,
        x,
        y,
        x + bend_size * x_dir,
        y
    )
}

fn handle_direction(position: Position) -> XYPosition {
    match position {
        Position::Left => XYPosition::new(-1.0, 0.0),
        Position::Right => XYPosition::new(1.0, 0.0),
        Position::Top => XYPosition::new(0.0, -1.0),
        Position::Bottom => XYPosition::new(0.0, 1.0),
    }
}

fn get_direction(source: XYPosition, source_position: Position, target: XYPosition) -> XYPosition {
    if source_position == Position::Left || source_position == Position::Right {
        if source.x < target.x {
            XYPosition::new(1.0, 0.0)
        } else {
            XYPosition::new(-1.0, 0.0)
        }
    } else if source.y < target.y {
        XYPosition::new(0.0, 1.0)
    } else {
        XYPosition::new(0.0, -1.0)
    }
}

fn distance(a: XYPosition, b: XYPosition) -> f64 {
    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Axis {
    X,
    Y,
}

trait AxisAccess {
    fn axis(self, axis: Axis) -> f64;
}

impl AxisAccess for XYPosition {
    fn axis(self, axis: Axis) -> f64 {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
        }
    }
}
