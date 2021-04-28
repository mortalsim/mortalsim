mod initializer;
mod connector;
pub use initializer::ClosedCircInitializer;
pub use connector::ClosedCircConnector;
use crate::core::sim::{SimComponent, SimComponentInitializer, SimConnector};

pub trait ClosedCircSimComponent {
    /// Initializes the component. Should register any `Event` objects to listen for
    /// and set initial state.
    /// 
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the component
    fn init(&mut self, initializer: ClosedCircInitializer);
    
    /// Runs an iteration of this component. Will be called anytime a `notify` registered
    /// `Event` changes on `Sim` state. All module logic should ideally occur within this
    /// call and all resulting `Event` objects scheduled for future emission.
    /// 
    /// Note that all `Event`s previously scheduled by this component which have not yet
    /// occurred will be unscheduled before `run` is executed.
    /// 
    /// ### Arguments
    /// * `connector` - Helper object for the component to interact with the rest of
    ///                 the simulation
    fn run(&mut self, connector: &mut ClosedCircConnector);
}

impl SimComponent for dyn ClosedCircSimComponent {
    fn init(&mut self, _: &mut SimComponentInitializer) {}
    fn run(&mut self, _: &mut SimConnector) {}
}