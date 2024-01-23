use std::marker::PhantomData;

use crate::sim::layer::core::component::{CoreComponent, CoreComponentInitializer, CoreConnector};
use crate::sim::organism::Organism;
use crate::sim::component::{registry::ComponentRegistry, SimComponent};

use super::empty_wrapper::{empty_cc_wrapper, empty_digestion_wrapper, empty_nervous_wrapper};
use super::ComponentWrapper;

pub struct CoreComponentWrapper<O: Organism, T: CoreComponent<O> + 'static>(pub T, pub PhantomData<O>);

empty_cc_wrapper!(CoreComponentWrapper<O, T>, CoreComponent<O>);
empty_digestion_wrapper!(CoreComponentWrapper<O, T>, CoreComponent<O>);
empty_nervous_wrapper!(CoreComponentWrapper<O, T>, CoreComponent<O>);

impl<O: Organism + 'static, T: CoreComponent<O>> SimComponent<O> for CoreComponentWrapper<O, T> {
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

impl<O: Organism + 'static, T: CoreComponent<O>> CoreComponent<O> for CoreComponentWrapper<O, T> {
    fn core_init(&mut self, initializer: &mut CoreComponentInitializer) {
        self.0.core_init(initializer)
    }
    fn core_connector(&mut self) -> &mut CoreConnector {
        self.0.core_connector()
    }
}

impl<O: Organism + 'static, T: CoreComponent<O>> ComponentWrapper<O> for CoreComponentWrapper<O, T> {}
