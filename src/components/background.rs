//! Background component

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BackgroundVariant {
    Dots,
    Lines,
    Cross,
}

impl Default for BackgroundVariant {
    fn default() -> Self {
        BackgroundVariant::Dots
    }
}

#[component]
pub fn Background(
    #[props(default)] variant: Option<BackgroundVariant>,
    #[props(default = 20.0)] gap: f64,
    #[props(default = 1.0)] size: f64,
    #[props(default)] color: Option<String>,
) -> Element {
    let variant = variant.unwrap_or_default();
    let color = color.unwrap_or_else(|| "var(--df-background-pattern-color)".to_string());

    let background = match variant {
        BackgroundVariant::Dots => format!(
            "radial-gradient(circle, {} {}px, transparent {}px)",
            color, size, size + 0.5
        ),
        BackgroundVariant::Lines => format!(
            "linear-gradient(90deg, {} 1px, transparent 1px), linear-gradient(180deg, {} 1px, transparent 1px)",
            color, color
        ),
        BackgroundVariant::Cross => format!(
            "linear-gradient(90deg, {} 1px, transparent 1px), linear-gradient(180deg, {} 1px, transparent 1px), radial-gradient(circle, {} {}px, transparent {}px)",
            color, color, color, size, size + 0.5
        ),
    };

    rsx! {
        div {
            class: "dioxus-flow__background",
            style: "background-image: {background}; background-size: {gap}px {gap}px;",
        }
    }
}
