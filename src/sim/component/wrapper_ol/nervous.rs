
use std::marker::PhantomData;

use crate::sim::layer::{NervousComponent, NervousConnector, NervousInitializer};
use crate::sim::organism::Organism;
use crate::sim::component::{registry::ComponentRegistry, SimComponent};

use super::empty_wrapper::{empty_cc_wrapper, empty_core_wrapper, empty_digestion_wrapper};
use super::ComponentWrapper;

pub struct NervousComponentWrapper<O: Organism + 'static, T: NervousComponent<O> + 'static>(pub T, pub PhantomData<O>);

empty_core_wrapper!(NervousComponentWrapper<O, T>, NervousComponent<O>);
empty_cc_wrapper!(NervousComponentWrapper<O, T>, NervousComponent<O>);
empty_digestion_wrapper!(NervousComponentWrapper<O, T>, NervousComponent<O>);

impl<O: Organism + 'static, T: NervousComponent<O>> SimComponent<O> for NervousComponentWrapper<O, T> {
    fn id(&self) -> &'static str {
        self.0.id()
    }
    fn attach(self, registry: &mut ComponentRegistry<O>) {
        self.0.attach(registry)
    }
    fn run(&mut self) {
        self.0.run();
    }
}

impl<O: Organism + 'static, T: NervousComponent<O>> NervousComponent<O> for NervousComponentWrapper<O, T> {
    fn nervous_init(&mut self, initializer: &mut NervousInitializer<O>) {
        self.0.nervous_init(initializer)
    }
    fn nervous_connector(&mut self) -> &mut NervousConnector<O> {
        self.0.nervous_connector()
    }
}

impl<O: Organism + 'static, T: NervousComponent<O>> ComponentWrapper<O> for NervousComponentWrapper<O, T> {}
