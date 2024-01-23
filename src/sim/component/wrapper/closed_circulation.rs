use core::panic;
use std::marker::PhantomData;

use super::super::{ComponentRegistry, SimComponent};
use super::ComponentWrapper;
use crate::sim::organism::Organism;
use crate::sim::layer::closed_circulation::{ClosedCircComponent, ClosedCircInitializer, ClosedCircConnector};

use super::empty_wrapper::{empty_core_wrapper, empty_digestion_wrapper, empty_nervous_wrapper};

pub struct ClosedCircComponentWrapper<O: Organism + 'static, T: ClosedCircComponent<O> + 'static>(pub T, pub PhantomData<O>);

empty_core_wrapper!(ClosedCircComponentWrapper<O, T>, ClosedCircComponent<O>);
empty_digestion_wrapper!(ClosedCircComponentWrapper<O, T>, ClosedCircComponent<O>);
empty_nervous_wrapper!(ClosedCircComponentWrapper<O, T>, ClosedCircComponent<O>);

impl<O: Organism, T: ClosedCircComponent<O>> SimComponent<O> for ClosedCircComponentWrapper<O, T> {
    fn id(&self) -> &'static str {
        self.0.id()
    }
    fn attach(self, registry: &mut ComponentRegistry<O>) {
        registry.add_closed_circulation_component(self);
    }
    fn run(&mut self) {
        self.0.run();
    }
}

impl<O: Organism, T: ClosedCircComponent<O>> ClosedCircComponent<O> for ClosedCircComponentWrapper<O, T> {
    fn cc_init(&mut self, initializer: &mut ClosedCircInitializer<O>) {
        self.0.cc_init(initializer)
    }
    fn cc_connector(&mut self) -> &mut ClosedCircConnector<O> {
        self.0.cc_connector()
    }
}

impl<O: Organism + 'static, T: ClosedCircComponent<O>> ComponentWrapper<O> for ClosedCircComponentWrapper<O, T> {
    fn attach(self, registry: &mut ComponentRegistry<O>) {
        registry.add_closed_circulation_component(self)
    }
}
