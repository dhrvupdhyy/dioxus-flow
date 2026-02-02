//! Controls component

use crate::state::FlowState;
use dioxus::prelude::*;

#[component]
pub fn Controls<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    #[props(default = true)] show_fit_view: bool,
    #[props(default = true)] show_zoom: bool,
    #[props(default)] position: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] _marker: std::marker::PhantomData<(N, E)>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();
    let position = position.unwrap_or_else(|| "top-left".to_string());
    let class = class.unwrap_or_default();

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
                if show_zoom {
                    button { onclick: on_zoom_in, "+" }
                    button { onclick: on_zoom_out, "-" }
                }
                if show_fit_view {
                    button { onclick: on_fit, "[]" }
                }
            }
        }
    }
}
