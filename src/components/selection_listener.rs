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
        let selected_nodes: Vec<_> = state
            .nodes
            .read()
            .iter()
            .filter(|n| n.selected)
            .cloned()
            .collect();
        let selected_edges: Vec<_> = state
            .edges
            .read()
            .iter()
            .filter(|e| e.selected)
            .cloned()
            .collect();

        let node_ids: Vec<String> = selected_nodes.iter().map(|n| n.id.clone()).collect();
        let edge_ids: Vec<String> = selected_edges.iter().map(|e| e.id.clone()).collect();

        if *last_nodes.read() != node_ids || *last_edges.read() != edge_ids {
            last_nodes.set(node_ids);
            last_edges.set(edge_ids);

            let change = SelectionChange {
                nodes: selected_nodes,
                edges: selected_edges,
            };

            if let Some(handler) = &on_selection_change {
                handler.call(change.clone());
            }

            let handlers = state.selection_change_handlers.read().clone();
            for (_, handler) in handlers {
                handler.call(change.clone());
            }
        }
    });

    rsx! {}
}
