use crate::sim::{layer::core::component::CoreComponent, organism::human::component::HumanComponent};
use crate::sim::layer::closed_circulation::ClosedCircComponent;
use std::collections::HashMap;

use super::{
    wrapper::{core::CoreComponentWrapper, closed_circulation::ClosedCircComponentWrapper, human::HumanComponentWrapper, ComponentWrapper},
    SimComponent,
};

pub struct ComponentRegistry<'a>(HashMap<&'a str, Box<dyn ComponentWrapper>>);

impl<'a> ComponentRegistry<'a> {
    // TODO: create add_{system}Component methods for each combination of systems
    pub fn add_core_component(&mut self, component: impl CoreComponent + 'static) {
        self.0
            .insert(component.id(), Box::new(CoreComponentWrapper(component)));
    }
    pub fn add_closed_circulation_component(&mut self, component: impl ClosedCircComponent + 'static) {
        self.0
            .insert(component.id(), Box::new(ClosedCircComponentWrapper(component)));
    }
    pub fn add_human_component(&mut self, component: impl HumanComponent + 'static) {
        self.0
            .insert(component.id(), Box::new(HumanComponentWrapper(component)));
    }
}
