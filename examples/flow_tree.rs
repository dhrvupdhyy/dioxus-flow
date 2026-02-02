use dioxus::prelude::*;
use std::collections::HashMap;

use dioxus_flow::components::{
    Background, Controls, DioxusFlow, Handle, MiniMap, NodeResizer, NodeToolbar,
};
use dioxus_flow::types::{
    Connection, Edge, EdgeMarker, HandleType, Node, Position, SelectionMode, XYPosition,
};
use dioxus_flow::BackgroundVariant;

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, PartialEq, Default)]
struct CardData {
    title: String,
    subtitle: String,
    badge: String,
}

#[allow(non_snake_case)]
fn CardNode(props: dioxus_flow::components::NodeProps<CardData, ()>) -> Element {
    let data = &props.node.data;
    let selected = props.selected;
    rsx! {
        div {
            class: if selected { "demo-card selected" } else { "demo-card" },
            div { class: "demo-card__header", "{data.title}" }
            div { class: "demo-card__subtitle", "{data.subtitle}" }
            div { class: "demo-card__badge", "{data.badge}" }
            Handle::<CardData, ()> {
                position: Position::Left,
                handle_type: HandleType::Target,
                node_id: props.node.id.clone(),
                is_connectable: true,
            }
            Handle::<CardData, ()> {
                position: Position::Right,
                handle_type: HandleType::Source,
                node_id: props.node.id.clone(),
                is_connectable: true,
            }
            NodeResizer::<CardData, ()> { is_visible: Some(selected) }
            NodeToolbar::<CardData, ()> {
                position: Position::Top,
                is_visible: Some(selected),
                div { class: "demo-toolbar", "Quick actions" }
            }
        }
    }
}

#[allow(non_snake_case)]
fn App() -> Element {
    let css = include_str!("../src/styles/dioxus-flow.css");
    let extra = r#"
    .demo-card {
        background: #ffffff;
        border: 1px solid #e1e1e1;
        border-radius: 8px;
        padding: 8px 12px;
        min-width: 200px;
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
        display: grid;
        gap: 4px;
        text-align: left;
        font-family: ui-sans-serif, system-ui, -apple-system, \"Segoe UI\", sans-serif;
    }
    .dioxus-flow__node.demo-node {
        background: transparent;
        border: none;
        box-shadow: none;
        padding: 0;
    }
    .dioxus-flow__node.demo-node.selected {
        box-shadow: none;
    }
    .demo-card.selected {
        border-color: #5b9bff;
        box-shadow: 0 0 0 1px rgba(91, 155, 255, 0.55), 0 1px 2px rgba(0, 0, 0, 0.08);
    }
    .demo-card__header {
        font-weight: 600;
        font-size: 13px;
    }
    .demo-card__subtitle {
        font-size: 11px;
        color: #666;
    }
    .demo-card__badge {
        font-size: 10px;
        padding: 2px 6px;
        border-radius: 999px;
        background: #f3f4f6;
        color: #4b5563;
        width: fit-content;
    }
    .demo-toolbar {
        background: #ffffff;
        border: 1px solid #e6e6e6;
        border-radius: 6px;
        padding: 4px 8px;
        font-size: 11px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
    }
    "#;

    let nodes = vec![
        Node::new("A", XYPosition::new(120.0, 120.0))
            .with_type("card")
            .with_class("demo-node")
            .with_data(CardData {
                title: "Onboarding".into(),
                subtitle: "3 steps pending".into(),
                badge: "Team".into(),
            })
            .with_dimensions(220.0, 90.0),
        Node::new("B", XYPosition::new(480.0, 80.0))
            .with_type("card")
            .with_class("demo-node")
            .with_data(CardData {
                title: "Design review".into(),
                subtitle: "Due tomorrow".into(),
                badge: "Design".into(),
            })
            .with_dimensions(220.0, 90.0),
        Node::new("C", XYPosition::new(480.0, 240.0))
            .with_type("card")
            .with_class("demo-node")
            .with_data(CardData {
                title: "Implementation".into(),
                subtitle: "In progress".into(),
                badge: "Engineering".into(),
            })
            .with_dimensions(220.0, 90.0),
        Node::new("D", XYPosition::new(860.0, 160.0))
            .with_type("card")
            .with_class("demo-node")
            .with_data(CardData {
                title: "QA & launch".into(),
                subtitle: "Blocked".into(),
                badge: "Release".into(),
            })
            .with_dimensions(220.0, 90.0),
    ];

    let mut edges: Vec<Edge<()>> = vec![
        Edge::new("e1", "A", "B").with_type("smoothstep"),
        Edge::new("e2", "A", "C").with_type("smoothstep"),
        Edge::new("e3", "B", "D").with_type("bezier"),
        Edge::new("e4", "C", "D").with_type("bezier"),
    ];

    for edge in edges.iter_mut() {
        edge.class_name = Some("dashed".into());
        edge.marker_end = Some(EdgeMarker::arrow_closed());
    }

    let mut node_types: HashMap<
        String,
        Component<dioxus_flow::components::NodeProps<CardData, ()>>,
    > = HashMap::new();
    node_types.insert("card".into(), CardNode);

    rsx! {
        div {
            style: "width: 100vw; height: 100vh; --df-background-pattern-color: #b7b7bf; --df-node-border-radius: 6px; --df-edge-color: #b1b1b7; --df-node-border-color: #e1e1e1; --df-handle-color: #666; --df-handle-border-color: #fff; --df-selection-border-color: #5b9bff;",
            style { "{css}" }
            style { "{extra}" }
            DioxusFlow {
                default_nodes: nodes,
                default_edges: edges,
                node_types: Some(node_types),
                selection_on_drag: true,
                selection_mode: SelectionMode::Partial,
                is_valid_connection: Some(validate_connection as dioxus_flow::types::IsValidConnection),

                Background { variant: Some(BackgroundVariant::Dots), gap: 26.0, size: 1.0 }
                Controls::<CardData, ()> { show_fit_view: true, show_zoom: true }
                MiniMap::<CardData, ()> { width: 180.0, height: 120.0 }
            }
        }
    }
}

fn validate_connection(conn: &Connection) -> bool {
    conn.source != conn.target
}
