use crate::sim::layer::{DigestionComponent, NervousComponent};
use crate::sim::{layer::core::component::CoreComponent, organism::Organism};
use crate::sim::layer::closed_circulation::ClosedCircComponent;
use std::marker::PhantomData;

use super::wrapper::digestion::DigestionComponentWrapper;
use super::wrapper::nervous::NervousComponentWrapper;
use super::SimComponent;
use super::wrapper::closed_circulation::ClosedCircComponentWrapper;
use super::wrapper::core::CoreComponentWrapper;

pub struct ComponentRegistry<O: Organism>(pub Vec<Box<dyn SimComponent<O>>>);

impl<O: Organism + 'static> ComponentRegistry<O> {
    pub fn add_component(&mut self, component: impl SimComponent<O>) {
        component.attach(self);
    }
    pub fn add_core_component(&mut self, component: impl CoreComponent<O> + 'static) {
        self.0
            .push(Box::new(CoreComponentWrapper(component, PhantomData)));
    }
    pub fn add_closed_circulation_component(&mut self, component: impl ClosedCircComponent<O> + 'static) {
        self.0
            .push(Box::new(ClosedCircComponentWrapper(component, PhantomData)));
    }
    pub fn add_digestion_component(&mut self, component: impl DigestionComponent<O> + 'static) {
        self.0
            .push(Box::new(DigestionComponentWrapper(component, PhantomData)));
    }
    pub fn add_nervous_component(&mut self, component: impl NervousComponent<O> + 'static) {
        self.0
            .push(Box::new(NervousComponentWrapper(component, PhantomData)));
    }
}
