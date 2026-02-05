use dioxus::prelude::*;
use std::collections::HashMap;

use dioxus_flow::BackgroundVariant;
use dioxus_flow::components::{Background, Controls, DefaultNode, DioxusFlow, MiniMap};
use dioxus_flow::types::{Connection, Edge, EdgeMarker, Node, SelectionMode, XYPosition};

fn main() {
    dioxus::launch(App);
}

#[allow(non_snake_case)]
fn App() -> Element {
    let css = include_str!("../src/styles/dioxus-flow.css");
    let node_dims = (180.0, 44.0);
    let mut nodes = vec![
        Node::new("Root", XYPosition::new(520.0, 40.0))
            .with_type("default")
            .with_dimensions(node_dims.0, node_dims.1),
        Node::new("Child 1", XYPosition::new(60.0, 180.0))
            .with_type("default")
            .with_dimensions(node_dims.0, node_dims.1),
        Node::new("Child 2", XYPosition::new(520.0, 260.0))
            .with_type("default")
            .with_dimensions(node_dims.0, node_dims.1),
        Node::new("Child 3", XYPosition::new(980.0, 260.0))
            .with_type("default")
            .with_dimensions(node_dims.0, node_dims.1),
        Node::new("Grandchild 1", XYPosition::new(0.0, 380.0))
            .with_type("default")
            .with_dimensions(node_dims.0, node_dims.1),
        Node::new("Grandchild 2", XYPosition::new(240.0, 380.0))
            .with_type("default")
            .with_dimensions(node_dims.0, node_dims.1),
        Node::new("Grandchild 4", XYPosition::new(480.0, 380.0))
            .with_type("default")
            .with_dimensions(node_dims.0, node_dims.1),
        Node::new("Grandchild 5", XYPosition::new(720.0, 380.0))
            .with_type("default")
            .with_dimensions(node_dims.0, node_dims.1),
        Node::new("Grandchild 7", XYPosition::new(940.0, 380.0))
            .with_type("default")
            .with_dimensions(node_dims.0, node_dims.1),
        Node::new("Grandchild 8", XYPosition::new(1140.0, 380.0))
            .with_type("default")
            .with_dimensions(node_dims.0, node_dims.1),
    ];
    if let Some(first) = nodes.get_mut(0) {
        first.selected = true;
    }

    let mut edges: Vec<Edge<()>> = vec![
        Edge::new("e-root-1", "Root", "Child 1").with_type("smoothstep"),
        Edge::new("e-root-2", "Root", "Child 2").with_type("smoothstep"),
        Edge::new("e-root-3", "Root", "Child 3").with_type("smoothstep"),
        Edge::new("e-1-1", "Child 1", "Grandchild 1").with_type("smoothstep"),
        Edge::new("e-1-2", "Child 1", "Grandchild 2").with_type("smoothstep"),
        Edge::new("e-2-4", "Child 2", "Grandchild 4").with_type("smoothstep"),
        Edge::new("e-2-5", "Child 2", "Grandchild 5").with_type("smoothstep"),
        Edge::new("e-3-7", "Child 3", "Grandchild 7").with_type("smoothstep"),
        Edge::new("e-3-8", "Child 3", "Grandchild 8").with_type("smoothstep"),
    ];

    for edge in edges.iter_mut() {
        edge.class_name = Some("dashed".into());
        edge.marker_end = Some(EdgeMarker::arrow());
    }
    if let Some(edge) = edges.get_mut(0) {
        edge.selected = true;
    }

    let mut node_types: HashMap<String, Component<dioxus_flow::components::NodeProps<(), ()>>> =
        HashMap::new();
    node_types.insert("default".into(), DefaultNode);

    rsx! {
        div {
            style: "width: 100vw; height: 100vh; --df-background-pattern-color-dots: #d6d6d6; --df-node-border-color: #e3e3e3; --df-edge-color: #b7b7b7; --df-node-border-radius: 10px; --df-handle-color: #d0d0d0; --df-handle-border-color: #ffffff;",
            style { "{css}" }
            DioxusFlow {
                default_nodes: nodes,
                default_edges: edges,
                node_types: Some(node_types),
                selection_on_drag: true,
                selection_mode: SelectionMode::Full,
                selection_key_code: Some(vec!["Shift".into()]),
                multi_selection_key_code: Some(vec!["Meta".into(), "Control".into()]),
                is_valid_connection: Some(validate_connection as dioxus_flow::types::IsValidConnection),

                Background { variant: Some(BackgroundVariant::Dots), gap: 24.0, size: 1.0 }
                Controls::<(), ()> { show_fit_view: true, show_zoom: true }
                MiniMap::<(), ()> { width: 180.0, height: 120.0 }
            }
        }
    }
}

fn validate_connection(conn: &Connection) -> bool {
    conn.source != conn.target
}
