pub mod registry;
pub mod wrapper;

use self::registry::ComponentRegistry;
use super::SimConnector;

/// Trait to be used by any modules for Sim objects
pub trait SimComponent {
    /// The unique id of the component
    fn id(&self) -> &'static str;
    /// Attaches the module to the ComponentKeeper
    fn attach(self, registry: &mut ComponentRegistry);
    /// Runs an iteration of this module.
    fn run(&mut self);
}

/// Trait to outline common methods for all systems that
/// process `SimComponent`s
pub trait SimComponentProcessor<T: SimComponent> {
    /// Execute initial setup for a component
    fn setup_component(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Prepare a component for their run, and indicate if they should trigger a run
    fn prepare_component(&mut self, connector: &SimConnector, component: &mut T) -> bool;
    /// Process a component after their run
    fn process_component(&mut self, connector: &mut SimConnector, component: &mut T);
}
