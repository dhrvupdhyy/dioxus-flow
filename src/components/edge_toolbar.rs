//! Edge toolbar component

use dioxus::prelude::ReadableExt;
use dioxus::prelude::*;

use crate::state::FlowState;
use crate::types::{ToolbarAlign, XYPosition};

#[component]
pub fn EdgeToolbar<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    children: Element,
    edge_id: String,
    x: f64,
    y: f64,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default)] is_visible: Option<bool>,
    #[props(default = ToolbarAlign::Center)] align_x: ToolbarAlign,
    #[props(default = ToolbarAlign::Center)] align_y: ToolbarAlign,
    #[props(default = true)] auto_scale: bool,
    #[props(default)] _marker: std::marker::PhantomData<(N, E)>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();
    let edge = state
        .edges
        .read()
        .iter()
        .find(|edge| edge.id == edge_id)
        .cloned();
    let Some(edge) = edge else {
        return rsx! {};
    };

    let active = if let Some(is_visible) = is_visible {
        is_visible
    } else {
        edge.selected
    };
    if !active {
        return rsx! {};
    }

    let viewport = *state.viewport.read();
    let screen_pos = state.flow_to_screen_position(XYPosition::new(x, y));
    let translate_x = match align_x {
        ToolbarAlign::Start => "0%",
        ToolbarAlign::Center => "-50%",
        ToolbarAlign::End => "-100%",
    };
    let translate_y = match align_y {
        ToolbarAlign::Start => "0%",
        ToolbarAlign::Center => "-50%",
        ToolbarAlign::End => "-100%",
    };
    let scale = if auto_scale {
        (1.0 / viewport.zoom).max(0.0001)
    } else {
        1.0
    };
    let z_index = edge.z_index.unwrap_or(0) + 1;
    let base_style = format!(
        "position: absolute; transform: translate({}px, {}px) translate({}, {}) scale({}); transform-origin: 0 0; z-index: {};",
        screen_pos.x, screen_pos.y, translate_x, translate_y, scale, z_index
    );
    let class = class.unwrap_or_default();
    let style = style.unwrap_or_default();
    let combined_style = if style.is_empty() {
        base_style
    } else {
        format!("{} {}", base_style, style)
    };

    rsx! {
        crate::components::EdgeLabelRenderer::<N, E> {
            no_scale: true,
            div {
                class: "dioxus-flow__edge-toolbar {class}",
                style: "{combined_style}",
                "data-id": "{edge.id}",
                {children}
            }
        }
    }
}
