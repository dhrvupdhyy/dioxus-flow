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
    #[props(default)] id: Option<String>,
    #[props(default)] variant: Option<BackgroundVariant>,
    #[props(default = 20.0)] gap: f64,
    #[props(default)] size: Option<f64>,
    #[props(default)] color: Option<String>,
    #[props(default)] pattern_class_name: Option<String>,
) -> Element {
    let variant = variant.unwrap_or_default();
    let size = size.unwrap_or_else(|| match variant {
        BackgroundVariant::Cross => 6.0,
        _ => 1.0,
    });
    let color = color.unwrap_or_else(|| match variant {
        BackgroundVariant::Dots => "var(--df-background-pattern-color-dots)".to_string(),
        BackgroundVariant::Lines => "var(--df-background-pattern-color-lines)".to_string(),
        BackgroundVariant::Cross => "var(--df-background-pattern-color-cross)".to_string(),
    });

    let background = match variant {
        BackgroundVariant::Dots => format!(
            "radial-gradient(circle, {} {}px, transparent {}px)",
            color,
            size,
            size + 0.5
        ),
        BackgroundVariant::Lines => format!(
            "linear-gradient(90deg, {} 1px, transparent 1px), linear-gradient(180deg, {} 1px, transparent 1px)",
            color, color
        ),
        BackgroundVariant::Cross => format!(
            "linear-gradient(90deg, {} 1px, transparent 1px), linear-gradient(180deg, {} 1px, transparent 1px), radial-gradient(circle, {} {}px, transparent {}px)",
            color,
            color,
            color,
            size,
            size + 0.5
        ),
    };

    let pattern_class = pattern_class_name.unwrap_or_default();
    let id_attr = id.unwrap_or_default();
    let class = if pattern_class.is_empty() {
        "dioxus-flow__background".to_string()
    } else {
        format!("dioxus-flow__background {}", pattern_class)
    };

    rsx! {
        div {
            class: "{class}",
            id: "{id_attr}",
            style: "background-image: {background}; background-size: {gap}px {gap}px;",
        }
    }
}
