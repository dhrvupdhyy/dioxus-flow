//! Edge label renderer component

use crate::state::FlowState;
use dioxus::prelude::ReadableExt;
use dioxus::prelude::*;

#[component]
pub fn EdgeLabelRenderer<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    children: Element,
    #[props(default = true)] no_scale: bool,
    #[props(default)] _marker: std::marker::PhantomData<(N, E)>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();
    let viewport = *state.viewport.read();
    let transform = if no_scale {
        format!("transform: translate({}px, {}px);", viewport.x, viewport.y)
    } else {
        format!(
            "transform: translate({}px, {}px) scale({});",
            viewport.x, viewport.y, viewport.zoom
        )
    };

    rsx! {
        div {
            class: "dioxus-flow__edge-labels",
            style: "{transform}",
            {children}
        }
    }
}
