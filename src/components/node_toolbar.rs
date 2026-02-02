//! Node toolbar component

use crate::state::{FlowState, NodeIdContext};
use crate::types::{Position, Rect, ToolbarAlign, XYPosition};
use dioxus::prelude::*;
use dioxus::prelude::{try_use_context, ReadableExt};

#[component]
pub fn NodeToolbar<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    children: Element,
    #[props(default)] node_id: Option<String>,
    #[props(default)] node_ids: Option<Vec<String>>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default)] is_visible: Option<bool>,
    #[props(default = Position::Top)] position: Position,
    #[props(default = 10.0)] offset: f64,
    #[props(default = ToolbarAlign::Center)] align: ToolbarAlign,
    #[props(default)] _marker: std::marker::PhantomData<(N, E)>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();
    let context_id = try_use_context::<NodeIdContext>().map(|ctx| ctx.0);

    let mut ids: Vec<String> = Vec::new();
    if let Some(node_ids) = node_ids {
        ids.extend(node_ids);
    } else if let Some(node_id) = node_id {
        ids.push(node_id);
    } else if let Some(ctx_id) = context_id {
        ids.push(ctx_id);
    }

    if ids.is_empty() {
        return rsx! {};
    }

    let selected_nodes_count = state.nodes.read().iter().filter(|n| n.selected).count();
    let internal_nodes: Vec<_> = ids
        .iter()
        .filter_map(|id| state.node_lookup.read().get(id).cloned())
        .collect();

    let is_active = if let Some(is_visible) = is_visible {
        is_visible
    } else {
        internal_nodes.len() == 1
            && internal_nodes
                .first()
                .map(|n| n.node.selected)
                .unwrap_or(false)
            && selected_nodes_count == 1
    };

    if !is_active || internal_nodes.is_empty() {
        return rsx! {};
    }

    let bounds = internal_nodes_bounds(&internal_nodes);
    let screen_pos = node_toolbar_position(&state, bounds, position, offset, align);
    let (translate_x, translate_y) = toolbar_translate(align, position);
    let z_index = internal_nodes
        .iter()
        .filter_map(|n| n.node.z_index)
        .max()
        .unwrap_or(0)
        + 1;

    let base_style = format!(
        "position: absolute; transform: translate({}px, {}px) translate({}, {}); z-index: {};",
        screen_pos.x, screen_pos.y, translate_x, translate_y, z_index
    );
    let class = class.unwrap_or_default();
    let style = style.unwrap_or_default();
    let combined_style = if style.is_empty() {
        base_style
    } else {
        format!("{} {}", base_style, style)
    };

    let data_id = internal_nodes
        .iter()
        .map(|node| node.node.id.clone())
        .collect::<Vec<_>>()
        .join(" ");

    rsx! {
        crate::components::EdgeLabelRenderer::<N, E> {
            no_scale: true,
            div {
                class: "dioxus-flow__node-toolbar {class}",
                style: "{combined_style}",
                "data-id": "{data_id}",
                {children}
            }
        }
    }
}

fn internal_nodes_bounds<N: Clone + PartialEq + Default>(
    nodes: &[crate::types::InternalNode<N>],
) -> Rect {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for node in nodes {
        let dims = node.dimensions;
        min_x = min_x.min(node.position_absolute.x);
        min_y = min_y.min(node.position_absolute.y);
        max_x = max_x.max(node.position_absolute.x + dims.width);
        max_y = max_y.max(node.position_absolute.y + dims.height);
    }

    if min_x == f64::MAX {
        return Rect::default();
    }

    Rect {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    }
}

fn node_toolbar_position<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    state: &FlowState<N, E>,
    bounds: Rect,
    position: Position,
    offset: f64,
    align: ToolbarAlign,
) -> XYPosition {
    let anchor = match position {
        Position::Top => XYPosition::new(bounds.x + bounds.width / 2.0, bounds.y),
        Position::Bottom => {
            XYPosition::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height)
        }
        Position::Left => XYPosition::new(bounds.x, bounds.y + bounds.height / 2.0),
        Position::Right => XYPosition::new(bounds.x + bounds.width, bounds.y + bounds.height / 2.0),
    };

    let mut anchor = match position {
        Position::Top => XYPosition::new(anchor.x, anchor.y - offset),
        Position::Bottom => XYPosition::new(anchor.x, anchor.y + offset),
        Position::Left => XYPosition::new(anchor.x - offset, anchor.y),
        Position::Right => XYPosition::new(anchor.x + offset, anchor.y),
    };

    match position {
        Position::Top | Position::Bottom => match align {
            ToolbarAlign::Start => anchor.x = bounds.x,
            ToolbarAlign::Center => anchor.x = bounds.x + bounds.width / 2.0,
            ToolbarAlign::End => anchor.x = bounds.x + bounds.width,
        },
        Position::Left | Position::Right => match align {
            ToolbarAlign::Start => anchor.y = bounds.y,
            ToolbarAlign::Center => anchor.y = bounds.y + bounds.height / 2.0,
            ToolbarAlign::End => anchor.y = bounds.y + bounds.height,
        },
    }

    state.flow_to_screen_position(anchor)
}

fn toolbar_translate(align: ToolbarAlign, position: Position) -> (&'static str, &'static str) {
    match position {
        Position::Top | Position::Bottom => match align {
            ToolbarAlign::Start => ("0%", "-100%"),
            ToolbarAlign::Center => ("-50%", "-100%"),
            ToolbarAlign::End => ("-100%", "-100%"),
        },
        Position::Left | Position::Right => match align {
            ToolbarAlign::Start => ("-100%", "0%"),
            ToolbarAlign::Center => ("-100%", "-50%"),
            ToolbarAlign::End => ("-100%", "-100%"),
        },
    }
}
