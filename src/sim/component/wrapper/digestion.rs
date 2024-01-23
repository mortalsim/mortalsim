use std::marker::PhantomData;

use crate::sim::layer::{DigestionComponent, DigestionConnector, DigestionInitializer};
use crate::sim::organism::Organism;
use crate::sim::component::{registry::ComponentRegistry, SimComponent};

use super::empty_wrapper::{empty_cc_wrapper, empty_core_wrapper, empty_nervous_wrapper};
use super::ComponentWrapper;

pub struct DigestionComponentWrapper<O: Organism + 'static, T: DigestionComponent<O> + 'static>(pub T, pub PhantomData<O>);

empty_core_wrapper!(DigestionComponentWrapper<O, T>, DigestionComponent<O>);
empty_cc_wrapper!(DigestionComponentWrapper<O, T>, DigestionComponent<O>);
empty_nervous_wrapper!(DigestionComponentWrapper<O, T>, DigestionComponent<O>);

impl<O: Organism + 'static, T: DigestionComponent<O>> SimComponent<O> for DigestionComponentWrapper<O, T> {
    fn id(&self) -> &'static str {
        self.0.id()
    }
    fn attach(self, registry: &mut ComponentRegistry<O>) {
        registry.add_digestion_component(self);
    }
    fn run(&mut self) {
        self.0.run();
    }
}

impl<O: Organism + 'static, T: DigestionComponent<O>> DigestionComponent<O> for DigestionComponentWrapper<O, T> {
    fn digestion_init(&mut self, initializer: &mut DigestionInitializer<O>) {
        self.0.digestion_init(initializer)
    }
    fn digestion_connector(&mut self) -> &mut DigestionConnector<O> {
        self.0.digestion_connector()
    }
}

impl<O: Organism + 'static, T: DigestionComponent<O>> ComponentWrapper<O> for DigestionComponentWrapper<O, T> {
    fn attach(self, registry: &mut ComponentRegistry<O>) {
        registry.add_digestion_component(self)
    }
}
