//! Attribution component

use dioxus::prelude::*;

#[component]
pub fn Attribution(
    #[props(default)] position: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] aria_label: Option<String>,
) -> Element {
    let position = position.unwrap_or_else(|| "bottom-right".to_string());
    let class = class.unwrap_or_default();
    let aria_label = aria_label.unwrap_or_else(|| "Dioxus Flow attribution".to_string());

    rsx! {
        div {
            class: "dioxus-flow__panel {position}",
            p {
                class: "dioxus-flow__attribution {class}",
                "aria-label": "{aria_label}",
                a { href: "https://github.com/dioxus-community/dioxus-flow", "Dioxus Flow" }
            }
        }
    }
}
