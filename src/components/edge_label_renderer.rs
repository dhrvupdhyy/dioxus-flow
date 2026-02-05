//! Edge label renderer component

use dioxus::prelude::*;

#[component]
pub fn EdgeLabelRenderer<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    children: Element,
    #[props(default = true)] _no_scale: bool,
    #[props(default)] _marker: std::marker::PhantomData<(N, E)>,
) -> Element {
    rsx! {
        div {
            class: "dioxus-flow__edgelabel-renderer",
            div { class: "dioxus-flow__edge-labels", {children} }
        }
    }
}
