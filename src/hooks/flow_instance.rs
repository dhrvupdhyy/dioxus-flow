//! Flow instance helper

use crate::state::FlowState;
use crate::types::{FitBoundsOptions, FitViewOptions, Rect, SetCenterOptions, XYPosition};

#[derive(Clone)]
pub struct FlowInstance<
    N: Clone + PartialEq + Default + 'static = (),
    E: Clone + PartialEq + Default + 'static = (),
> {
    state: FlowState<N, E>,
}

impl<N, E> FlowInstance<N, E>
where
    N: Clone + PartialEq + Default + 'static,
    E: Clone + PartialEq + Default + 'static,
{
    pub fn new(state: FlowState<N, E>) -> Self {
        Self { state }
    }

    pub fn zoom_in(&mut self, factor: Option<f64>) {
        self.state.zoom_in(factor);
    }

    pub fn zoom_out(&mut self, factor: Option<f64>) {
        self.state.zoom_out(factor);
    }

    pub fn set_center(&mut self, x: f64, y: f64, options: Option<SetCenterOptions>) {
        self.state.set_center(x, y, options);
    }

    pub fn fit_view(&mut self, options: Option<FitViewOptions>) {
        self.state.fit_view(options);
    }

    pub fn fit_bounds(&mut self, bounds: Rect, options: Option<FitBoundsOptions>) {
        self.state.fit_bounds(bounds, options);
    }

    pub fn screen_to_flow_position(&self, position: XYPosition) -> XYPosition {
        self.state.screen_to_flow_position(position)
    }

    pub fn flow_to_screen_position(&self, position: XYPosition) -> XYPosition {
        self.state.flow_to_screen_position(position)
    }
}
