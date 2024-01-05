use crate::sim::organism::generic::GenericSim;
use crate::sim::{layer::core::component::CoreComponent, organism::Organism};
use crate::sim::layer::closed_circulation::ClosedCircComponent;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::Component;

use super::{
    wrapper::{core::CoreComponentWrapper, closed_circulation::ClosedCircComponentWrapper, ComponentWrapper},
    SimComponent,
};

pub struct ComponentRegistry<'a, O: Organism>(pub HashMap<&'a str, Box<dyn ComponentWrapper<O>>>);

impl<'a, O: Organism + 'static> ComponentRegistry<'a, O> {
    pub fn add_core_component(&mut self, component: impl CoreComponent<O> + 'static) {
        self.0
            .insert(component.id(), Box::new(CoreComponentWrapper(component, PhantomData)));
    }
    pub fn add_closed_circulation_component(&mut self, component: impl ClosedCircComponent<O> + 'static) {
        self.0
            .insert(component.id(), Box::new(ClosedCircComponentWrapper(component, PhantomData)));
    }
}
