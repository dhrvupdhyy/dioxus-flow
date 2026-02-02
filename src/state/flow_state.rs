//! Flow state management using Dioxus signals

use crate::types::*;
use dioxus::prelude::*;
use dioxus::prelude::{ReadableExt, WritableExt};
use js_sys::Date;
use std::collections::HashMap;
use wasm_bindgen::JsCast;

/// Main flow state that holds all reactive data
#[derive(Clone)]
pub struct FlowState<
    N: Clone + PartialEq + Default + 'static = (),
    E: Clone + PartialEq + Default + 'static = (),
> {
    // Core data
    pub nodes: Signal<Vec<Node<N>>>,
    pub edges: Signal<Vec<Edge<E>>>,

    // Lookups for O(1) access
    pub node_lookup: Signal<HashMap<String, InternalNode<N>>>,
    pub edge_lookup: Signal<HashMap<String, Edge<E>>>,
    pub parent_lookup: Signal<HashMap<String, Vec<String>>>,

    // Viewport state
    pub viewport: Signal<Viewport>,
    pub width: Signal<f64>,
    pub height: Signal<f64>,
    pub min_zoom: Signal<f64>,
    pub max_zoom: Signal<f64>,
    pub translate_extent: Signal<Option<CoordinateExtent>>,

    // Interaction state
    pub nodes_draggable: Signal<bool>,
    pub nodes_connectable: Signal<bool>,
    pub nodes_focusable: Signal<bool>,
    pub edges_focusable: Signal<bool>,
    pub edges_reconnectable: Signal<bool>,
    pub elements_selectable: Signal<bool>,
    pub only_render_visible_elements: Signal<bool>,
    pub visible_area_padding: Signal<f64>,
    pub selection_change_handlers:
        Signal<Vec<(usize, EventHandler<crate::types::SelectionChange<N, E>>)>>,
    pub selection_change_handler_id: Signal<usize>,

    // Selection state
    pub multi_selection_active: Signal<bool>,
    pub nodes_selection_active: Signal<bool>,
    pub user_selection_active: Signal<bool>,
    pub user_selection_rect: Signal<Option<Rect>>,

    // Connection state
    pub connection: Signal<ConnectionState>,
    pub connection_mode: Signal<ConnectionMode>,
    pub connection_radius: Signal<f64>,
    pub connection_line_type: Signal<ConnectionLineType>,
    pub connection_line_style: Signal<Option<String>>,
    pub connection_line_component: Signal<Option<Component<crate::types::ConnectionLineProps>>>,
    pub is_valid_connection: Signal<Option<IsValidConnection>>,

    // Grid/snapping
    pub snap_to_grid: Signal<bool>,
    pub snap_grid: Signal<(f64, f64)>,

    // Panning state
    pub panning: Signal<bool>,
    pub pan_on_drag: Signal<bool>,
    pub pan_on_scroll: Signal<bool>,
    pub pan_on_scroll_mode: Signal<PanOnScrollMode>,
    pub zoom_on_scroll: Signal<bool>,
    pub zoom_on_pinch: Signal<bool>,
    pub zoom_on_double_click: Signal<bool>,

    // Selection config
    pub selection_on_drag: Signal<bool>,
    pub selection_key_pressed: Signal<bool>,
    pub multi_selection_key_pressed: Signal<bool>,
    pub selection_mode: Signal<SelectionMode>,
    pub node_extent: Signal<Option<CoordinateExtent>>,
    pub focused_node_id: Signal<Option<String>>,

    // Delete key
    pub delete_key_pressed: Signal<bool>,

    // Node dragging
    pub node_drag: Signal<Option<NodeDragState>>,

    // Internal markers
    _node_marker: std::marker::PhantomData<N>,
    _edge_marker: std::marker::PhantomData<E>,
}

impl<N, E> FlowState<N, E>
where
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
{
    pub fn new() -> Self {
        Self {
            nodes: Signal::new(vec![]),
            edges: Signal::new(vec![]),
            node_lookup: Signal::new(HashMap::new()),
            edge_lookup: Signal::new(HashMap::new()),
            parent_lookup: Signal::new(HashMap::new()),
            viewport: Signal::new(Viewport::identity()),
            width: Signal::new(0.0),
            height: Signal::new(0.0),
            min_zoom: Signal::new(0.5),
            max_zoom: Signal::new(2.0),
            translate_extent: Signal::new(None),
            nodes_draggable: Signal::new(true),
            nodes_connectable: Signal::new(true),
            nodes_focusable: Signal::new(true),
            edges_focusable: Signal::new(true),
            edges_reconnectable: Signal::new(true),
            elements_selectable: Signal::new(true),
            only_render_visible_elements: Signal::new(false),
            visible_area_padding: Signal::new(0.2),
            selection_change_handlers: Signal::new(Vec::new()),
            selection_change_handler_id: Signal::new(0),
            multi_selection_active: Signal::new(false),
            nodes_selection_active: Signal::new(false),
            user_selection_active: Signal::new(false),
            user_selection_rect: Signal::new(None),
            connection: Signal::new(ConnectionState::default()),
            connection_mode: Signal::new(ConnectionMode::Strict),
            connection_radius: Signal::new(20.0),
            connection_line_type: Signal::new(ConnectionLineType::Bezier),
            connection_line_style: Signal::new(None),
            connection_line_component: Signal::new(None),
            is_valid_connection: Signal::new(None),
            snap_to_grid: Signal::new(false),
            snap_grid: Signal::new((15.0, 15.0)),
            panning: Signal::new(false),
            pan_on_drag: Signal::new(true),
            pan_on_scroll: Signal::new(false),
            pan_on_scroll_mode: Signal::new(PanOnScrollMode::Free),
            zoom_on_scroll: Signal::new(true),
            zoom_on_pinch: Signal::new(true),
            zoom_on_double_click: Signal::new(true),
            selection_on_drag: Signal::new(false),
            selection_key_pressed: Signal::new(false),
            multi_selection_key_pressed: Signal::new(false),
            selection_mode: Signal::new(SelectionMode::Partial),
            node_extent: Signal::new(None),
            focused_node_id: Signal::new(None),
            delete_key_pressed: Signal::new(false),
            node_drag: Signal::new(None),
            _node_marker: std::marker::PhantomData,
            _edge_marker: std::marker::PhantomData,
        }
    }

    /// Initialize state with nodes and edges
    pub fn init(&mut self, nodes: Vec<Node<N>>, edges: Vec<Edge<E>>) {
        self.set_nodes(nodes);
        self.set_edges(edges);
    }

    /// Set nodes and rebuild lookups
    pub fn set_nodes(&mut self, nodes: Vec<Node<N>>) {
        let previous = self.node_lookup.read().clone();
        let mut node_lookup = HashMap::new();
        let mut parent_lookup: HashMap<String, Vec<String>> = HashMap::new();

        for node in &nodes {
            let previous_bounds = previous
                .get(&node.id)
                .and_then(|internal| internal.handle_bounds.clone());
            let internal = InternalNode {
                position_absolute: self.compute_absolute_position(&node, &nodes),
                dimensions: node.get_dimensions(),
                handle_bounds: previous_bounds,
                node: node.clone(),
            };
            node_lookup.insert(node.id.clone(), internal);

            if let Some(parent_id) = &node.parent_id {
                parent_lookup
                    .entry(parent_id.clone())
                    .or_default()
                    .push(node.id.clone());
            }
        }

        self.nodes.set(nodes);
        self.node_lookup.set(node_lookup);
        self.parent_lookup.set(parent_lookup);
    }

    /// Set edges and rebuild lookup
    pub fn set_edges(&mut self, edges: Vec<Edge<E>>) {
        let edge_lookup: HashMap<String, Edge<E>> =
            edges.iter().map(|e| (e.id.clone(), e.clone())).collect();

        self.edges.set(edges);
        self.edge_lookup.set(edge_lookup);
    }

    /// Compute absolute position including parent offsets
    fn compute_absolute_position(&self, node: &Node<N>, all_nodes: &[Node<N>]) -> XYPosition {
        let mut position = node.position;

        if let Some(parent_id) = &node.parent_id {
            if let Some(parent) = all_nodes.iter().find(|n| &n.id == parent_id) {
                let parent_pos = self.compute_absolute_position(parent, all_nodes);
                position = position + parent_pos;
            }
        }

        position
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<Node<N>> {
        self.nodes.read().iter().find(|n| n.id == id).cloned()
    }

    /// Get an edge by ID
    pub fn get_edge(&self, id: &str) -> Option<Edge<E>> {
        self.edges.read().iter().find(|e| e.id == id).cloned()
    }

    /// Get internal node by ID
    pub fn get_internal_node(&self, id: &str) -> Option<InternalNode<N>> {
        self.node_lookup.read().get(id).cloned()
    }

    /// Update a single node
    pub fn update_node<F>(&mut self, id: &str, f: F)
    where
        F: FnOnce(&mut Node<N>),
    {
        let mut nodes = self.nodes.write();
        if let Some(node) = nodes.iter_mut().find(|n| n.id == id) {
            f(node);
        }
    }

    /// Update a single edge
    pub fn update_edge<F>(&mut self, id: &str, f: F)
    where
        F: FnOnce(&mut Edge<E>),
    {
        let mut edges = self.edges.write();
        if let Some(edge) = edges.iter_mut().find(|e| e.id == id) {
            f(edge);
        }
    }

    /// Apply node changes
    pub fn apply_node_changes(&mut self, changes: Vec<NodeChange<N>>) {
        let nodes = self.nodes.read().clone();
        let new_nodes = apply_node_changes(changes, nodes);
        self.set_nodes(new_nodes);
    }

    /// Apply edge changes
    pub fn apply_edge_changes(&mut self, changes: Vec<EdgeChange<E>>) {
        let edges = self.edges.read().clone();
        let new_edges = apply_edge_changes(changes, edges);
        self.set_edges(new_edges);
    }

    /// Update internal node values (dimensions/absolute position) for a set of node ids.
    pub fn update_node_internals(&mut self, node_ids: impl IntoIterator<Item = String>) {
        let nodes = self.nodes.read().clone();
        let mut updates = Vec::new();

        for node_id in node_ids {
            if let Some(node) = nodes.iter().find(|n| n.id == node_id) {
                let existing = self.node_lookup.read().get(&node.id).cloned();
                let internal = InternalNode {
                    position_absolute: self.compute_absolute_position(node, &nodes),
                    dimensions: node.get_dimensions(),
                    handle_bounds: existing.and_then(|i| i.handle_bounds.clone()),
                    node: node.clone(),
                };
                updates.push((node.id.clone(), internal));
            }
        }

        if updates.is_empty() {
            return;
        }

        let mut lookup = self.node_lookup.write();
        for (id, internal) in updates {
            lookup.insert(id, internal);
        }
    }

    pub fn update_handle_bounds(&mut self, node_id: &str, bounds: HandleBounds) {
        let mut lookup = self.node_lookup.write();
        if let Some(internal) = lookup.get_mut(node_id) {
            internal.handle_bounds = Some(bounds);
        }
    }

    /// Get selected nodes
    pub fn get_selected_nodes(&self) -> Vec<Node<N>> {
        self.nodes
            .read()
            .iter()
            .filter(|n| n.selected)
            .cloned()
            .collect()
    }

    /// Get selected edges
    pub fn get_selected_edges(&self) -> Vec<Edge<E>> {
        self.edges
            .read()
            .iter()
            .filter(|e| e.selected)
            .cloned()
            .collect()
    }

    /// Get visible nodes (not hidden, within viewport)
    pub fn get_visible_nodes(&self) -> Vec<Node<N>> {
        let viewport = *self.viewport.read();
        let width = *self.width.read();
        let height = *self.height.read();

        let padding = *self.visible_area_padding.read();
        let pad_x = (width / viewport.zoom) * padding;
        let pad_y = (height / viewport.zoom) * padding;
        let view_rect = Rect {
            x: -viewport.x / viewport.zoom,
            y: -viewport.y / viewport.zoom,
            width: width / viewport.zoom,
            height: height / viewport.zoom,
        };
        let view_rect = Rect {
            x: view_rect.x - pad_x,
            y: view_rect.y - pad_y,
            width: view_rect.width + pad_x * 2.0,
            height: view_rect.height + pad_y * 2.0,
        };

        self.node_lookup
            .read()
            .values()
            .filter(|internal| {
                if internal.node.hidden {
                    return false;
                }
                let dims = internal.dimensions;
                let node_rect = Rect {
                    x: internal.position_absolute.x,
                    y: internal.position_absolute.y,
                    width: dims.width,
                    height: dims.height,
                };
                view_rect.intersects(&node_rect)
            })
            .map(|internal| internal.node.clone())
            .collect()
    }

    /// Get all edges connected to a node
    pub fn get_connected_edges(&self, node_id: &str) -> Vec<Edge<E>> {
        self.edges
            .read()
            .iter()
            .filter(|e| e.source == node_id || e.target == node_id)
            .cloned()
            .collect()
    }

    /// Get edges between two nodes
    pub fn get_edges_between(&self, source: &str, target: &str) -> Vec<Edge<E>> {
        self.edges
            .read()
            .iter()
            .filter(|e| {
                (e.source == source && e.target == target)
                    || (e.source == target && e.target == source)
            })
            .cloned()
            .collect()
    }

    /// Zoom in by a factor
    pub fn zoom_in(&mut self, factor: Option<f64>) {
        let factor = factor.unwrap_or(1.2);
        let current = *self.viewport.read();
        let max_zoom = *self.max_zoom.read();
        let next = Viewport {
            zoom: (current.zoom * factor).min(max_zoom),
            ..current
        };
        let clamped = self.clamp_viewport(next);
        self.viewport.set(clamped);
    }

    /// Zoom out by a factor
    pub fn zoom_out(&mut self, factor: Option<f64>) {
        let factor = factor.unwrap_or(1.2);
        let current = *self.viewport.read();
        let min_zoom = *self.min_zoom.read();
        let next = Viewport {
            zoom: (current.zoom / factor).max(min_zoom),
            ..current
        };
        let clamped = self.clamp_viewport(next);
        self.viewport.set(clamped);
    }

    /// Set zoom level
    pub fn set_zoom(&mut self, zoom: f64) {
        let min_zoom = *self.min_zoom.read();
        let max_zoom = *self.max_zoom.read();
        let current = *self.viewport.read();
        let next = Viewport {
            zoom: zoom.clamp(min_zoom, max_zoom),
            ..current
        };
        let clamped = self.clamp_viewport(next);
        self.viewport.set(clamped);
    }

    /// Pan by delta
    pub fn pan_by(&mut self, delta: XYPosition) {
        let current = *self.viewport.read();
        let next = Viewport {
            x: current.x + delta.x,
            y: current.y + delta.y,
            zoom: current.zoom,
        };
        let clamped = self.clamp_viewport(next);
        self.viewport.set(clamped);
    }

    /// Set viewport center
    pub fn set_center(&mut self, x: f64, y: f64, options: Option<crate::types::SetCenterOptions>) {
        let width = *self.width.read();
        let height = *self.height.read();
        let zoom = options
            .as_ref()
            .and_then(|o| o.zoom)
            .unwrap_or(self.viewport.read().zoom);

        let next = Viewport {
            x: width / 2.0 - x * zoom,
            y: height / 2.0 - y * zoom,
            zoom,
        };
        let clamped = self.clamp_viewport(next);
        self.set_viewport(clamped, options.and_then(|o| o.duration));
    }

    /// Fit view to show all nodes
    pub fn fit_view(&mut self, options: Option<FitViewOptions>) {
        let options = options.unwrap_or_default();
        let nodes_all = self.nodes.read().clone();
        let nodes = filter_fit_view_nodes(&nodes_all, &options);
        if nodes.is_empty() {
            return;
        }

        let padding = options.padding.unwrap_or(0.1);

        let bounds = get_nodes_bounds(&nodes);
        if bounds.width == 0.0 || bounds.height == 0.0 {
            return;
        }

        let width = *self.width.read();
        let height = *self.height.read();
        let min_zoom = options.min_zoom.unwrap_or(*self.min_zoom.read());
        let max_zoom = options.max_zoom.unwrap_or(*self.max_zoom.read());

        let x_zoom = width / bounds.width / (1.0 + padding * 2.0);
        let y_zoom = height / bounds.height / (1.0 + padding * 2.0);
        let zoom = x_zoom.min(y_zoom).clamp(min_zoom, max_zoom);

        let x = (width - bounds.width * zoom) / 2.0 - bounds.x * zoom;
        let y = (height - bounds.height * zoom) / 2.0 - bounds.y * zoom;

        let clamped = self.clamp_viewport(Viewport { x, y, zoom });
        self.set_viewport(clamped, options.duration);
    }

    /// Fit bounds to a specific rectangle.
    pub fn fit_bounds(&mut self, bounds: Rect, options: Option<FitBoundsOptions>) {
        if bounds.width == 0.0 || bounds.height == 0.0 {
            return;
        }
        let options = options.unwrap_or_default();
        let padding = options.padding.unwrap_or(0.1);
        let width = *self.width.read();
        let height = *self.height.read();
        let min_zoom = *self.min_zoom.read();
        let max_zoom = *self.max_zoom.read();

        let x_zoom = width / bounds.width / (1.0 + padding * 2.0);
        let y_zoom = height / bounds.height / (1.0 + padding * 2.0);
        let zoom = x_zoom.min(y_zoom).clamp(min_zoom, max_zoom);

        let x = (width - bounds.width * zoom) / 2.0 - bounds.x * zoom;
        let y = (height - bounds.height * zoom) / 2.0 - bounds.y * zoom;

        let clamped = self.clamp_viewport(Viewport { x, y, zoom });
        self.set_viewport(clamped, options.duration);
    }

    /// Screen position to flow position
    pub fn screen_to_flow_position(&self, position: XYPosition) -> XYPosition {
        let viewport = self.viewport.read();
        XYPosition {
            x: (position.x - viewport.x) / viewport.zoom,
            y: (position.y - viewport.y) / viewport.zoom,
        }
    }

    pub fn set_viewport(&mut self, viewport: Viewport, duration: Option<u32>) {
        if let Some(duration) = duration {
            if duration == 0 {
                self.viewport.set(viewport);
            } else {
                self.animate_viewport(viewport, duration);
            }
        } else {
            self.viewport.set(viewport);
        }
    }

    fn animate_viewport(&mut self, target: Viewport, duration: u32) {
        let start = *self.viewport.read();
        let duration_ms = duration as f64;
        let Some(window) = web_sys::window() else {
            self.viewport.set(target);
            return;
        };
        let start_time = Date::now();
        let mut state = self.clone();

        let raf: std::rc::Rc<
            std::cell::RefCell<Option<wasm_bindgen::closure::Closure<dyn FnMut(f64)>>>,
        > = std::rc::Rc::new(std::cell::RefCell::new(None));
        let raf_clone = raf.clone();
        let raf_loop = raf.clone();
        *raf_clone.borrow_mut() = Some(wasm_bindgen::closure::Closure::wrap(Box::new(
            move |time: f64| {
                let mut t = (time - start_time) / duration_ms;
                if t < 0.0 {
                    t = 0.0;
                }
                if t > 1.0 {
                    t = 1.0;
                }

                let eased = Self::ease_in_out_cubic(t);
                let lerp = |a: f64, b: f64| a + (b - a) * eased;
                let next = Viewport {
                    x: lerp(start.x, target.x),
                    y: lerp(start.y, target.y),
                    zoom: lerp(start.zoom, target.zoom),
                };
                state.viewport.set(next);

                if t < 1.0 {
                    if let Some(window) = web_sys::window() {
                        if let Some(callback) = raf_loop.borrow().as_ref() {
                            let _ =
                                window.request_animation_frame(callback.as_ref().unchecked_ref());
                        }
                    }
                } else {
                    raf_loop.borrow_mut().take();
                }
            },
        )));

        let _ = window.request_animation_frame(
            raf_clone
                .borrow()
                .as_ref()
                .unwrap()
                .as_ref()
                .unchecked_ref(),
        );
    }

    fn ease_in_out_cubic(t: f64) -> f64 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }

    /// Flow position to screen position
    pub fn flow_to_screen_position(&self, position: XYPosition) -> XYPosition {
        let viewport = self.viewport.read();
        XYPosition {
            x: position.x * viewport.zoom + viewport.x,
            y: position.y * viewport.zoom + viewport.y,
        }
    }

    /// Clamp viewport to translate extent if configured.
    pub fn clamp_viewport(&self, viewport: Viewport) -> Viewport {
        let extent = *self.translate_extent.read();
        if extent.is_none() {
            return viewport;
        }
        let extent = extent.unwrap();
        let width = *self.width.read();
        let height = *self.height.read();
        if width == 0.0 || height == 0.0 {
            return viewport;
        }

        let min_x = extent[0][0];
        let min_y = extent[0][1];
        let max_x = extent[1][0];
        let max_y = extent[1][1];
        let zoom = viewport.zoom;

        let mut min_screen_x = width - max_x * zoom;
        let mut max_screen_x = -min_x * zoom;
        let mut min_screen_y = height - max_y * zoom;
        let mut max_screen_y = -min_y * zoom;

        if min_screen_x > max_screen_x {
            std::mem::swap(&mut min_screen_x, &mut max_screen_x);
        }
        if min_screen_y > max_screen_y {
            std::mem::swap(&mut min_screen_y, &mut max_screen_y);
        }

        Viewport {
            x: viewport.x.clamp(min_screen_x, max_screen_x),
            y: viewport.y.clamp(min_screen_y, max_screen_y),
            zoom: viewport.zoom,
        }
    }

    /// Delete selected elements
    pub fn delete_selected(&mut self) {
        let selected_node_ids: Vec<String> = self
            .nodes
            .read()
            .iter()
            .filter(|n| n.selected && n.deletable.unwrap_or(true))
            .map(|n| n.id.clone())
            .collect();

        let selected_edge_ids: Vec<String> = self
            .edges
            .read()
            .iter()
            .filter(|e| e.selected && e.deletable.unwrap_or(true))
            .map(|e| e.id.clone())
            .collect();

        // Also delete edges connected to deleted nodes
        let edges_to_delete: Vec<String> = self
            .edges
            .read()
            .iter()
            .filter(|e| {
                selected_edge_ids.contains(&e.id)
                    || selected_node_ids.contains(&e.source)
                    || selected_node_ids.contains(&e.target)
            })
            .map(|e| e.id.clone())
            .collect();

        let node_changes: Vec<NodeChange<N>> = selected_node_ids
            .into_iter()
            .map(NodeChange::remove)
            .collect();

        let edge_changes: Vec<EdgeChange<E>> = edges_to_delete
            .into_iter()
            .map(EdgeChange::remove)
            .collect();

        self.apply_node_changes(node_changes);
        self.apply_edge_changes(edge_changes);
    }

    /// Select all elements
    pub fn select_all(&mut self) {
        let node_changes: Vec<NodeChange<N>> = self
            .nodes
            .read()
            .iter()
            .filter(|n| !n.selected && n.selectable.unwrap_or(true))
            .map(|n| NodeChange::select(n.id.clone(), true))
            .collect();

        let edge_changes: Vec<EdgeChange<E>> = self
            .edges
            .read()
            .iter()
            .filter(|e| !e.selected && e.selectable.unwrap_or(true))
            .map(|e| EdgeChange::select(e.id.clone(), true))
            .collect();

        self.apply_node_changes(node_changes);
        self.apply_edge_changes(edge_changes);
    }

    /// Deselect all elements
    pub fn deselect_all(&mut self) {
        let node_changes: Vec<NodeChange<N>> = self
            .nodes
            .read()
            .iter()
            .filter(|n| n.selected)
            .map(|n| NodeChange::select(n.id.clone(), false))
            .collect();

        let edge_changes: Vec<EdgeChange<E>> = self
            .edges
            .read()
            .iter()
            .filter(|e| e.selected)
            .map(|e| EdgeChange::select(e.id.clone(), false))
            .collect();

        self.apply_node_changes(node_changes);
        self.apply_edge_changes(edge_changes);
    }

    /// Register a selection change handler.
    pub fn add_selection_change_handler(
        &mut self,
        handler: EventHandler<crate::types::SelectionChange<N, E>>,
    ) -> usize {
        let mut next_id = self.selection_change_handler_id.write();
        let id = *next_id;
        *next_id = id + 1;
        let mut handlers = self.selection_change_handlers.write();
        handlers.push((id, handler));
        id
    }

    pub fn remove_selection_change_handler(&mut self, id: usize) {
        let mut handlers = self.selection_change_handlers.write();
        handlers.retain(|(handler_id, _)| *handler_id != id);
    }
}

impl<N, E> Default for FlowState<N, E>
where
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Get bounds of a set of nodes
pub fn get_nodes_bounds<N: Clone + PartialEq + Default>(nodes: &[Node<N>]) -> Rect {
    if nodes.is_empty() {
        return Rect::default();
    }

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for node in nodes {
        if node.hidden {
            continue;
        }
        let dims = node.get_dimensions();
        min_x = min_x.min(node.position.x);
        min_y = min_y.min(node.position.y);
        max_x = max_x.max(node.position.x + dims.width);
        max_y = max_y.max(node.position.y + dims.height);
    }

    if min_x == f64::MAX {
        return Rect::default();
    }

    Rect {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    }
}

/// Node ID context for child components
#[derive(Clone, PartialEq)]
pub struct NodeIdContext(pub String);

/// Drag state for moving one or more nodes.
#[derive(Clone, PartialEq, Debug)]
pub struct NodeDragState {
    pub origin_node_id: String,
    pub start_pointer: XYPosition,
    pub nodes: Vec<(String, XYPosition)>,
}

/// Get incomers (nodes that connect to the given node)
pub fn get_incomers<N, E>(node: &Node<N>, nodes: &[Node<N>], edges: &[Edge<E>]) -> Vec<Node<N>>
where
    N: Clone + PartialEq + Default,
    E: Clone + PartialEq + Default,
{
    let incoming_edges: Vec<_> = edges.iter().filter(|e| e.target == node.id).collect();

    incoming_edges
        .iter()
        .filter_map(|edge| nodes.iter().find(|n| n.id == edge.source))
        .cloned()
        .collect()
}

/// Get outgoers (nodes that the given node connects to)
pub fn get_outgoers<N, E>(node: &Node<N>, nodes: &[Node<N>], edges: &[Edge<E>]) -> Vec<Node<N>>
where
    N: Clone + PartialEq + Default,
    E: Clone + PartialEq + Default,
{
    let outgoing_edges: Vec<_> = edges.iter().filter(|e| e.source == node.id).collect();

    outgoing_edges
        .iter()
        .filter_map(|edge| nodes.iter().find(|n| n.id == edge.target))
        .cloned()
        .collect()
}

/// Get all edges connected to a set of nodes
pub fn get_connected_edges_for_nodes<N, E>(nodes: &[Node<N>], edges: &[Edge<E>]) -> Vec<Edge<E>>
where
    N: Clone + PartialEq + Default,
    E: Clone + PartialEq + Default,
{
    let node_ids: std::collections::HashSet<_> = nodes.iter().map(|n| &n.id).collect();

    edges
        .iter()
        .filter(|edge| node_ids.contains(&edge.source) || node_ids.contains(&edge.target))
        .cloned()
        .collect()
}

fn filter_fit_view_nodes<N: Clone + PartialEq + Default>(
    nodes: &[Node<N>],
    options: &FitViewOptions,
) -> Vec<Node<N>> {
    let mut filtered: Vec<Node<N>> = if let Some(ids) = &options.nodes {
        nodes
            .iter()
            .filter(|node| ids.contains(&node.id))
            .cloned()
            .collect()
    } else {
        nodes.to_vec()
    };

    if !options.include_hidden_nodes {
        filtered.retain(|node| !node.hidden);
    }

    filtered
}

/// Add edge with duplicate check
pub fn add_edge<E: Clone + PartialEq + Default>(
    edge: Edge<E>,
    mut edges: Vec<Edge<E>>,
) -> Vec<Edge<E>> {
    let exists = edges.iter().any(|e| {
        e.source == edge.source
            && e.target == edge.target
            && e.source_handle == edge.source_handle
            && e.target_handle == edge.target_handle
    });

    if !exists {
        edges.push(edge);
    }

    edges
}

/// Create an edge from a connection
pub fn connection_to_edge<E: Clone + PartialEq + Default>(
    connection: &Connection,
    edge_type: Option<String>,
) -> Edge<E> {
    let id = format!(
        "e{}-{}-{}-{}",
        connection.source,
        connection.source_handle.as_deref().unwrap_or(""),
        connection.target,
        connection.target_handle.as_deref().unwrap_or("")
    );

    Edge {
        id,
        source: connection.source.clone(),
        target: connection.target.clone(),
        source_handle: connection.source_handle.clone(),
        target_handle: connection.target_handle.clone(),
        edge_type,
        ..Default::default()
    }
}
