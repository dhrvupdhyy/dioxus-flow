use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};

use dioxus_flow::components::{
    Background, BaseEdge, Controls, DioxusFlow, EdgeToolbar, GroupNode, Handle, MiniMap,
};
use dioxus_flow::hooks::{use_edges_state, use_nodes_state};
use dioxus_flow::state::connection_to_edge;
use dioxus_flow::types::{
    ColorMode, Connection, ConnectionLineProps, ConnectionLineType, ConnectionMode, Edge,
    EdgeMarker, EdgeMouseEvent, EdgeUpdateEndEvent, EdgeUpdateEvent, HandleType, IsValidConnection,
    Node, NodeDragEvent, NodeExtent, NodeMouseEvent, PanOnScrollMode, Position,
    ReconnectableValue, SelectionChange, SelectionMode, Viewport, XYPosition,
};
use dioxus_flow::utils::add_edge;
use dioxus_flow::{BackgroundVariant, EdgeChange, NodeChange};

fn main() {
    dioxus::launch(App);
}

fn preset_empty() -> (Vec<Node<()>>, Vec<Edge<()>>) {
    (vec![], vec![])
}

fn flow_node(id: impl Into<String>, pos: XYPosition, ty: &str) -> Node<()> {
    Node::new(id, pos).with_type(ty)
}

fn flow_node_horizontal(id: impl Into<String>, pos: XYPosition, ty: &str) -> Node<()> {
    let mut node = flow_node(id, pos, ty);
    node.source_position = Some(Position::Right);
    node.target_position = Some(Position::Left);
    node
}

fn flow_node_vertical(id: impl Into<String>, pos: XYPosition, ty: &str) -> Node<()> {
    let mut node = flow_node(id, pos, ty);
    node.source_position = Some(Position::Bottom);
    node.target_position = Some(Position::Top);
    node
}

fn preset_basic() -> (Vec<Node<()>>, Vec<Edge<()>>) {
    let nodes = vec![
        flow_node_horizontal("Input 1", XYPosition::new(80.0, 100.0), "input"),
        flow_node_horizontal("Process", XYPosition::new(360.0, 80.0), "default"),
        flow_node_horizontal("Output 1", XYPosition::new(640.0, 100.0), "output"),
    ];
    let mut e1 = Edge::new("e-in1-proc", "Input 1", "Process");
    e1.edge_type = Some("smoothstep".into());
    e1.marker_end = Some(EdgeMarker::arrow());
    let mut e2 = Edge::new("e-proc-out1", "Process", "Output 1");
    e2.edge_type = Some("bezier".into());
    e2.marker_end = Some(EdgeMarker::arrow());
    (nodes, vec![e1, e2])
}

fn preset_pipeline() -> (Vec<Node<()>>, Vec<Edge<()>>) {
    let types = ["input", "custom", "default", "custom", "output"];
    let labels = ["Source", "Transform", "Validate", "Enrich", "Sink"];
    let nodes: Vec<Node<()>> = types
        .iter()
        .zip(labels.iter())
        .enumerate()
        .map(|(i, (t, l))| {
            flow_node(*l, XYPosition::new(60.0 + i as f64 * 220.0, 140.0), t)
        })
        .map(|mut n| {
            n.source_position = Some(Position::Right);
            n.target_position = Some(Position::Left);
            n
        })
        .collect();

    let edge_types = ["bezier", "smoothstep", "step", "straight"];
    let edges: Vec<Edge<()>> = (0..4)
        .map(|i| {
            let mut e = Edge::new(
                &format!("e-pipe-{i}"),
                labels[i],
                labels[i + 1],
            );
            e.edge_type = Some(edge_types[i].into());
            e.label = Some(edge_types[i].into());
            e.label_show_bg = Some(true);
            e.marker_end = Some(EdgeMarker::arrow());
            e
        })
        .collect();
    (nodes, edges)
}

fn preset_tree() -> (Vec<Node<()>>, Vec<Edge<()>>) {
    let nodes = vec![
        flow_node_vertical("Root", XYPosition::new(400.0, 40.0), "default"),
        flow_node_vertical("A", XYPosition::new(120.0, 180.0), "default"),
        flow_node_vertical("B", XYPosition::new(400.0, 180.0), "default"),
        flow_node_vertical("C", XYPosition::new(680.0, 180.0), "default"),
        flow_node_vertical("A1", XYPosition::new(40.0, 340.0), "output"),
        flow_node_vertical("A2", XYPosition::new(200.0, 340.0), "output"),
        flow_node_vertical("B1", XYPosition::new(320.0, 340.0), "output"),
        flow_node_vertical("B2", XYPosition::new(480.0, 340.0), "output"),
        flow_node_vertical("C1", XYPosition::new(600.0, 340.0), "output"),
        flow_node_vertical("C2", XYPosition::new(760.0, 340.0), "output"),
    ];
    let pairs = [
        ("Root", "A"), ("Root", "B"), ("Root", "C"),
        ("A", "A1"), ("A", "A2"),
        ("B", "B1"), ("B", "B2"),
        ("C", "C1"), ("C", "C2"),
    ];
    let edges: Vec<Edge<()>> = pairs
        .iter()
        .enumerate()
        .map(|(i, (s, t))| {
            let mut e = Edge::new(&format!("e-tree-{i}"), *s, *t);
            e.edge_type = Some("smoothstep".into());
            e.marker_end = Some(EdgeMarker::arrow());
            e
        })
        .collect();
    (nodes, edges)
}

fn preset_kitchen_sink() -> (Vec<Node<()>>, Vec<Edge<()>>) {
    let mut group = Node::new("Group", XYPosition::new(40.0, 40.0)).with_type("group");
    group.width = Some(420.0);
    group.height = Some(260.0);
    group.style = Some("background: rgba(30, 41, 59, 0.04);".into());

    let mut child_a = flow_node_horizontal("Group-A", XYPosition::new(24.0, 52.0), "default");
    child_a.parent_id = Some("Group".into());
    child_a.extent = Some(NodeExtent::Parent);

    let mut child_b = flow_node_horizontal("Group-B", XYPosition::new(24.0, 140.0), "custom");
    child_b.parent_id = Some("Group".into());
    child_b.extent = Some(NodeExtent::Parent);

    let mut input = flow_node_horizontal("Input", XYPosition::new(560.0, 40.0), "input");
    input.style = Some("background: #e0f2fe;".into());

    let mut processor = flow_node_horizontal("Processor", XYPosition::new(520.0, 200.0), "custom");
    processor.style = Some("background: #fff7ed;".into());
    processor.selected = true;

    let mut output = flow_node_horizontal("Output", XYPosition::new(840.0, 160.0), "output");
    output.style = Some("background: #dcfce7;".into());

    let mut floating = flow_node_horizontal("Floating", XYPosition::new(700.0, 360.0), "default");
    floating.width = Some(180.0);
    floating.height = Some(56.0);

    let nodes = vec![group, child_a, child_b, input, processor, output, floating];

    let mut e1 = Edge::new("e-ks-1", "Input", "Processor");
    e1.edge_type = Some("smoothstep".into());
    e1.label = Some("ingest".into());
    e1.label_show_bg = Some(true);
    e1.marker_end = Some(EdgeMarker::arrow().with_color("#2563eb"));

    let mut e2 = Edge::new("e-ks-2", "Processor", "Output");
    e2.edge_type = Some("custom-edge".into());
    e2.animated = true;
    e2.reconnectable = Some(ReconnectableValue::Target);
    e2.selected = true;

    let mut e3 = Edge::new("e-ks-3", "Group-A", "Group-B");
    e3.edge_type = Some("step".into());

    let mut e4 = Edge::new("e-ks-4", "Group-B", "Processor");
    e4.edge_type = Some("straight".into());
    e4.label = Some("handoff".into());

    let mut e5 = Edge::new("e-ks-5", "Processor", "Floating");
    e5.edge_type = Some("simplebezier".into());
    e5.marker_end = Some(EdgeMarker::arrow_closed().with_color("#0f766e"));

    (nodes, vec![e1, e2, e3, e4, e5])
}

fn parse_key_list(value: &str) -> Option<Vec<String>> {
    let mut keys: Vec<String> = value
        .split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    keys.dedup();
    if keys.is_empty() { None } else { Some(keys) }
}

fn edge_type_for_line_type(line_type: ConnectionLineType) -> Option<String> {
    Some(
        match line_type {
            ConnectionLineType::Bezier => "bezier",
            ConnectionLineType::SmoothStep => "smoothstep",
            ConnectionLineType::Step => "step",
            ConnectionLineType::Straight => "straight",
            ConnectionLineType::SimpleBezier => "simplebezier",
        }
        .to_string(),
    )
}

fn position_from_str(value: &str) -> Position {
    match value {
        "left" => Position::Left,
        "right" => Position::Right,
        "top" => Position::Top,
        "bottom" => Position::Bottom,
        _ => Position::Right,
    }
}

fn position_to_str(value: &Position) -> &str {
    match value {
        Position::Left => "left",
        Position::Right => "right",
        Position::Top => "top",
        Position::Bottom => "bottom",
    }
}

fn validate_connection(conn: &Connection) -> bool {
    conn.source != conn.target
}

fn toggle(mut s: Signal<bool>) {
    let v = *s.read();
    s.set(!v);
}

fn push_log(log: &mut Signal<Vec<String>>, kind: &str, msg: String) {
    let mut v = log.read().clone();
    let stamp = v.len();
    v.push(format!("[{stamp:>4}] {kind}: {msg}"));
    if v.len() > 200 {
        v.drain(0..v.len() - 200);
    }
    log.set(v);
}

const PLAYGROUND_CSS: &str = r#"
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
:root {
    --pg-bg: #eef2f7; --pg-surface: #ffffff; --pg-border: #dbe4f0;
    --pg-text: #0f172a; --pg-muted: #64748b; --pg-accent: #3b82f6;
    --pg-accent-light: #eaf2ff; --pg-danger: #ef4444; --pg-success: #22c55e;
    --pg-canvas-bg: #f8fafc;
    --pg-radius: 10px; --pg-font: "IBM Plex Sans", "Segoe UI", sans-serif;
}
[data-theme="dark"] {
    --pg-bg: #0b1020; --pg-surface: #141b2d; --pg-border: #263145;
    --pg-text: #f1f5f9; --pg-muted: #94a3b8; --pg-accent: #60a5fa;
    --pg-accent-light: #1c2d49; --pg-danger: #f87171; --pg-success: #4ade80;
    --pg-canvas-bg: #090f1d;
}
body { font-family: var(--pg-font); color: var(--pg-text); background: var(--pg-bg); }
.pg {
    display: flex;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    color: var(--pg-text);
    background: var(--pg-bg);
}
.pg__side {
    width: 340px; min-width: 340px; display: flex; flex-direction: column;
    background: var(--pg-surface); border-right: 1px solid var(--pg-border);
}
.pg__header {
    display: flex; align-items: center; justify-content: space-between; gap: 12px;
    padding: 14px 16px; border-bottom: 1px solid var(--pg-border);
}
.pg__title { font-size: 15px; font-weight: 700; letter-spacing: -0.02em; }
.pg__theme-btn {
    background: none; border: 1px solid var(--pg-border); border-radius: 6px;
    padding: 4px 10px; font-size: 12px; cursor: pointer; color: var(--pg-text);
}
.pg__tabs {
    display: flex; border-bottom: 1px solid var(--pg-border); padding: 0 8px;
}
.pg__tab {
    padding: 10px 14px; font-size: 12px; font-weight: 600; cursor: pointer;
    border-bottom: 2px solid transparent; color: var(--pg-muted); user-select: none;
    text-transform: uppercase; letter-spacing: 0.04em;
}
.pg__tab:hover { color: var(--pg-text); }
.pg__tab.active { border-bottom-color: var(--pg-accent); color: var(--pg-accent); }
.pg__body { flex: 1; overflow-y: auto; padding: 12px 14px; }
.pg__canvas { flex: 1; position: relative; background: var(--pg-canvas-bg); }
.pg__flow { width: 100%; height: 100%; }
.pg__canvas .dioxus-flow__background {
    background-color: var(--pg-canvas-bg);
}
.pg__flow.dioxus-flow {
    --df-background-color: var(--pg-canvas-bg);
    --df-node-border-radius: 6px;
    --df-edge-stroke-width: 1px;
    --df-node-color: var(--pg-text);
    --df-controls-button-color: var(--pg-text);
    --df-controls-button-color-hover: var(--pg-text);
    --df-edge-label-color: var(--pg-text);
    --df-minimap-bg-color: #ffffff;
    --df-minimap-border-color: #cbd5e1;
    --df-minimap-mask-color: rgba(148, 163, 184, 0.28);
    --df-minimap-mask-stroke-color: #64748b;
    --df-minimap-node-fill: #dbeafe;
    --df-minimap-node-stroke-color: #93c5fd;
    --df-controls-box-shadow: 0 8px 24px rgba(15, 23, 42, 0.12);
}
.pg__flow.dioxus-flow,
.pg__flow .dioxus-flow__node-default,
.pg__flow .dioxus-flow__node-input,
.pg__flow .dioxus-flow__node-output,
.pg__flow .dioxus-flow__node-group {
    color: var(--pg-text);
}
[data-theme="dark"] .pg__flow.dioxus-flow {
    --df-background-color: var(--pg-canvas-bg);
    --df-background-pattern-color: #334155;
    --df-background-pattern-color-dots: #3a4b66;
    --df-background-pattern-color-lines: #2a3b54;
    --df-background-pattern-color-cross: #2a3b54;
    --df-node-background-color: #111a2b;
    --df-node-border-color: #3a4860;
    --df-node-border-selected-color: #60a5fa;
    --df-edge-color: #8ca3c7;
    --df-edge-color-selected: #c9dcff;
    --df-handle-color: #d8e5ff;
    --df-handle-border-color: #111a2b;
    --df-edge-label-bg-color: #111a2b;
    --df-edge-label-color: #e2e8f0;
    --df-controls-button-bg: #1a2539;
    --df-controls-button-bg-hover: #243654;
    --df-controls-button-color: #e2e8f0;
    --df-controls-button-color-hover: #f8fafc;
    --df-controls-button-border-color: #344865;
    --df-minimap-bg-color: #0f172a;
    --df-minimap-border-color: #334155;
    --df-minimap-mask-color: rgba(15, 23, 42, 0.55);
    --df-minimap-mask-stroke-color: #94a3b8;
    --df-minimap-node-fill: #1f2937;
    --df-minimap-node-stroke-color: #4b5563;
    --df-controls-box-shadow: 0 10px 28px rgba(2, 6, 23, 0.5);
}
.pg__flow .dioxus-flow__edge-label-text {
    fill: var(--df-edge-label-color);
}
.pg__status {
    position: absolute; bottom: 12px; left: 12px; display: flex; gap: 12px;
    font-size: 11px; color: var(--pg-muted); background: var(--pg-surface);
    border: 1px solid var(--pg-border); border-radius: 8px; padding: 6px 12px;
    pointer-events: none; z-index: 10; font-family: "SF Mono", "Fira Code", monospace;
}
.pg__status span { white-space: nowrap; }
.section { margin-bottom: 6px; border: 1px solid var(--pg-border); border-radius: var(--pg-radius); overflow: hidden; }
.section__head {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 12px; font-size: 12px; font-weight: 600; cursor: pointer;
    user-select: none; color: var(--pg-text); background: var(--pg-surface);
    text-transform: uppercase; letter-spacing: 0.04em;
}
.section__head:hover { background: var(--pg-accent-light); }
.section__chevron { font-size: 10px; color: var(--pg-muted); transition: transform 0.15s; }
.section__chevron.open { transform: rotate(90deg); }
.section__body { padding: 8px 12px 12px; border-top: 1px solid var(--pg-border); }
.ctrl {
    display: flex; align-items: center; justify-content: space-between;
    gap: 8px; padding: 5px 0; font-size: 12px;
}
.ctrl label { color: var(--pg-text); flex-shrink: 0; }
.ctrl input[type="text"], .ctrl input[type="number"], .ctrl select {
    width: 130px; padding: 5px 8px; border-radius: 6px;
    border: 1px solid var(--pg-border); background: var(--pg-bg);
    color: var(--pg-text); font-size: 12px;
}
.ctrl input[type="text"]:focus, .ctrl input[type="number"]:focus, .ctrl select:focus {
    outline: 2px solid color-mix(in srgb, var(--pg-accent) 45%, transparent);
    border-color: var(--pg-accent);
}
.ctrl input[type="checkbox"] { width: 16px; height: 16px; accent-color: var(--pg-accent); }
.palette { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; margin-bottom: 8px; }
.palette__item {
    display: flex; align-items: center; gap: 8px; padding: 8px 10px;
    border: 1px solid var(--pg-border); border-radius: 8px; cursor: pointer;
    font-size: 12px; font-weight: 500; background: var(--pg-surface);
}
.palette__item:hover { background: var(--pg-accent-light); border-color: var(--pg-accent); }
.palette__dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
.btn-row { display: flex; gap: 6px; flex-wrap: wrap; }
.btn {
    padding: 6px 12px; border: 1px solid var(--pg-border); border-radius: 8px;
    background: var(--pg-surface); font-size: 11px; font-weight: 500;
    cursor: pointer; color: var(--pg-text);
}
.btn:hover { background: var(--pg-accent-light); border-color: var(--pg-accent); }
.btn--accent { background: var(--pg-accent); color: #fff; border-color: var(--pg-accent); }
.btn--accent:hover { opacity: 0.9; }
.btn--danger { color: var(--pg-danger); border-color: var(--pg-danger); }
.btn--danger:hover { background: #fef2f2; }
[data-theme="dark"] .btn--danger:hover { background: rgba(239,68,68,0.12); }
.preset-row { display: flex; gap: 6px; flex-wrap: wrap; margin-bottom: 12px; }
.preset {
    padding: 6px 10px; border: 1px solid var(--pg-border); border-radius: 999px;
    font-size: 11px; font-weight: 500; cursor: pointer; background: var(--pg-surface);
    color: var(--pg-text);
}
.preset:hover { background: var(--pg-accent-light); border-color: var(--pg-accent); }
.preset.active { background: var(--pg-accent); color: #fff; border-color: var(--pg-accent); }
.inspector { font-size: 12px; }
.inspector__empty { color: var(--pg-muted); padding: 20px 0; text-align: center; }
.inspector__field { display: flex; justify-content: space-between; padding: 4px 0; border-bottom: 1px solid var(--pg-border); }
.inspector__key { color: var(--pg-muted); }
.inspector__val { font-weight: 500; font-family: "SF Mono", monospace; font-size: 11px; }
.log { font-family: "SF Mono", "Fira Code", monospace; font-size: 11px; }
.log__empty { color: var(--pg-muted); padding: 20px 0; text-align: center; }
.log__entry { padding: 3px 0; border-bottom: 1px solid var(--pg-border); word-break: break-all; color: var(--pg-muted); }
.log__clear { margin-bottom: 8px; }
.custom-node {
    min-width: 120px; min-height: 40px; width: 100%; height: 100%; position: relative;
    padding: 10px; border-radius: var(--df-node-border-radius);
    border: var(--df-node-border); background: var(--df-node-background-color);
    box-shadow: none; display: flex;
    align-items: center; justify-content: center; color: var(--pg-text);
    box-sizing: border-box;
}
.pg__flow .dioxus-flow__node.selected .custom-node { border-color: var(--pg-accent); }
.custom-node__title { font-weight: 500; font-size: 12px; line-height: 1.1; }
.custom-toolbar {
    display: flex; gap: 6px; background: var(--pg-text); color: var(--pg-bg);
    padding: 4px 10px; border-radius: 999px; font-size: 10px; font-weight: 500;
}
@media (max-width: 1024px) {
    .pg__side { width: 300px; min-width: 300px; }
}
@media (max-width: 860px) {
    .pg { flex-direction: column; }
    .pg__side {
        width: 100%;
        min-width: 100%;
        height: 46vh;
        border-right: none;
        border-bottom: 1px solid var(--pg-border);
    }
    .pg__canvas { height: 54vh; }
}
"#;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab { Build, Settings, Log }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Preset { Empty, Basic, Pipeline, Tree, KitchenSink }

#[derive(Clone, Copy)]
struct CustomNodeHandleFlags {
    left_target: bool,
    right_source: bool,
    top_target: bool,
    bottom_source: bool,
}

impl Default for CustomNodeHandleFlags {
    fn default() -> Self {
        Self {
            left_target: true,
            right_source: true,
            top_target: false,
            bottom_source: false,
        }
    }
}

#[derive(Clone, Copy)]
struct CustomNodeUiConfig {
    handle_overrides: Signal<HashMap<String, CustomNodeHandleFlags>>,
}

impl Preset {
    fn label(&self) -> &'static str {
        match self {
            Preset::Empty => "Empty",
            Preset::Basic => "Basic",
            Preset::Pipeline => "Pipeline",
            Preset::Tree => "Tree",
            Preset::KitchenSink => "Kitchen Sink",
        }
    }
    fn load(&self) -> (Vec<Node<()>>, Vec<Edge<()>>) {
        match self {
            Preset::Empty => preset_empty(),
            Preset::Basic => preset_basic(),
            Preset::Pipeline => preset_pipeline(),
            Preset::Tree => preset_tree(),
            Preset::KitchenSink => preset_kitchen_sink(),
        }
    }
}

const ALL_PRESETS: [Preset; 5] = [
    Preset::Empty, Preset::Basic, Preset::Pipeline, Preset::Tree, Preset::KitchenSink,
];

#[allow(non_snake_case)]
fn App() -> Element {
    let flow_css = include_str!("../src/styles/dioxus-flow.css");

    let active_preset = use_signal(|| Preset::KitchenSink);
    let (initial_n, initial_e) = Preset::KitchenSink.load();
    let (nodes, mut on_nodes_change) = use_nodes_state(initial_n);
    let (edges, mut on_edges_change) = use_edges_state(initial_e);
    let flow_instance = use_signal(|| None::<dioxus_flow::hooks::FlowInstance<(), ()>>);
    let node_counter = use_signal(|| 100u32);

    let mut active_tab = use_signal(|| Tab::Build);
    let dark_mode = use_signal(|| false);
    let open_sections = use_signal(|| {
        let mut s = HashSet::new();
        s.insert("interaction".to_string());
        s.insert("ui".to_string());
        s
    });

    let mut event_log: Signal<Vec<String>> = use_signal(Vec::new);

    let inspected_node = use_signal(|| None::<String>);
    let inspected_edge = use_signal(|| None::<String>);

    let viewport_info = use_signal(|| Viewport::new(0.0, 0.0, 1.0));

    let show_background = use_signal(|| true);
    let background_variant = use_signal(|| BackgroundVariant::Dots);
    let show_controls = use_signal(|| true);
    let show_minimap = use_signal(|| true);
    let minimap_w = use_signal(|| 180.0);
    let minimap_h = use_signal(|| 120.0);

    let selection_mode = use_signal(|| SelectionMode::Partial);
    let selection_on_drag = use_signal(|| true);
    let connection_mode = use_signal(|| ConnectionMode::Strict);
    let connection_line_type = use_signal(|| ConnectionLineType::Bezier);
    let pan_on_scroll_mode = use_signal(|| PanOnScrollMode::Free);

    let nodes_draggable = use_signal(|| true);
    let nodes_connectable = use_signal(|| true);
    let nodes_focusable = use_signal(|| true);
    let edges_focusable = use_signal(|| true);
    let edges_reconnectable = use_signal(|| true);
    let elements_selectable = use_signal(|| true);
    let select_nodes_on_drag = use_signal(|| true);

    let default_node_draggable = use_signal(|| true);
    let default_node_selectable = use_signal(|| true);
    let default_node_connectable = use_signal(|| true);
    let default_node_focusable = use_signal(|| true);
    let default_node_deletable = use_signal(|| true);
    let default_node_source_position = use_signal(|| Position::Right);
    let default_node_target_position = use_signal(|| Position::Left);
    let custom_node_handle_overrides = use_signal(HashMap::<String, CustomNodeHandleFlags>::new);

    let default_edge_animated = use_signal(|| false);
    let default_edge_selectable = use_signal(|| true);
    let default_edge_focusable = use_signal(|| true);
    let default_edge_deletable = use_signal(|| true);
    let default_edge_reconnectable = use_signal(|| true);
    let default_edge_arrow = use_signal(|| true);
    let default_edge_label = use_signal(String::new);
    let default_edge_label_show_bg = use_signal(|| true);

    let only_render_visible = use_signal(|| false);
    let visible_area_padding = use_signal(|| 0.2);

    let snap_to_grid = use_signal(|| false);
    let snap_grid_x = use_signal(|| 20.0);
    let snap_grid_y = use_signal(|| 20.0);

    let zoom_on_scroll = use_signal(|| true);
    let zoom_on_pinch = use_signal(|| true);
    let zoom_on_double_click = use_signal(|| true);
    let pan_on_drag = use_signal(|| true);
    let pan_on_scroll = use_signal(|| true);
    let pan_on_scroll_speed = use_signal(|| 0.5);
    let prevent_scrolling = use_signal(|| true);

    let min_zoom = use_signal(|| 0.5);
    let max_zoom = use_signal(|| 2.0);
    let connection_radius = use_signal(|| 20.0);
    let reconnect_radius = use_signal(|| 10.0);
    let node_drag_threshold = use_signal(|| 1.0);
    let connection_drag_threshold = use_signal(|| 1.0);
    let connect_on_click = use_signal(|| true);

    let auto_pan_on_node_drag = use_signal(|| true);
    let auto_pan_on_connect = use_signal(|| true);
    let auto_pan_speed = use_signal(|| 15.0);

    let custom_connection_line = use_signal(|| false);
    let custom_validation = use_signal(|| true);

    let delete_key_input = use_signal(|| "Backspace".to_string());
    let selection_key_input = use_signal(|| "Shift".to_string());
    let multi_selection_key_input = use_signal(|| "Meta,Control".to_string());
    let pan_activation_input = use_signal(|| "Space".to_string());
    let zoom_activation_input = use_signal(|| "Control".to_string());

    let translate_extent_enabled = use_signal(|| false);
    let translate_min_x = use_signal(|| -200.0);
    let translate_min_y = use_signal(|| -200.0);
    let translate_max_x = use_signal(|| 1200.0);
    let translate_max_y = use_signal(|| 800.0);

    let is_valid_connection = if *custom_validation.read() {
        Some(validate_connection as IsValidConnection)
    } else {
        None
    };
    let connection_line_component: Option<Component<ConnectionLineProps>> =
        if *custom_connection_line.read() { Some(CustomConnectionLine) } else { None };

    let delete_key_code = parse_key_list(&delete_key_input.read());
    let selection_key_code = parse_key_list(&selection_key_input.read());
    let multi_selection_key_code = parse_key_list(&multi_selection_key_input.read());
    let pan_activation_key_code = parse_key_list(&pan_activation_input.read());
    let zoom_activation_key_code = parse_key_list(&zoom_activation_input.read());

    let translate_extent = if *translate_extent_enabled.read() {
        Some([
            [*translate_min_x.read(), *translate_min_y.read()],
            [*translate_max_x.read(), *translate_max_y.read()],
        ])
    } else {
        None
    };

    let color_mode = if *dark_mode.read() { ColorMode::Dark } else { ColorMode::Light };

    use_context_provider(move || CustomNodeUiConfig { handle_overrides: custom_node_handle_overrides });

    let mut node_types: HashMap<String, Component<dioxus_flow::components::NodeProps<(), ()>>> =
        HashMap::new();
    node_types.insert("default".into(), CustomNode);
    node_types.insert("input".into(), CustomNode);
    node_types.insert("output".into(), CustomNode);
    node_types.insert("group".into(), GroupNode);
    node_types.insert("custom".into(), CustomNode);

    let mut edge_types: HashMap<String, Component<dioxus_flow::components::EdgeComponentProps<()>>> =
        HashMap::new();
    edge_types.insert("custom-edge".into(), CustomEdge);

    let on_connect = {
        let mut edges = edges.clone();
        let mut event_log = event_log.clone();
        let connection_line_type = connection_line_type.clone();
        let default_edge_animated = default_edge_animated.clone();
        let default_edge_selectable = default_edge_selectable.clone();
        let default_edge_focusable = default_edge_focusable.clone();
        let default_edge_deletable = default_edge_deletable.clone();
        let default_edge_reconnectable = default_edge_reconnectable.clone();
        let default_edge_arrow = default_edge_arrow.clone();
        let default_edge_label = default_edge_label.clone();
        let default_edge_label_show_bg = default_edge_label_show_bg.clone();
        move |connection: Connection| {
            push_log(&mut event_log, "connect", format!("{} -> {}", connection.source, connection.target));
            let mut edge = connection_to_edge::<()>(
                &connection,
                edge_type_for_line_type(*connection_line_type.read()),
            );
            edge.animated = *default_edge_animated.read();
            edge.selectable = Some(*default_edge_selectable.read());
            edge.focusable = Some(*default_edge_focusable.read());
            edge.deletable = Some(*default_edge_deletable.read());
            edge.reconnectable = Some(if *default_edge_reconnectable.read() {
                ReconnectableValue::True
            } else {
                ReconnectableValue::False
            });
            edge.marker_end = if *default_edge_arrow.read() {
                Some(EdgeMarker::arrow())
            } else {
                None
            };
            let label = default_edge_label.read().trim().to_string();
            edge.label = if label.is_empty() { None } else { Some(label) };
            edge.label_show_bg = Some(*default_edge_label_show_bg.read());
            let next = add_edge(edge, edges.read().clone());
            edges.set(next);
        }
    };

    let on_nodes_change_handler = {
        let mut event_log = event_log.clone();
        move |changes: Vec<NodeChange<()>>| {
            if changes.len() <= 3 {
                for c in &changes {
                    let desc = match c {
                        NodeChange::Position { id, .. } => format!("position {id}"),
                        NodeChange::Selection { id, selected } => format!("select {id}={selected}"),
                        NodeChange::Remove { id } => format!("remove {id}"),
                        NodeChange::Add { node } => format!("add {}", node.id),
                        NodeChange::Dimensions { id, .. } => format!("dims {id}"),
                        _ => "change".into(),
                    };
                    push_log(&mut event_log, "node", desc);
                }
            }
            on_nodes_change(changes);
        }
    };

    let on_edges_change_handler = {
        let mut event_log = event_log.clone();
        move |changes: Vec<EdgeChange<()>>| {
            if changes.len() <= 3 {
                for c in &changes {
                    let desc = match c {
                        EdgeChange::Selection { id, selected } => format!("select {id}={selected}"),
                        EdgeChange::Remove { id } => format!("remove {id}"),
                        EdgeChange::Add { edge } => format!("add {}", edge.id),
                        _ => "change".into(),
                    };
                    push_log(&mut event_log, "edge", desc);
                }
            }
            on_edges_change(changes);
        }
    };

    let on_node_click = {
        let mut inspected_node = inspected_node.clone();
        let mut inspected_edge = inspected_edge.clone();
        let mut active_tab = active_tab.clone();
        let mut event_log = event_log.clone();
        move |evt: NodeMouseEvent<()>| {
            push_log(&mut event_log, "click", format!("node {}", evt.node.id));
            inspected_node.set(Some(evt.node.id.clone()));
            inspected_edge.set(None);
            active_tab.set(Tab::Settings);
        }
    };

    let on_edge_click = {
        let mut inspected_node = inspected_node.clone();
        let mut inspected_edge = inspected_edge.clone();
        let mut event_log = event_log.clone();
        move |evt: EdgeMouseEvent<()>| {
            push_log(&mut event_log, "click", format!("edge {}", evt.edge.id));
            inspected_edge.set(Some(evt.edge.id.clone()));
            inspected_node.set(None);
        }
    };

    let on_node_drag_stop = {
        let mut event_log = event_log.clone();
        move |evt: NodeDragEvent<()>| {
            push_log(&mut event_log, "drag-stop", format!("{} at ({:.0},{:.0})", evt.node.id, evt.node.position.x, evt.node.position.y));
        }
    };

    let on_selection_change = {
        let mut event_log = event_log.clone();
        move |evt: SelectionChange<(), ()>| {
            push_log(&mut event_log, "selection", format!("{}n {}e", evt.nodes.len(), evt.edges.len()));
        }
    };

    let on_move_end = {
        let mut viewport_info = viewport_info.clone();
        move |vp: Viewport| {
            viewport_info.set(vp);
        }
    };

    let on_edge_update_start = move |_: Edge<()>| {};
    let on_edge_update = move |_: EdgeUpdateEvent<()>| {};
    let on_edge_update_end = move |_: EdgeUpdateEndEvent<()>| {};

    let node_count = nodes.read().len();
    let edge_count = edges.read().len();
    let vp = *viewport_info.read();

    let apply_node_defaults = {
        let mut nodes = nodes.clone();
        let default_node_draggable = default_node_draggable.clone();
        let default_node_selectable = default_node_selectable.clone();
        let default_node_connectable = default_node_connectable.clone();
        let default_node_focusable = default_node_focusable.clone();
        let default_node_deletable = default_node_deletable.clone();
        move |_| {
            let next: Vec<Node<()>> = nodes
                .read()
                .iter()
                .cloned()
                .map(|mut node| {
                    node.draggable = Some(*default_node_draggable.read());
                    node.selectable = Some(*default_node_selectable.read());
                    node.connectable = Some(*default_node_connectable.read());
                    node.focusable = Some(*default_node_focusable.read());
                    node.deletable = Some(*default_node_deletable.read());
                    node
                })
                .collect();
            nodes.set(next);
        }
    };

    let apply_edge_defaults = {
        let mut edges = edges.clone();
        let default_edge_animated = default_edge_animated.clone();
        let default_edge_selectable = default_edge_selectable.clone();
        let default_edge_focusable = default_edge_focusable.clone();
        let default_edge_deletable = default_edge_deletable.clone();
        let default_edge_reconnectable = default_edge_reconnectable.clone();
        let default_edge_arrow = default_edge_arrow.clone();
        let default_edge_label = default_edge_label.clone();
        let default_edge_label_show_bg = default_edge_label_show_bg.clone();
        move |_| {
            let next: Vec<Edge<()>> = edges
                .read()
                .iter()
                .cloned()
                .map(|mut edge| {
                    edge.animated = *default_edge_animated.read();
                    edge.selectable = Some(*default_edge_selectable.read());
                    edge.focusable = Some(*default_edge_focusable.read());
                    edge.deletable = Some(*default_edge_deletable.read());
                    edge.reconnectable = Some(if *default_edge_reconnectable.read() {
                        ReconnectableValue::True
                    } else {
                        ReconnectableValue::False
                    });
                    edge.marker_end = if *default_edge_arrow.read() {
                        Some(EdgeMarker::arrow())
                    } else {
                        None
                    };
                    let label = default_edge_label.read().trim().to_string();
                    edge.label = if label.is_empty() { None } else { Some(label) };
                    edge.label_show_bg = Some(*default_edge_label_show_bg.read());
                    edge
                })
                .collect();
            edges.set(next);
        }
    };

    let add_node = {
        let mut nodes = nodes.clone();
        let mut node_counter = node_counter.clone();
        let default_node_draggable = default_node_draggable.clone();
        let default_node_selectable = default_node_selectable.clone();
        let default_node_connectable = default_node_connectable.clone();
        let default_node_focusable = default_node_focusable.clone();
        let default_node_deletable = default_node_deletable.clone();
        let default_node_source_position = default_node_source_position.clone();
        let default_node_target_position = default_node_target_position.clone();
        move |_| {
            let c = *node_counter.read();
            node_counter.set(c + 1);
            let id = format!("node-{c}");
            let x = 220.0 + (c as f64 * 17.0) % 420.0;
            let y = 120.0 + (c as f64 * 23.0) % 320.0;
            let mut node = flow_node(&id, XYPosition::new(x, y), "custom");
            node.draggable = Some(*default_node_draggable.read());
            node.selectable = Some(*default_node_selectable.read());
            node.connectable = Some(*default_node_connectable.read());
            node.focusable = Some(*default_node_focusable.read());
            node.deletable = Some(*default_node_deletable.read());
            node.source_position = Some(*default_node_source_position.read());
            node.target_position = Some(*default_node_target_position.read());
            let mut v = nodes.read().clone();
            v.push(node);
            nodes.set(v);
        }
    };

    let load_preset = {
        let mut nodes = nodes.clone();
        let mut edges = edges.clone();
        let mut active_preset = active_preset.clone();
        let mut inspected_node = inspected_node.clone();
        let mut inspected_edge = inspected_edge.clone();
        let mut event_log = event_log.clone();
        move |p: Preset| {
            let (n, e) = p.load();
            nodes.set(n);
            edges.set(e);
            active_preset.set(p);
            inspected_node.set(None);
            inspected_edge.set(None);
            push_log(&mut event_log, "preset", format!("loaded {}", p.label()));
        }
    };

    let toggle_section = |_: String| {};

    rsx! {
        div {
            class: "pg",
            "data-theme": if *dark_mode.read() { "dark" } else { "light" },
            style { "{flow_css}\n{PLAYGROUND_CSS}" }

            div { class: "pg__side",
                div { class: "pg__header",
                    div { class: "pg__title", "Dioxus Flow" }
                    button {
                        class: "pg__theme-btn",
                        onclick: move |_| toggle(dark_mode),
                        if *dark_mode.read() { "Light" } else { "Dark" }
                    }
                }

                div { class: "pg__tabs",
                    div {
                        class: if *active_tab.read() == Tab::Build { "pg__tab active" } else { "pg__tab" },
                        onclick: move |_| active_tab.set(Tab::Build),
                        "Build"
                    }
                    div {
                        class: if *active_tab.read() == Tab::Settings { "pg__tab active" } else { "pg__tab" },
                        onclick: move |_| active_tab.set(Tab::Settings),
                        "Settings"
                    }
                    div {
                        class: if *active_tab.read() == Tab::Log { "pg__tab active" } else { "pg__tab" },
                        onclick: move |_| active_tab.set(Tab::Log),
                        "Log"
                    }
                }

                div { class: "pg__body",
                    if *active_tab.read() == Tab::Build {
                        div { style: "margin-bottom: 14px;",
                            div { style: "font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.06em; color: var(--pg-muted); margin-bottom: 6px;", "Presets" }
                            div { class: "preset-row",
                                for p in ALL_PRESETS.iter() {
                                    {
                                        let p = *p;
                                        let mut load_preset = load_preset.clone();
                                        rsx! {
                                            button {
                                                class: if *active_preset.read() == p { "preset active" } else { "preset" },
                                                onclick: move |_| load_preset(p),
                                                "{p.label()}"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        div { style: "margin-bottom: 14px;",
                            div { style: "font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.06em; color: var(--pg-muted); margin-bottom: 6px;", "Actions" }
                            div { class: "btn-row",
                                button {
                                    class: "btn btn--accent",
                                    onclick: add_node,
                                    "Add node"
                                }
                                button {
                                    class: "btn",
                                    onclick: {
                                        let flow_instance = flow_instance.clone();
                                        move |_| {
                                            if let Some(mut inst) = flow_instance.read().clone() {
                                                inst.fit_view(None);
                                            }
                                        }
                                    },
                                    "Fit view"
                                }
                                button {
                                    class: "btn",
                                    onclick: {
                                        let flow_instance = flow_instance.clone();
                                        move |_| {
                                            if let Some(mut inst) = flow_instance.read().clone() {
                                                inst.zoom_in(Some(1.2));
                                            }
                                        }
                                    },
                                    "Zoom +"
                                }
                                button {
                                    class: "btn",
                                    onclick: {
                                        let flow_instance = flow_instance.clone();
                                        move |_| {
                                            if let Some(mut inst) = flow_instance.read().clone() {
                                                inst.zoom_out(Some(1.2));
                                            }
                                        }
                                    },
                                    "Zoom -"
                                }
                                button {
                                    class: "btn btn--danger",
                                    onclick: {
                                        let mut nodes = nodes.clone();
                                        let mut edges = edges.clone();
                                        let mut event_log = event_log.clone();
                                        move |_| {
                                            nodes.set(vec![]);
                                            edges.set(vec![]);
                                            push_log(&mut event_log, "action", "cleared graph".into());
                                        }
                                    },
                                    "Clear"
                                }
                            }
                        }

                    }

                    if *active_tab.read() == Tab::Settings {
                        div { style: "margin-bottom: 14px;",
                            div { style: "font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.06em; color: var(--pg-muted); margin-bottom: 6px;", "Selected Element" }
                            { render_element_editor(nodes, edges, inspected_node, inspected_edge, custom_node_handle_overrides) }
                        }

                        { section(&open_sections, &toggle_section, "ui", "UI Components", rsx! {
                            { checkbox("Show background", show_background) }
                            { select_ctrl("Background", background_variant, &[
                                ("dots", "Dots"), ("lines", "Lines"), ("cross", "Cross"),
                            ], |v| match v { "lines" => BackgroundVariant::Lines, "cross" => BackgroundVariant::Cross, _ => BackgroundVariant::Dots },
                            |v| match v { BackgroundVariant::Lines => "lines", BackgroundVariant::Cross => "cross", _ => "dots" }) }
                            { checkbox("Show controls", show_controls) }
                            { checkbox("Show minimap", show_minimap) }
                            { number("Minimap width", minimap_w, 1.0, 60.0) }
                            { number("Minimap height", minimap_h, 1.0, 60.0) }
                        }) }

                        { section(&open_sections, &toggle_section, "interaction", "Interaction", rsx! {
                            { checkbox("Nodes draggable", nodes_draggable) }
                            { checkbox("Nodes connectable", nodes_connectable) }
                            { checkbox("Nodes focusable", nodes_focusable) }
                            { checkbox("Edges focusable", edges_focusable) }
                            { checkbox("Edges reconnectable", edges_reconnectable) }
                            { checkbox("Elements selectable", elements_selectable) }
                            { checkbox("Select nodes on drag", select_nodes_on_drag) }
                        }) }

                        { section(&open_sections, &toggle_section, "panzoom", "Pan & Zoom", rsx! {
                            { checkbox("Zoom on scroll", zoom_on_scroll) }
                            { checkbox("Zoom on pinch", zoom_on_pinch) }
                            { checkbox("Zoom on double click", zoom_on_double_click) }
                            { checkbox("Pan on drag", pan_on_drag) }
                            { checkbox("Pan on scroll", pan_on_scroll) }
                            { select_ctrl("Pan scroll mode", pan_on_scroll_mode, &[
                                ("free", "Free"), ("horizontal", "Horizontal"), ("vertical", "Vertical"),
                            ], |v| match v { "horizontal" => PanOnScrollMode::Horizontal, "vertical" => PanOnScrollMode::Vertical, _ => PanOnScrollMode::Free },
                            |v| match v { PanOnScrollMode::Horizontal => "horizontal", PanOnScrollMode::Vertical => "vertical", _ => "free" }) }
                            { number("Pan scroll speed", pan_on_scroll_speed, 0.1, 0.0) }
                            { checkbox("Prevent scrolling", prevent_scrolling) }
                            { number("Min zoom", min_zoom, 0.1, 0.01) }
                            { number("Max zoom", max_zoom, 0.1, 0.1) }
                        }) }

                        { section(&open_sections, &toggle_section, "connection", "Connection", rsx! {
                            { select_ctrl("Mode", connection_mode, &[
                                ("strict", "Strict"), ("loose", "Loose"),
                            ], |v| match v { "loose" => ConnectionMode::Loose, _ => ConnectionMode::Strict },
                            |v| match v { ConnectionMode::Loose => "loose", _ => "strict" }) }
                            { select_ctrl("Line type", connection_line_type, &[
                                ("bezier", "Bezier"), ("smoothstep", "SmoothStep"), ("step", "Step"), ("straight", "Straight"), ("simplebezier", "SimpleBezier"),
                            ], |v| match v {
                                "smoothstep" => ConnectionLineType::SmoothStep,
                                "step" => ConnectionLineType::Step,
                                "straight" => ConnectionLineType::Straight,
                                "simplebezier" => ConnectionLineType::SimpleBezier,
                                _ => ConnectionLineType::Bezier,
                            }, |v| match v {
                                ConnectionLineType::SmoothStep => "smoothstep",
                                ConnectionLineType::Step => "step",
                                ConnectionLineType::Straight => "straight",
                                ConnectionLineType::SimpleBezier => "simplebezier",
                                _ => "bezier",
                            }) }
                            { checkbox("Connect on click", connect_on_click) }
                            { number("Connection radius", connection_radius, 1.0, 1.0) }
                            { number("Reconnect radius", reconnect_radius, 1.0, 1.0) }
                            { number("Drag threshold", connection_drag_threshold, 0.5, 0.0) }
                            { checkbox("Custom validation", custom_validation) }
                            { checkbox("Custom connection line", custom_connection_line) }
                        }) }

                        { section(&open_sections, &toggle_section, "selection", "Selection", rsx! {
                            { checkbox("Selection on drag", selection_on_drag) }
                            { select_ctrl("Selection mode", selection_mode, &[
                                ("partial", "Partial"), ("full", "Full"),
                            ], |v| match v { "full" => SelectionMode::Full, _ => SelectionMode::Partial },
                            |v| match v { SelectionMode::Full => "full", _ => "partial" }) }
                        }) }

                        { section(&open_sections, &toggle_section, "grid", "Grid & Snapping", rsx! {
                            { checkbox("Snap to grid", snap_to_grid) }
                            { number("Grid X", snap_grid_x, 5.0, 1.0) }
                            { number("Grid Y", snap_grid_y, 5.0, 1.0) }
                        }) }

                        { section(&open_sections, &toggle_section, "autopan", "Auto Pan", rsx! {
                            { checkbox("On node drag", auto_pan_on_node_drag) }
                            { checkbox("On connect", auto_pan_on_connect) }
                            { number("Speed", auto_pan_speed, 1.0, 1.0) }
                        }) }

                        { section(&open_sections, &toggle_section, "rendering", "Rendering", rsx! {
                            { checkbox("Only render visible", only_render_visible) }
                            { number("Visible padding", visible_area_padding, 0.05, 0.0) }
                            { number("Drag threshold", node_drag_threshold, 0.5, 0.0) }
                        }) }

                        { section(&open_sections, &toggle_section, "keyboard", "Keyboard", rsx! {
                            { text_input("Delete key(s)", delete_key_input) }
                            { text_input("Selection key(s)", selection_key_input) }
                            { text_input("Multi select key(s)", multi_selection_key_input) }
                            { text_input("Pan activation", pan_activation_input) }
                            { text_input("Zoom activation", zoom_activation_input) }
                        }) }

                        { section(&open_sections, &toggle_section, "extent", "Translate Extent", rsx! {
                            { checkbox("Enable extent", translate_extent_enabled) }
                            { number("Min X", translate_min_x, 10.0, f64::NEG_INFINITY) }
                            { number("Min Y", translate_min_y, 10.0, f64::NEG_INFINITY) }
                            { number("Max X", translate_max_x, 10.0, f64::NEG_INFINITY) }
                            { number("Max Y", translate_max_y, 10.0, f64::NEG_INFINITY) }
                        }) }

                        { section(&open_sections, &toggle_section, "node-defaults", "Default Node Props", rsx! {
                            { checkbox("Node draggable", default_node_draggable) }
                            { checkbox("Node selectable", default_node_selectable) }
                            { checkbox("Node connectable", default_node_connectable) }
                            { checkbox("Node focusable", default_node_focusable) }
                            { checkbox("Node deletable", default_node_deletable) }
                            { select_ctrl("Source position", default_node_source_position, &[
                                ("left", "Left"), ("right", "Right"), ("top", "Top"), ("bottom", "Bottom"),
                            ], position_from_str, position_to_str) }
                            { select_ctrl("Target position", default_node_target_position, &[
                                ("left", "Left"), ("right", "Right"), ("top", "Top"), ("bottom", "Bottom"),
                            ], position_from_str, position_to_str) }
                            div { class: "btn-row",
                                button { class: "btn", onclick: apply_node_defaults, "Apply to existing nodes" }
                            }
                        }) }

                        { section(&open_sections, &toggle_section, "edge-defaults", "Default Edge Props", rsx! {
                            { checkbox("Edge animated", default_edge_animated) }
                            { checkbox("Edge selectable", default_edge_selectable) }
                            { checkbox("Edge focusable", default_edge_focusable) }
                            { checkbox("Edge deletable", default_edge_deletable) }
                            { checkbox("Edge reconnectable", default_edge_reconnectable) }
                            { checkbox("Edge arrow marker", default_edge_arrow) }
                            { text_input("Edge label", default_edge_label) }
                            { checkbox("Label show bg", default_edge_label_show_bg) }
                            div { class: "btn-row",
                                button { class: "btn", onclick: apply_edge_defaults, "Apply to existing edges" }
                            }
                        }) }
                    }

                    if *active_tab.read() == Tab::Log {
                        div { class: "log__clear",
                            button {
                                class: "btn",
                                onclick: move |_| event_log.set(vec![]),
                                "Clear log"
                            }
                        }
                        div { class: "log",
                            if event_log.read().is_empty() {
                                div { class: "log__empty", "Events will appear here" }
                            }
                            for entry in event_log.read().iter().rev() {
                                div { class: "log__entry", "{entry}" }
                            }
                        }
                    }
                }
            }

            div { class: "pg__canvas",
                DioxusFlow {
                    nodes: Some(nodes),
                    edges: Some(edges),
                    node_types: Some(node_types),
                    edge_types: Some(edge_types),
                    on_nodes_change: on_nodes_change_handler,
                    on_edges_change: on_edges_change_handler,
                    on_connect,
                    on_edge_update_start,
                    on_edge_update,
                    on_edge_update_end,
                    on_node_click,
                    on_edge_click,
                    on_node_drag_stop,
                    on_selection_change,
                    on_move_end,
                    on_init: {
                        let mut flow_instance = flow_instance.clone();
                        move |instance| flow_instance.set(Some(instance))
                    },
                    color_mode,
                    min_zoom: *min_zoom.read(),
                    max_zoom: *max_zoom.read(),
                    translate_extent,
                    zoom_on_scroll: *zoom_on_scroll.read(),
                    zoom_on_pinch: *zoom_on_pinch.read(),
                    zoom_on_double_click: *zoom_on_double_click.read(),
                    pan_on_drag: *pan_on_drag.read(),
                    pan_on_scroll: *pan_on_scroll.read(),
                    pan_on_scroll_speed: *pan_on_scroll_speed.read(),
                    pan_on_scroll_mode: *pan_on_scroll_mode.read(),
                    prevent_scrolling: *prevent_scrolling.read(),
                    pan_activation_key_code,
                    zoom_activation_key_code,
                    auto_pan_on_node_drag: *auto_pan_on_node_drag.read(),
                    auto_pan_on_connect: *auto_pan_on_connect.read(),
                    auto_pan_speed: *auto_pan_speed.read(),
                    nodes_draggable: *nodes_draggable.read(),
                    snap_to_grid: *snap_to_grid.read(),
                    snap_grid: (*snap_grid_x.read(), *snap_grid_y.read()),
                    nodes_connectable: *nodes_connectable.read(),
                    nodes_focusable: *nodes_focusable.read(),
                    edges_focusable: *edges_focusable.read(),
                    edges_reconnectable: *edges_reconnectable.read(),
                    elements_selectable: *elements_selectable.read(),
                    select_nodes_on_drag: *select_nodes_on_drag.read(),
                    only_render_visible_elements: *only_render_visible.read(),
                    visible_area_padding: *visible_area_padding.read(),
                    selection_on_drag: *selection_on_drag.read(),
                    selection_mode: *selection_mode.read(),
                    connection_mode: Some(*connection_mode.read()),
                    connection_line_type: Some(*connection_line_type.read()),
                    connection_line_component,
                    is_valid_connection,
                    connection_radius: *connection_radius.read(),
                    reconnect_radius: *reconnect_radius.read(),
                    node_drag_threshold: *node_drag_threshold.read(),
                    connection_drag_threshold: *connection_drag_threshold.read(),
                    connect_on_click: *connect_on_click.read(),
                    delete_key_code,
                    selection_key_code,
                    multi_selection_key_code,
                    class: Some("pg__flow".into()),

                    if *show_background.read() {
                        Background {
                            variant: Some(*background_variant.read()),
                            gap: 24.0,
                            size: 1.0,
                        }
                    }
                    if *show_controls.read() {
                        Controls::<(), ()> { show_fit_view: true, show_zoom: true }
                    }
                    if *show_minimap.read() {
                        MiniMap::<(), ()> {
                            width: *minimap_w.read(),
                            height: *minimap_h.read(),
                            node_color: Some(if *dark_mode.read() { "#1f2937".into() } else { "#dbeafe".into() }),
                            node_stroke_color: Some(if *dark_mode.read() { "#4b5563".into() } else { "#93c5fd".into() }),
                            mask_color: Some(if *dark_mode.read() { "rgba(15, 23, 42, 0.55)".into() } else { "rgba(148, 163, 184, 0.28)".into() }),
                            mask_stroke_color: Some(if *dark_mode.read() { "#94a3b8".into() } else { "#64748b".into() }),
                        }
                    }
                }

                div { class: "pg__status",
                    span { "Nodes: {node_count}" }
                    span { "Edges: {edge_count}" }
                    span { "Zoom: {vp.zoom:.2}" }
                    span { "Pan: ({vp.x:.0}, {vp.y:.0})" }
                }
            }
        }
    }
}

fn render_element_editor(
    mut nodes: Signal<Vec<Node<()>>>,
    mut edges: Signal<Vec<Edge<()>>>,
    mut inspected_node: Signal<Option<String>>,
    inspected_edge: Signal<Option<String>>,
    mut custom_node_handle_overrides: Signal<HashMap<String, CustomNodeHandleFlags>>,
) -> Element {
    if let Some(node_id) = inspected_node.read().clone() {
        let node = nodes.read().iter().find(|n| n.id == node_id).cloned();
        if let Some(node) = node {
            let node_type = node.node_type.clone().unwrap_or_else(|| "custom".to_string());
            let source_pos = node.source_position.unwrap_or(Position::Right);
            let target_pos = node.target_position.unwrap_or(Position::Left);
            let handle_flags = custom_node_handle_overrides
                .read()
                .get(&node_id)
                .copied()
                .unwrap_or_default();

            rsx! {
                div { class: "inspector",
                    div { class: "ctrl",
                        label { "Node ID" }
                        input {
                            r#type: "text",
                            value: "{node.id}",
                            oninput: {
                                let old_id = node.id.clone();
                                move |evt| {
                                    let new_id = evt.value().trim().to_string();
                                    if new_id.is_empty() {
                                        let next_nodes: Vec<Node<()>> = nodes
                                            .read()
                                            .iter()
                                            .filter(|n| n.id != old_id)
                                            .cloned()
                                            .collect();
                                        nodes.set(next_nodes);

                                        let next_edges: Vec<Edge<()>> = edges
                                            .read()
                                            .iter()
                                            .filter(|e| e.source != old_id && e.target != old_id)
                                            .cloned()
                                            .collect();
                                        edges.set(next_edges);

                                        let mut hm = custom_node_handle_overrides.read().clone();
                                        hm.remove(&old_id);
                                        custom_node_handle_overrides.set(hm);
                                        inspected_node.set(None);
                                        return;
                                    }
                                    if new_id == old_id {
                                        return;
                                    }
                                    let mut next_nodes = nodes.read().clone();
                                    if next_nodes.iter().any(|n| n.id == new_id) {
                                        return;
                                    }
                                    for n in &mut next_nodes {
                                        if n.id == old_id {
                                            n.id = new_id.clone();
                                        }
                                    }
                                    nodes.set(next_nodes);

                                    let mut next_edges = edges.read().clone();
                                    for e in &mut next_edges {
                                        if e.source == old_id {
                                            e.source = new_id.clone();
                                        }
                                        if e.target == old_id {
                                            e.target = new_id.clone();
                                        }
                                    }
                                    edges.set(next_edges);

                                    let mut hm = custom_node_handle_overrides.read().clone();
                                    if let Some(flags) = hm.remove(&old_id) {
                                        hm.insert(new_id.clone(), flags);
                                        custom_node_handle_overrides.set(hm);
                                    }
                                    inspected_node.set(Some(new_id));
                                }
                            }
                        }
                    }
                    div { class: "ctrl",
                        label { "Type" }
                        select {
                            value: "{node_type}",
                            onchange: {
                                let id = node.id.clone();
                                move |_| {
                                    let next = nodes
                                        .read()
                                        .iter()
                                        .cloned()
                                        .map(|mut n| {
                                            if n.id == id {
                                                n.node_type = Some("custom".to_string());
                                            }
                                            n
                                        })
                                        .collect();
                                    nodes.set(next);
                                }
                            },
                            option { value: "custom", "custom" }
                        }
                    }
                    div { class: "ctrl",
                        label { "Source pos" }
                        select {
                            value: "{position_to_str(&source_pos)}",
                            onchange: {
                                let id = node.id.clone();
                                move |evt| {
                                    let pos = position_from_str(&evt.value());
                                    let next = nodes.read().iter().cloned().map(|mut n| {
                                        if n.id == id { n.source_position = Some(pos); }
                                        n
                                    }).collect();
                                    nodes.set(next);
                                }
                            },
                            option { value: "left", "left" }
                            option { value: "right", "right" }
                            option { value: "top", "top" }
                            option { value: "bottom", "bottom" }
                        }
                    }
                    div { class: "ctrl",
                        label { "Target pos" }
                        select {
                            value: "{position_to_str(&target_pos)}",
                            onchange: {
                                let id = node.id.clone();
                                move |evt| {
                                    let pos = position_from_str(&evt.value());
                                    let next = nodes.read().iter().cloned().map(|mut n| {
                                        if n.id == id { n.target_position = Some(pos); }
                                        n
                                    }).collect();
                                    nodes.set(next);
                                }
                            },
                            option { value: "left", "left" }
                            option { value: "right", "right" }
                            option { value: "top", "top" }
                            option { value: "bottom", "bottom" }
                        }
                    }
                    div { class: "ctrl",
                        label { "Draggable" }
                        input {
                            r#type: "checkbox",
                            checked: node.draggable.unwrap_or(true),
                            onchange: {
                                let id = node.id.clone();
                                move |_| {
                                    let next = nodes.read().iter().cloned().map(|mut n| {
                                        if n.id == id {
                                            n.draggable = Some(!n.draggable.unwrap_or(true));
                                        }
                                        n
                                    }).collect();
                                    nodes.set(next);
                                }
                            }
                        }
                    }
                    div { class: "ctrl",
                        label { "Connectable" }
                        input {
                            r#type: "checkbox",
                            checked: node.connectable.unwrap_or(true),
                            onchange: {
                                let id = node.id.clone();
                                move |_| {
                                    let next = nodes.read().iter().cloned().map(|mut n| {
                                        if n.id == id {
                                            n.connectable = Some(!n.connectable.unwrap_or(true));
                                        }
                                        n
                                    }).collect();
                                    nodes.set(next);
                                }
                            }
                        }
                    }
                    div { class: "ctrl",
                        label { "Focusable" }
                        input {
                            r#type: "checkbox",
                            checked: node.focusable.unwrap_or(true),
                            onchange: {
                                let id = node.id.clone();
                                move |_| {
                                    let next = nodes.read().iter().cloned().map(|mut n| {
                                        if n.id == id {
                                            n.focusable = Some(!n.focusable.unwrap_or(true));
                                        }
                                        n
                                    }).collect();
                                    nodes.set(next);
                                }
                            }
                        }
                    }
                    div { class: "ctrl",
                        label { "Deletable" }
                        input {
                            r#type: "checkbox",
                            checked: node.deletable.unwrap_or(true),
                            onchange: {
                                let id = node.id.clone();
                                move |_| {
                                    let next = nodes.read().iter().cloned().map(|mut n| {
                                        if n.id == id {
                                            n.deletable = Some(!n.deletable.unwrap_or(true));
                                        }
                                        n
                                    }).collect();
                                    nodes.set(next);
                                }
                            }
                        }
                    }

                    if node.node_type.as_deref() == Some("custom") {
                        div { class: "ctrl",
                            label { "Left target" }
                            input {
                                r#type: "checkbox",
                                checked: handle_flags.left_target,
                                onchange: {
                                    let id = node.id.clone();
                                    move |_| {
                                        let mut hm = custom_node_handle_overrides.read().clone();
                                        let mut flags = hm.get(&id).copied().unwrap_or_default();
                                        flags.left_target = !flags.left_target;
                                        hm.insert(id.clone(), flags);
                                        custom_node_handle_overrides.set(hm);
                                    }
                                }
                            }
                        }
                        div { class: "ctrl",
                            label { "Right source" }
                            input {
                                r#type: "checkbox",
                                checked: handle_flags.right_source,
                                onchange: {
                                    let id = node.id.clone();
                                    move |_| {
                                        let mut hm = custom_node_handle_overrides.read().clone();
                                        let mut flags = hm.get(&id).copied().unwrap_or_default();
                                        flags.right_source = !flags.right_source;
                                        hm.insert(id.clone(), flags);
                                        custom_node_handle_overrides.set(hm);
                                    }
                                }
                            }
                        }
                        div { class: "ctrl",
                            label { "Top target" }
                            input {
                                r#type: "checkbox",
                                checked: handle_flags.top_target,
                                onchange: {
                                    let id = node.id.clone();
                                    move |_| {
                                        let mut hm = custom_node_handle_overrides.read().clone();
                                        let mut flags = hm.get(&id).copied().unwrap_or_default();
                                        flags.top_target = !flags.top_target;
                                        hm.insert(id.clone(), flags);
                                        custom_node_handle_overrides.set(hm);
                                    }
                                }
                            }
                        }
                        div { class: "ctrl",
                            label { "Bottom source" }
                            input {
                                r#type: "checkbox",
                                checked: handle_flags.bottom_source,
                                onchange: {
                                    let id = node.id.clone();
                                    move |_| {
                                        let mut hm = custom_node_handle_overrides.read().clone();
                                        let mut flags = hm.get(&id).copied().unwrap_or_default();
                                        flags.bottom_source = !flags.bottom_source;
                                        hm.insert(id.clone(), flags);
                                        custom_node_handle_overrides.set(hm);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            rsx! { div { class: "inspector__empty", "Selected node no longer exists" } }
        }
    } else if let Some(edge_id) = inspected_edge.read().clone() {
        let edge = edges.read().iter().find(|e| e.id == edge_id).cloned();
        if let Some(edge) = edge {
            let edge_type = edge.edge_type.clone().unwrap_or_else(|| "bezier".to_string());
            rsx! {
                div { class: "inspector",
                    div { class: "ctrl",
                        label { "Edge label" }
                        input {
                            r#type: "text",
                            value: "{edge.label.clone().unwrap_or_default()}",
                            oninput: {
                                let id = edge.id.clone();
                                move |evt| {
                                    let v = evt.value();
                                    let next = edges.read().iter().cloned().map(|mut e| {
                                        if e.id == id {
                                            let t = v.trim().to_string();
                                            e.label = if t.is_empty() { None } else { Some(t) };
                                        }
                                        e
                                    }).collect();
                                    edges.set(next);
                                }
                            }
                        }
                    }
                    div { class: "ctrl",
                        label { "Edge type" }
                        select {
                            value: "{edge_type}",
                            onchange: {
                                let id = edge.id.clone();
                                move |evt| {
                                    let t = evt.value();
                                    let next = edges.read().iter().cloned().map(|mut e| {
                                        if e.id == id {
                                            e.edge_type = Some(t.clone());
                                        }
                                        e
                                    }).collect();
                                    edges.set(next);
                                }
                            },
                            option { value: "bezier", "bezier" }
                            option { value: "smoothstep", "smoothstep" }
                            option { value: "step", "step" }
                            option { value: "straight", "straight" }
                            option { value: "simplebezier", "simplebezier" }
                            option { value: "custom-edge", "custom-edge" }
                        }
                    }
                    div { class: "ctrl",
                        label { "Animated" }
                        input {
                            r#type: "checkbox",
                            checked: edge.animated,
                            onchange: {
                                let id = edge.id.clone();
                                move |_| {
                                    let next = edges.read().iter().cloned().map(|mut e| {
                                        if e.id == id { e.animated = !e.animated; }
                                        e
                                    }).collect();
                                    edges.set(next);
                                }
                            }
                        }
                    }
                    div { class: "ctrl",
                        label { "Reconnectable" }
                        input {
                            r#type: "checkbox",
                            checked: !matches!(edge.reconnectable, Some(ReconnectableValue::False)),
                            onchange: {
                                let id = edge.id.clone();
                                move |_| {
                                    let next = edges.read().iter().cloned().map(|mut e| {
                                        if e.id == id {
                                            e.reconnectable = Some(if matches!(e.reconnectable, Some(ReconnectableValue::False)) {
                                                ReconnectableValue::True
                                            } else {
                                                ReconnectableValue::False
                                            });
                                        }
                                        e
                                    }).collect();
                                    edges.set(next);
                                }
                            }
                        }
                    }
                    div { class: "ctrl",
                        label { "Arrow marker" }
                        input {
                            r#type: "checkbox",
                            checked: edge.marker_end.is_some(),
                            onchange: {
                                let id = edge.id.clone();
                                move |_| {
                                    let next = edges.read().iter().cloned().map(|mut e| {
                                        if e.id == id {
                                            e.marker_end = if e.marker_end.is_some() { None } else { Some(EdgeMarker::arrow()) };
                                        }
                                        e
                                    }).collect();
                                    edges.set(next);
                                }
                            }
                        }
                    }
                }
            }
        } else {
            rsx! { div { class: "inspector__empty", "Selected edge no longer exists" } }
        }
    } else {
        rsx! {
            div { class: "inspector",
                div { class: "inspector__empty", "Click a node or edge to configure it" }
            }
        }
    }
}

fn section(
    open: &Signal<HashSet<String>>,
    _toggle_section: &dyn Fn(String),
    key: &str,
    title: &str,
    children: Element,
) -> Element {
    let is_open = open.read().contains(key);
    let key_owned = key.to_string();
    let mut open = open.clone();
    rsx! {
        div { class: "section",
            div {
                class: "section__head",
                onclick: move |_| {
                    let mut s = open.read().clone();
                    let k = key_owned.clone();
                    if s.contains(&k) { s.remove(&k); } else { s.insert(k); }
                    open.set(s);
                },
                span { "{title}" }
                span { class: if is_open { "section__chevron open" } else { "section__chevron" }, ">" }
            }
            if is_open {
                div { class: "section__body", {children} }
            }
        }
    }
}

fn checkbox(label: &str, signal: Signal<bool>) -> Element {
    rsx! {
        div { class: "ctrl",
            label { "{label}" }
            input {
                r#type: "checkbox",
                checked: *signal.read(),
                onchange: move |_| toggle(signal),
            }
        }
    }
}

fn number(label: &str, mut signal: Signal<f64>, step: f64, min: f64) -> Element {
    let step_str = format!("{step}");
    rsx! {
        div { class: "ctrl",
            label { "{label}" }
            input {
                r#type: "number",
                value: "{signal.read()}",
                step: "{step_str}",
                oninput: move |evt| {
                    if let Ok(v) = evt.value().parse::<f64>() {
                        signal.set(v.max(min));
                    }
                },
            }
        }
    }
}

fn text_input(label: &str, mut signal: Signal<String>) -> Element {
    rsx! {
        div { class: "ctrl",
            label { "{label}" }
            input {
                r#type: "text",
                value: "{signal.read()}",
                oninput: move |evt| signal.set(evt.value()),
            }
        }
    }
}

fn select_ctrl<T: Clone + PartialEq + 'static>(
    label: &str,
    mut signal: Signal<T>,
    options: &[(&str, &str)],
    from_str: fn(&str) -> T,
    to_str: fn(&T) -> &str,
) -> Element {
    let current = to_str(&signal.read()).to_string();
    let opts: Vec<(String, String)> = options.iter().map(|(v, l)| (v.to_string(), l.to_string())).collect();
    rsx! {
        div { class: "ctrl",
            label { "{label}" }
            select {
                value: "{current}",
                onchange: move |evt| {
                    signal.set(from_str(&evt.value()));
                },
                for (val, display) in opts.iter() {
                    option { value: "{val}", selected: *val == current, "{display}" }
                }
            }
        }
    }
}

#[component]
fn CustomNode(props: dioxus_flow::components::NodeProps<(), ()>) -> Element {
    let cfg = use_context::<CustomNodeUiConfig>();
    let title = props.node.id.clone();
    let flags = cfg
        .handle_overrides
        .read()
        .get(&props.node.id)
        .copied()
        .unwrap_or_default();
    rsx! {
        div {
            class: "custom-node",
            if flags.left_target {
                Handle::<(), ()> {
                    node_id: props.node.id.clone(),
                    handle_type: HandleType::Target,
                    position: Position::Left,
                    id: Some("in".into()),
                    is_connectable: props.connectable,
                }
            }
            if flags.right_source {
                Handle::<(), ()> {
                    node_id: props.node.id.clone(),
                    handle_type: HandleType::Source,
                    position: Position::Right,
                    id: Some("out".into()),
                    is_connectable: props.connectable,
                }
            }
            if flags.top_target {
                Handle::<(), ()> {
                    node_id: props.node.id.clone(),
                    handle_type: HandleType::Target,
                    position: Position::Top,
                    id: Some("top".into()),
                    is_connectable: props.connectable,
                }
            }
            if flags.bottom_source {
                Handle::<(), ()> {
                    node_id: props.node.id.clone(),
                    handle_type: HandleType::Source,
                    position: Position::Bottom,
                    id: Some("bottom".into()),
                    is_connectable: props.connectable,
                }
            }
            div { class: "custom-node__title", "{title}" }
        }
    }
}

#[component]
fn CustomEdge(props: dioxus_flow::components::EdgeComponentProps<()>) -> Element {
    let path_result = dioxus_flow::utils::get_smooth_step_path(
        props.source_x, props.source_y,
        props.target_x, props.target_y,
        props.source_position, props.target_position,
        Some(10.0),
        None,
        None,
    );

    rsx! {
        BaseEdge {
            path: path_result.path,
            label: props.edge.label.clone(),
            label_x: Some(path_result.label_x),
            label_y: Some(path_result.label_y),
            label_show_bg: Some(true),
        }
        EdgeToolbar::<(), ()> {
            edge_id: props.edge.id.clone(),
            x: path_result.label_x,
            y: path_result.label_y,
            class: Some("custom-toolbar".into()),
            div { "Edge" }
        }
    }
}

#[component]
fn CustomConnectionLine(props: ConnectionLineProps) -> Element {
    let path = match props.connection_line_type {
        ConnectionLineType::Straight => {
            dioxus_flow::utils::get_straight_path(
                props.from_x,
                props.from_y,
                props.to_x,
                props.to_y,
            )
            .path
        }
        ConnectionLineType::Step => {
            dioxus_flow::utils::get_step_path(
                props.from_x,
                props.from_y,
                props.to_x,
                props.to_y,
                props.from_position,
                props.to_position,
                None,
            )
            .path
        }
        ConnectionLineType::SmoothStep => {
            dioxus_flow::utils::get_smooth_step_path(
                props.from_x,
                props.from_y,
                props.to_x,
                props.to_y,
                props.from_position,
                props.to_position,
                None,
                None,
                None,
            )
            .path
        }
        ConnectionLineType::SimpleBezier => {
            dioxus_flow::utils::get_simple_bezier_path(
                props.from_x,
                props.from_y,
                props.to_x,
                props.to_y,
                props.from_position,
                props.to_position,
            )
            .path
        }
        ConnectionLineType::Bezier => {
            dioxus_flow::utils::get_bezier_path(
                props.from_x,
                props.from_y,
                props.to_x,
                props.to_y,
                props.from_position,
                props.to_position,
                None,
            )
            .path
        }
    };
    let class = if props.is_valid { "dioxus-flow__connection valid" } else { "dioxus-flow__connection invalid" };

    rsx! {
        svg {
            class: "{class}",
            width: "100%",
            height: "100%",
            path {
                class: "dioxus-flow__connection-path",
                style: "stroke-dasharray: 6 6;",
                d: "{path}",
            }
        }
    }
}
