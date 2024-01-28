pub mod registry;

use self::registry::ComponentRegistry;
use super::SimConnector;
use super::organism::Organism;

/// Common trait for all simulation components
pub trait SimComponent<O: Organism> {
    /// The unique id of the component
    fn id(&self) -> &'static str;
    /// Attaches the module to the ComponentRegistry
    fn attach(self, registry: &mut ComponentRegistry<O>);
    /// Runs an iteration of this module.
    fn run(&mut self);
}

/// Trait to outline common methods for all layers that
/// process `SimComponent`s
pub trait SimComponentProcessor<O: Organism, T: SimComponent<O>> {
    /// Execute initial setup for a component
    fn setup_component(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Update prior to component processing
    fn pre_exec(&mut self, connector: &mut SimConnector);
    /// Indicate if the given component should trigger a run
    fn check_component(&mut self, component: &T) -> bool;
    /// Prepare a component for their run
    fn prepare_component(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Process a component after their run.
    fn process_component(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Update after component processing
    fn post_exec(&mut self, connector: &mut SimConnector);
}
