use crate::sim::Organism;

use super::{ComponentRegistry, SimComponent};


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