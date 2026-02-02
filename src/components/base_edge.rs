//! Base edge component for custom edges

use dioxus::prelude::*;

#[component]
pub fn BaseEdge(
    path: String,
    #[props(default)] class: Option<String>,
    #[props(default)] marker_start: Option<String>,
    #[props(default)] marker_end: Option<String>,
    #[props(default)] interaction_width: Option<f64>,
    #[props(default)] label: Option<String>,
    #[props(default)] label_x: Option<f64>,
    #[props(default)] label_y: Option<f64>,
    #[props(default)] label_style: Option<String>,
    #[props(default)] label_show_bg: Option<bool>,
    #[props(default)] label_bg_style: Option<String>,
    #[props(default)] label_bg_padding: Option<(f64, f64)>,
    #[props(default)] label_bg_border_radius: Option<f64>,
) -> Element {
    let class = class.unwrap_or_default();
    let marker_start = marker_start.unwrap_or_default();
    let marker_end = marker_end.unwrap_or_default();
    let interaction_width = interaction_width.unwrap_or(20.0);

    let label_metrics = label.as_ref().map(|text| {
        let text_len = text.chars().count() as f64;
        let text_width = text_len * 6.0;
        let text_height = 14.0;
        let padding = label_bg_padding.unwrap_or((6.0, 4.0));
        let bg_width = text_width + padding.0 * 2.0;
        let bg_height = text_height + padding.1 * 2.0;
        (bg_width, bg_height, padding)
    });

    rsx! {
        path {
            class: "dioxus-flow__edge-path {class}",
            d: "{path}",
            marker_start: "{marker_start}",
            marker_end: "{marker_end}",
        }
        if interaction_width > 0.0 {
            path {
                class: "dioxus-flow__edge-interaction",
                d: "{path}",
                stroke_width: "{interaction_width}",
            }
        }
        if let (Some(text), Some(x), Some(y)) = (label, label_x, label_y) {
            g {
                class: "dioxus-flow__edge-label",
                if label_show_bg.unwrap_or(false) {
                    if let Some((bg_width, bg_height, padding)) = label_metrics {
                        rect {
                            x: "{x - bg_width / 2.0}",
                            y: "{y - bg_height / 2.0}",
                            rx: "{label_bg_border_radius.unwrap_or(0.0)}",
                            ry: "{label_bg_border_radius.unwrap_or(0.0)}",
                            width: "{bg_width}",
                            height: "{bg_height}",
                            class: "dioxus-flow__edge-label-bg",
                            style: "{label_bg_style.clone().unwrap_or_default()}",
                        }
                    }
                }
                text {
                    x: "{x}",
                    y: "{y}",
                    text_anchor: "middle",
                    dominant_baseline: "middle",
                    class: "dioxus-flow__edge-label-text",
                    style: "{label_style.clone().unwrap_or_default()}",
                    "{text}"
                }
            }
        }
    }
}
