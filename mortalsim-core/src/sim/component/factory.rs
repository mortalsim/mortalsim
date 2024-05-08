use crate::sim::Organism;

use super::{registry::ComponentWrapper, ComponentRegistry, SimComponent};


pub struct ComponentFactory<'a, O: Organism> {
    /// Container for the factory function
    attach_fn: Box<dyn (FnMut(&mut ComponentRegistry<O>) -> &'_ mut Box<dyn ComponentWrapper<O>>) + 'a + Send>,
}

impl<'a, O: Organism> ComponentFactory<'a, O> {
    pub fn new<T: SimComponent<O>>(mut factory: impl (FnMut() -> T) + 'a + Send) -> Self {
        Self {
            // Magic happens here. We get compile-time assurance and usage
            // of the actual ComponentFactory type while also encapsulating
            // the factory for dynamic dispatch
            attach_fn: Box::new(move |registry: &mut ComponentRegistry<O>| {
                let comp = factory();
                log::trace!("adding component: {}", comp.id());
                registry.add_component(comp).unwrap()
            }),
        }
    }

    pub fn attach<'b>(&mut self, registry: &'b mut ComponentRegistry<O>) -> &'b mut Box<dyn ComponentWrapper<O>> {
        self.attach_fn.as_mut()(registry)
    }
}