//! Selection listener component

use dioxus::prelude::ReadableExt;
use dioxus::prelude::*;

use crate::state::FlowState;
use crate::types::SelectionChange;

#[component]
pub fn SelectionListener<
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
>(
    #[props(default)] on_selection_change: Option<EventHandler<SelectionChange<N, E>>>,
) -> Element {
    let state = use_context::<FlowState<N, E>>();
    let mut last_nodes = use_signal(|| Vec::<String>::new());
    let mut last_edges = use_signal(|| Vec::<String>::new());

    use_effect(move || {
        let node_ids: Vec<String> = state
            .nodes
            .read()
            .iter()
            .filter(|n| n.selected)
            .map(|n| n.id.clone())
            .collect();
        let edge_ids: Vec<String> = state
            .edges
            .read()
            .iter()
            .filter(|e| e.selected)
            .map(|e| e.id.clone())
            .collect();

        if *last_nodes.read() == node_ids && *last_edges.read() == edge_ids {
            return;
        }

        last_nodes.set(node_ids);
        last_edges.set(edge_ids);

        let change = SelectionChange {
            nodes: state
                .nodes
                .read()
                .iter()
                .filter(|n| n.selected)
                .cloned()
                .collect(),
            edges: state
                .edges
                .read()
                .iter()
                .filter(|e| e.selected)
                .cloned()
                .collect(),
        };

        let mut sinks: Vec<EventHandler<SelectionChange<N, E>>> = Vec::new();
        if let Some(handler) = on_selection_change.clone() {
            sinks.push(handler);
        }
        sinks.extend(
            state
                .selection_change_handlers
                .read()
                .iter()
                .map(|(_, handler)| handler.clone()),
        );

        if sinks.is_empty() {
            return;
        }

        let last = sinks.len() - 1;
        let mut maybe_change = Some(change);
        for (index, handler) in sinks.into_iter().enumerate() {
            if index == last {
                if let Some(final_change) = maybe_change.take() {
                    handler.call(final_change);
                }
            } else if let Some(current) = maybe_change.as_ref() {
                handler.call(current.clone());
            }
        }
    });

    rsx! {}
}
