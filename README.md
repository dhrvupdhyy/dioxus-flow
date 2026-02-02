# Dioxus Flow (WIP)

A node-based graph editor like React Flow for Dioxus. 

## For testing

1) Add the stylesheet to your app bundle:

```rust
let css = include_str!("../src/styles/dioxus-flow.css");
rsx! { style { "{css}" } }
```

2) Render the flow:

```rust
DioxusFlow {
    default_nodes: nodes,
    default_edges: edges,
    node_types: Some(node_types),
    Background { variant: Some(BackgroundVariant::Dots), gap: 24.0, size: 1.0 }
    Controls::<(), ()> { show_fit_view: true, show_zoom: true }
    MiniMap::<(), ()> { width: 180.0, height: 120.0 }
}
```

## Common Props

- `default_nodes`, `default_edges`: Initial graph
- `node_types`, `edge_types`: Custom renderers
- `selection_on_drag`, `selection_mode`: Selection behavior
- `on_connect`, `on_nodes_change`, `on_edges_change`: Change handlers
- `min_zoom`, `max_zoom`, `pan_on_scroll`, `zoom_on_scroll`: Viewport behavior

## Examples (WIP)

```bash
dx serve --example flow_tree # or basic
```
