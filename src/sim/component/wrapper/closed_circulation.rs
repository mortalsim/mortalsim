use core::panic;
use std::marker::PhantomData;

use super::super::{ComponentRegistry, SimComponent};
use crate::sim::layer::core::component::{CoreComponent, CoreComponentInitializer, CoreConnector};
use crate::sim::organism::Organism;
use crate::sim::layer::closed_circulation::{ClosedCircComponent, ClosedCircInitializer, ClosedCircConnector};

pub struct ClosedCircComponentWrapper<O: Organism + 'static, T: ClosedCircComponent<O> + 'static>(pub T, pub PhantomData<O>);

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

impl<O: Organism, T: ClosedCircComponent<O>> CoreComponent<O> for ClosedCircComponentWrapper<O, T> {
    fn core_init(&mut self, _initializer: &mut CoreComponentInitializer) {
        panic!()
    }
    fn core_connector(&mut self) -> &mut CoreConnector {
        panic!()
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
