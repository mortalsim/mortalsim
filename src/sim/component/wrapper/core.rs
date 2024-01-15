use std::{marker::PhantomData, collections::HashSet};

use crate::sim::layer::core::component::{CoreComponent, CoreComponentInitializer, CoreConnector};
use crate::sim::layer::SimLayer;
use crate::sim::layer::closed_circulation::{ClosedCircComponent, ClosedCircInitializer, ClosedCircConnector, BloodVessel, DummyVessel};
use crate::sim::organism::Organism;
use crate::sim::component::{registry::ComponentRegistry, SimComponent};

use super::ComponentWrapper;

pub struct CoreComponentWrapper<O: Organism, T: CoreComponent<O> + 'static>(pub T, pub PhantomData<O>);

impl<O: Organism + 'static, T: CoreComponent<O>> SimComponent<O> for CoreComponentWrapper<O, T> {
    fn id(&self) -> &'static str {
        self.0.id()
    }
    fn attach(self, registry: &mut ComponentRegistry<O>) {
        registry.add_core_component(self);
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

impl<O: Organism + 'static, T: CoreComponent<O>> ClosedCircComponent<O> for CoreComponentWrapper<O, T> {
    fn cc_init(&mut self, _initializer: &mut ClosedCircInitializer<O>) {
        panic!()
    }
    fn cc_connector(&mut self) -> &mut ClosedCircConnector<O> {
        panic!()
    }
}
