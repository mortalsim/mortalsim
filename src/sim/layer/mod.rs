
use std::collections::hash_set;
use std::fmt::Debug;

pub mod core;
pub mod circulation;
pub mod digestion;
pub mod nervous;

use crate::event::Event;

pub use self::core::component::*;
use self::{circulation::CirculationLayer, core::CoreLayer, digestion::digestion_layer::DigestionLayer, nervous::nervous_layer::NervousLayer};
pub use circulation::component::*;
pub use digestion::component::*;
pub use nervous::component::*;

use super::component::registry::ComponentRegistry;
use super::component::{SimComponent, SimComponentProcessor};
use super::{Organism, SimConnector, SimTime};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum SimLayer {
    Core,
    Circulation,
    Digestion,
    Nervous,
}

#[derive(Debug)]
pub(crate) struct InternalLayerTrigger(SimLayer);

impl InternalLayerTrigger {
    pub fn layer(&self) -> SimLayer {
        self.0
    }
}

impl Event for InternalLayerTrigger {
    fn event_name(&self) -> &str {
        "InternalLayerEvent"
    }
}

pub struct AnatomicalRegionIter<'a, T: Clone>(pub hash_set::Iter<'a, T>);

impl<'a, T: Clone> Iterator for AnatomicalRegionIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        Some(self.0.next()?.clone())
    }
}

pub struct LayerManager<O: Organism + 'static> {
    sim_connector: SimConnector,
    component_registry: ComponentRegistry<O>,
    core_layer: CoreLayer<O>,
    circulation_layer: CirculationLayer<O>,
    digestion_layer: DigestionLayer<O>,
    nervous_layer: NervousLayer<O>,
}

impl<O: Organism> LayerManager<O> {
    pub fn new() -> Self {
        Self {
            sim_connector: SimConnector::new(),
            component_registry: ComponentRegistry::new(),
            core_layer: CoreLayer::new(),
            circulation_layer: CirculationLayer::new(),
            digestion_layer: DigestionLayer::new(),
            nervous_layer: NervousLayer::new(),
        }
    }

    fn update(&mut self) {
        let mut update_list = Vec::new();

        for component in self.component_registry.all_components_mut() {
            if (component.is_core_component() && self.core_layer.check_component(component)) ||
               (component.is_circulation_component() && self.circulation_layer.check_component(component)) ||
               (component.is_digestion_component() && self.digestion_layer.check_component(component)) ||
               (component.is_nervous_component() && self.nervous_layer.check_component(component)) {
                update_list.push(component);
            }
        }

        for component in update_list {
            
            // Prepare the component with each of the associated layers
            if component.is_core_component() {
                self.core_layer.prepare_component(&self.sim_connector, component);
            }
            if component.is_circulation_component() {
                self.circulation_layer.prepare_component(&self.sim_connector, component);
            }
            if component.is_digestion_component() {
                self.digestion_layer.prepare_component(&self.sim_connector, component);
            }
            if component.is_nervous_component() {
                self.nervous_layer.prepare_component(&self.sim_connector, component);
            }

            // Execute component logic
            component.run();

            // Process the component with each of the associated layers
            if component.is_core_component() {
                self.core_layer.process_component(&mut self.sim_connector, component);
            }
            if component.is_circulation_component() {
                self.circulation_layer.process_component(&mut self.sim_connector, component);
            }
            if component.is_digestion_component() {
                self.digestion_layer.process_component(&mut self.sim_connector, component);
            }
            if component.is_nervous_component() {
                self.nervous_layer.process_component(&mut self.sim_connector, component);
            }
        }
    }

    pub fn advance(&mut self) {
        self.sim_connector.time_manager.advance()
    }

    pub fn advance_by(&mut self, time_step: SimTime) {
        self.sim_connector.time_manager.advance_by(time_step)
    }
}
