//! Panel component

use dioxus::prelude::*;

#[component]
pub fn Panel(
    children: Element,
    #[props(default)] position: Option<String>,
    #[props(default)] class: Option<String>,
) -> Element {
    let position = position.unwrap_or_else(|| "top-left".to_string());
    let class = class.unwrap_or_default();

    rsx! {
        div {
            class: "dioxus-flow__panel {position} {class}",
            {children}
        }
    }
}
