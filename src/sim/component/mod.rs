pub(crate) mod registry;
pub(crate) mod factory;

use super::organism::Organism;
use super::SimConnector;

pub use registry::ComponentRegistry;
pub use factory::ComponentFactory;

/// Common trait for all simulation components
pub trait SimComponent<O: Organism>: Send {
    /// The unique id of the component
    fn id(&self) -> &'static str;
    /// Attaches the module to the ComponentRegistry
    fn attach(self, registry: &mut ComponentRegistry<O>);
    /// Runs an iteration of this module.
    fn run(&mut self);
}

/// Trait to outline common methods for all layers that
/// process `SimComponent`s
pub trait SimComponentProcessor<O: Organism, T: SimComponent<O> + ?Sized> {
    /// Execute initial setup for a component
    fn setup_component(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Indicate if the given component should trigger a run
    fn check_component(&mut self, component: &T) -> bool;
    /// Prepare a component for their run
    fn prepare_component(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Process a component after their run.
    fn process_component(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Execute removal of a component
    fn remove_component(&mut self, connector: &mut SimConnector, component: &mut T);
}

/// Trait to outline common methods for all layers that
/// process `SimComponent`s (thread safe)
pub trait SimComponentProcessorSync<O: Organism, T: SimComponent<O> + ?Sized> {
    /// Execute initial setup for a component (thread safe)
    fn setup_component_sync(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Indicate if the given component should trigger a run (thread safe)
    fn check_component_sync(&mut self, component: &T) -> bool;
    /// Prepare a component for their run (thread safe)
    fn prepare_component_sync(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Process a component after their run. (thread safe)
    fn process_component_sync(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Execute removal of a component
    fn remove_component_sync(&mut self, connector: &mut SimConnector, component: &mut T);
}
