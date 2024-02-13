pub mod registry;

use self::registry::ComponentRegistry;
use super::organism::Organism;
use super::SimConnector;

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
pub trait SimComponentProcessor<O: Organism, T: SimComponent<O>> {
    /// Execute initial setup for a component
    fn setup_component(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Indicate if the given component should trigger a run
    fn check_component(&mut self, component: &T) -> bool;
    /// Prepare a component for their run
    fn prepare_component(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Process a component after their run.
    fn process_component(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Execute initial setup for a component (thread safe)
    fn setup_component_sync(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Indicate if the given component should trigger a run (thread safe)
    fn check_component_sync(&mut self, component: &T) -> bool;
    /// Prepare a component for their run (thread safe)
    fn prepare_component_sync(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Process a component after their run. (thread safe)
    fn process_component_sync(&mut self, connector: &mut SimConnector, component: &mut T);
}

// pub trait SimComponentFactory<O: Organism> {
//     fn attach_new(&mut self, registry: &mut ComponentRegistry<O>);
// }

pub struct ComponentFactory<'a, O: Organism> {
    /// Container for the factory function
    attach_fn: Box<dyn FnMut(&mut ComponentRegistry<O>) + 'a + Send>,
}

impl<'a, O: Organism + 'static> ComponentFactory<'a, O> {
    pub fn new<T: SimComponent<O>>(mut factory: impl FnMut() -> T + 'a + Send) -> Self {
        Self {
            // Magic happens here. We get compile-time assurance and usage
            // of the actual ComponentFactory type while also encapsulating
            // the factory for dynamic dispatch
            attach_fn: Box::new(move |registry: &mut ComponentRegistry<O>| {
                registry.add_component(factory()).unwrap();
            }),
        }
    }

    pub fn attach(&mut self, registry: &mut ComponentRegistry<O>) {
        self.attach_fn.as_mut()(registry);
    }
}
