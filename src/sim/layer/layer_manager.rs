use crate::sim::{Organism, SimConnector, SimTime};
use crate::sim::component::registry::ComponentRegistry;
use crate::sim::component::SimComponentProcessor;

use super::circulation::CirculationLayer;
use super::core::CoreLayer;
use super::digestion::digestion_layer::DigestionLayer;
use super::nervous::nervous_layer::NervousLayer;

pub struct LayerManager<O: Organism> {
    sim_connector: SimConnector,
    component_registry: ComponentRegistry<O>,
    core_layer: CoreLayer<O>,
    circulation_layer: CirculationLayer<O>,
    digestion_layer: DigestionLayer<O>,
    nervous_layer: NervousLayer<O>,
}

impl<O: Organism + 'static> LayerManager<O> {
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

    fn sim_time(&self) -> SimTime {
        self.sim_connector.time_manager.get_time()
    }

    fn update(&mut self) {
        self.core_layer.update(&mut self.sim_connector);
        self.circulation_layer.update(&mut self.sim_connector);
        self.digestion_layer.update(&mut self.sim_connector);
        self.nervous_layer.update(&mut self.sim_connector);

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
                self.core_layer.prepare_component(&mut self.sim_connector, component);
            }
            if component.is_circulation_component() {
                self.circulation_layer.prepare_component(&mut self.sim_connector, component);
            }
            if component.is_digestion_component() {
                self.digestion_layer.prepare_component(&mut self.sim_connector, component);
            }
            if component.is_nervous_component() {
                self.nervous_layer.prepare_component(&mut self.sim_connector, component);
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
        self.sim_connector.time_manager.advance();
        self.update();
    }

    pub fn advance_by(&mut self, time_step: SimTime) {
        self.sim_connector.time_manager.advance_by(time_step);
        self.update();
    }

}