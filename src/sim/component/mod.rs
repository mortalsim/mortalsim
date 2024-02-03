pub mod registry;

use self::registry::ComponentRegistry;
use super::SimConnector;
use super::organism::Organism;

/// Common trait for all simulation components
pub trait SimComponent<O: Organism + ?Sized> {
    /// The unique id of the component
    fn id(&self) -> &'static str;
    /// Attaches the module to the ComponentRegistry
    fn attach(self, registry: &mut ComponentRegistry<O>);
    /// Runs an iteration of this module.
    fn run(&mut self);
}

/// Trait to outline common methods for all layers that
/// process `SimComponent`s
pub trait SimComponentProcessor<O: Organism + ?Sized, T: SimComponent<O>> {
    /// Execute initial setup for a component
    fn setup_component(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Indicate if the given component should trigger a run
    fn check_component(&mut self, component: &T) -> bool;
    /// Prepare a component for their run
    fn prepare_component(&mut self, connector: &mut SimConnector, component: &mut T);
    /// Process a component after their run.
    fn process_component(&mut self, connector: &mut SimConnector, component: &mut T);
}

// pub trait SimComponentFactory<O: Organism + ?Sized> {
//     fn attach_new(&mut self, registry: &mut ComponentRegistry<O>);
// }

pub struct ComponentFactory<'a, O: Organism + ?Sized> {
    /// Container for the factory function
    attach_fn: Box<dyn FnMut(&mut ComponentRegistry<O>) + 'a + Send + Sync>,
}

impl<'a, O: Organism + ?Sized + 'static> ComponentFactory<'a, O> {
    pub fn new<T: SimComponent<O>>(mut factory: impl FnMut() -> T + 'a + Send + Sync) -> Self {
        Self {
            // Magic happens here. We get compile-time assurance and usage
            // of the actual ComponentFactory type while also encapsulating
            // the factory for dynamic dispatch
            attach_fn: Box::new(move |registry: &mut ComponentRegistry<O>| {
                registry.add_component(factory()).unwrap();
            })
        }
    }
    
    pub fn attach(&mut self, registry: &mut ComponentRegistry<O>) {
        self.attach_fn.as_mut()(registry);
    }
}
