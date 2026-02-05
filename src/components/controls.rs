//! Controls component

use crate::state::FlowState;
use dioxus::prelude::*;

#[component]
pub fn Controls<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    #[props(default)] children: Element,
    #[props(default = true)] show_fit_view: bool,
    #[props(default = true)] show_zoom: bool,
    #[props(default)] position: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] _marker: std::marker::PhantomData<(N, E)>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();
    let position = position.unwrap_or_else(|| "top-left".to_string());
    let class = class.unwrap_or_default();
    let aria_label = aria_label
        .or_else(|| state.aria_label_config.read().controls_fit_view.clone())
        .unwrap_or_else(|| "Flow controls".to_string());
    let zoom_in_label = state
        .aria_label_config
        .read()
        .controls_zoom_in
        .clone()
        .unwrap_or_else(|| "Zoom in".to_string());
    let zoom_out_label = state
        .aria_label_config
        .read()
        .controls_zoom_out
        .clone()
        .unwrap_or_else(|| "Zoom out".to_string());
    let fit_view_label = state
        .aria_label_config
        .read()
        .controls_fit_view
        .clone()
        .unwrap_or_else(|| "Fit view".to_string());

    let mut state_zoom_in = state.clone();
    let mut state_zoom_out = state.clone();
    let mut state_fit = state.clone();

    let on_zoom_in = move |_| state_zoom_in.zoom_in(None);
    let on_zoom_out = move |_| state_zoom_out.zoom_out(None);
    let on_fit = move |_| state_fit.fit_view(None);

    rsx! {
        div {
            class: "dioxus-flow__panel {position}",
            div {
                class: "dioxus-flow__controls {class}",
                "aria-label": "{aria_label}",
                if show_zoom {
                    button { onclick: on_zoom_in, "aria-label": "{zoom_in_label}", "+" }
                    button { onclick: on_zoom_out, "aria-label": "{zoom_out_label}", "-" }
                }
                if show_fit_view {
                    button { onclick: on_fit, "aria-label": "{fit_view_label}", "[]" }
                }
                {children}
            }
        }
    }
}
