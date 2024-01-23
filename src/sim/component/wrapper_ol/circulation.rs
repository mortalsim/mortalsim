use core::panic;
use std::marker::PhantomData;

use super::super::{ComponentRegistry, SimComponent};
use super::ComponentWrapper;
use crate::sim::organism::Organism;
use crate::sim::layer::circulation::{CirculationComponent, CirculationInitializer, CirculationConnector};

use super::empty_wrapper::{empty_core_wrapper, empty_digestion_wrapper, empty_nervous_wrapper};

pub struct CirculationComponentWrapper<O: Organism + 'static, T: CirculationComponent<O> + 'static>(pub T, pub PhantomData<O>);

empty_core_wrapper!(CirculationComponentWrapper<O, T>, CirculationComponent<O>);
empty_digestion_wrapper!(CirculationComponentWrapper<O, T>, CirculationComponent<O>);
empty_nervous_wrapper!(CirculationComponentWrapper<O, T>, CirculationComponent<O>);

impl<O: Organism, T: CirculationComponent<O>> SimComponent<O> for CirculationComponentWrapper<O, T> {
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

impl<O: Organism, T: CirculationComponent<O>> CirculationComponent<O> for CirculationComponentWrapper<O, T> {
    fn cc_init(&mut self, initializer: &mut CirculationInitializer<O>) {
        self.0.cc_init(initializer)
    }
    fn cc_connector(&mut self) -> &mut CirculationConnector<O> {
        self.0.cc_connector()
    }
}

impl<O: Organism + 'static, T: CirculationComponent<O>> ComponentWrapper<O> for CirculationComponentWrapper<O, T> {}
