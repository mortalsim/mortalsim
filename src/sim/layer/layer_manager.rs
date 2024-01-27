
use crate::sim::{Organism, SimConnector};
use crate::sim::component::registry::ComponentRegistry;
use crate::sim::component::SimComponentProcessor;

use super::core::CoreLayer;
use super::circulation::CirculationLayer;
use super::digestion::DigestionLayer;
use super::nervous::NervousLayer;



pub struct LayerManager<O: Organism> {
    core_layer: CoreLayer<O>,
    circulation_layer: CirculationLayer<O>,
    digestion_layer: DigestionLayer<O>,
    nervous_layer: NervousLayer<O>,
}

impl<O: Organism + 'static> LayerManager<O> {
    pub fn new() -> Self {
        Self {
            core_layer: CoreLayer::new(),
            circulation_layer: CirculationLayer::new(),
            digestion_layer: DigestionLayer::new(),
            nervous_layer: NervousLayer::new(),
        }
    }

    pub fn update(&mut self, component_registry: &mut ComponentRegistry<O>, connector: &mut SimConnector) {
    
        self.core_layer.update(connector);
        self.circulation_layer.update(connector);
        self.digestion_layer.update(connector);
        self.nervous_layer.update(connector);
        let mut update_list = Vec::new();

        for component in component_registry.all_components_mut() {
            if 
                ( component.is_core_component() &&
                    self.core_layer.check_component(component) ) ||
                ( component.is_circulation_component() &&
                    self.circulation_layer.check_component(component) ) ||
                ( component.is_digestion_component() &&
                    self.digestion_layer.check_component(component) ) ||
                ( component.is_nervous_component() &&
                    self.nervous_layer.check_component(component) ) {
                update_list.push(component);
            }
        }

        for component in update_list {
            // Prepare the component with each of the associated layers
    
            if component.is_core_component() {
                self.core_layer.prepare_component(connector, component);
            }
            if component.is_circulation_component() {
                self.circulation_layer.prepare_component(connector, component);
            }
            if component.is_digestion_component() {
                self.digestion_layer.prepare_component(connector, component);
            }
            if component.is_nervous_component() {
                self.nervous_layer.prepare_component(connector, component);
            }

            // Execute component logic
            component.run();

    
            if component.is_core_component() {
                self.core_layer.process_component(connector, component);
            }
            if component.is_circulation_component() {
                self.circulation_layer.process_component(connector, component);
            }
            if component.is_digestion_component() {
                self.digestion_layer.process_component(connector, component);
            }
            if component.is_nervous_component() {
                self.nervous_layer.process_component(connector, component);
            }
        }
    }
}

pub struct CoreCirculationDigestionLayerManager<O: Organism> {
    core_layer: CoreLayer<O>,
    circulation_layer: CirculationLayer<O>,
    digestion_layer: DigestionLayer<O>,
}

impl<O: Organism + 'static> CoreCirculationDigestionLayerManager<O> {
    pub fn new() -> Self {
        Self {
            core_layer: CoreLayer::new(),
            circulation_layer: CirculationLayer::new(),
            digestion_layer: DigestionLayer::new(),
        }
    }

    pub fn update(&mut self, component_registry: &mut ComponentRegistry<O>, connector: &mut SimConnector) {
    
        self.core_layer.update(connector);
        self.circulation_layer.update(connector);
        self.digestion_layer.update(connector);
        let mut update_list = Vec::new();

        for component in component_registry.all_components_mut() {
            if 
                ( component.is_core_component() &&
                    self.core_layer.check_component(component) ) ||
                ( component.is_circulation_component() &&
                    self.circulation_layer.check_component(component) ) ||
                ( component.is_digestion_component() &&
                    self.digestion_layer.check_component(component) ) {
                update_list.push(component);
            }
        }

        for component in update_list {
            // Prepare the component with each of the associated layers
    
            if component.is_core_component() {
                self.core_layer.prepare_component(connector, component);
            }
            if component.is_circulation_component() {
                self.circulation_layer.prepare_component(connector, component);
            }
            if component.is_digestion_component() {
                self.digestion_layer.prepare_component(connector, component);
            }

            // Execute component logic
            component.run();

    
            if component.is_core_component() {
                self.core_layer.process_component(connector, component);
            }
            if component.is_circulation_component() {
                self.circulation_layer.process_component(connector, component);
            }
            if component.is_digestion_component() {
                self.digestion_layer.process_component(connector, component);
            }
        }
    }
}

pub struct CoreCirculationNervousLayerManager<O: Organism> {
    core_layer: CoreLayer<O>,
    circulation_layer: CirculationLayer<O>,
    nervous_layer: NervousLayer<O>,
}

impl<O: Organism + 'static> CoreCirculationNervousLayerManager<O> {
    pub fn new() -> Self {
        Self {
            core_layer: CoreLayer::new(),
            circulation_layer: CirculationLayer::new(),
            nervous_layer: NervousLayer::new(),
        }
    }

    pub fn update(&mut self, component_registry: &mut ComponentRegistry<O>, connector: &mut SimConnector) {
    
        self.core_layer.update(connector);
        self.circulation_layer.update(connector);
        self.nervous_layer.update(connector);
        let mut update_list = Vec::new();

        for component in component_registry.all_components_mut() {
            if 
                ( component.is_core_component() &&
                    self.core_layer.check_component(component) ) ||
                ( component.is_circulation_component() &&
                    self.circulation_layer.check_component(component) ) ||
                ( component.is_nervous_component() &&
                    self.nervous_layer.check_component(component) ) {
                update_list.push(component);
            }
        }

        for component in update_list {
            // Prepare the component with each of the associated layers
    
            if component.is_core_component() {
                self.core_layer.prepare_component(connector, component);
            }
            if component.is_circulation_component() {
                self.circulation_layer.prepare_component(connector, component);
            }
            if component.is_nervous_component() {
                self.nervous_layer.prepare_component(connector, component);
            }

            // Execute component logic
            component.run();

    
            if component.is_core_component() {
                self.core_layer.process_component(connector, component);
            }
            if component.is_circulation_component() {
                self.circulation_layer.process_component(connector, component);
            }
            if component.is_nervous_component() {
                self.nervous_layer.process_component(connector, component);
            }
        }
    }
}

pub struct CoreCirculationLayerManager<O: Organism> {
    core_layer: CoreLayer<O>,
    circulation_layer: CirculationLayer<O>,
}

impl<O: Organism + 'static> CoreCirculationLayerManager<O> {
    pub fn new() -> Self {
        Self {
            core_layer: CoreLayer::new(),
            circulation_layer: CirculationLayer::new(),
        }
    }

    pub fn update(&mut self, component_registry: &mut ComponentRegistry<O>, connector: &mut SimConnector) {
    
        self.core_layer.update(connector);
        self.circulation_layer.update(connector);
        let mut update_list = Vec::new();

        for component in component_registry.all_components_mut() {
            if 
                ( component.is_core_component() &&
                    self.core_layer.check_component(component) ) ||
                ( component.is_circulation_component() &&
                    self.circulation_layer.check_component(component) ) {
                update_list.push(component);
            }
        }

        for component in update_list {
            // Prepare the component with each of the associated layers
    
            if component.is_core_component() {
                self.core_layer.prepare_component(connector, component);
            }
            if component.is_circulation_component() {
                self.circulation_layer.prepare_component(connector, component);
            }

            // Execute component logic
            component.run();

    
            if component.is_core_component() {
                self.core_layer.process_component(connector, component);
            }
            if component.is_circulation_component() {
                self.circulation_layer.process_component(connector, component);
            }
        }
    }
}

pub struct CoreDigestionNervousLayerManager<O: Organism> {
    core_layer: CoreLayer<O>,
    digestion_layer: DigestionLayer<O>,
    nervous_layer: NervousLayer<O>,
}

impl<O: Organism + 'static> CoreDigestionNervousLayerManager<O> {
    pub fn new() -> Self {
        Self {
            core_layer: CoreLayer::new(),
            digestion_layer: DigestionLayer::new(),
            nervous_layer: NervousLayer::new(),
        }
    }

    pub fn update(&mut self, component_registry: &mut ComponentRegistry<O>, connector: &mut SimConnector) {
    
        self.core_layer.update(connector);
        self.digestion_layer.update(connector);
        self.nervous_layer.update(connector);
        let mut update_list = Vec::new();

        for component in component_registry.all_components_mut() {
            if 
                ( component.is_core_component() &&
                    self.core_layer.check_component(component) ) ||
                ( component.is_digestion_component() &&
                    self.digestion_layer.check_component(component) ) ||
                ( component.is_nervous_component() &&
                    self.nervous_layer.check_component(component) ) {
                update_list.push(component);
            }
        }

        for component in update_list {
            // Prepare the component with each of the associated layers
    
            if component.is_core_component() {
                self.core_layer.prepare_component(connector, component);
            }
            if component.is_digestion_component() {
                self.digestion_layer.prepare_component(connector, component);
            }
            if component.is_nervous_component() {
                self.nervous_layer.prepare_component(connector, component);
            }

            // Execute component logic
            component.run();

    
            if component.is_core_component() {
                self.core_layer.process_component(connector, component);
            }
            if component.is_digestion_component() {
                self.digestion_layer.process_component(connector, component);
            }
            if component.is_nervous_component() {
                self.nervous_layer.process_component(connector, component);
            }
        }
    }
}

pub struct CoreDigestionLayerManager<O: Organism> {
    core_layer: CoreLayer<O>,
    digestion_layer: DigestionLayer<O>,
}

impl<O: Organism + 'static> CoreDigestionLayerManager<O> {
    pub fn new() -> Self {
        Self {
            core_layer: CoreLayer::new(),
            digestion_layer: DigestionLayer::new(),
        }
    }

    pub fn update(&mut self, component_registry: &mut ComponentRegistry<O>, connector: &mut SimConnector) {
    
        self.core_layer.update(connector);
        self.digestion_layer.update(connector);
        let mut update_list = Vec::new();

        for component in component_registry.all_components_mut() {
            if 
                ( component.is_core_component() &&
                    self.core_layer.check_component(component) ) ||
                ( component.is_digestion_component() &&
                    self.digestion_layer.check_component(component) ) {
                update_list.push(component);
            }
        }

        for component in update_list {
            // Prepare the component with each of the associated layers
    
            if component.is_core_component() {
                self.core_layer.prepare_component(connector, component);
            }
            if component.is_digestion_component() {
                self.digestion_layer.prepare_component(connector, component);
            }

            // Execute component logic
            component.run();

    
            if component.is_core_component() {
                self.core_layer.process_component(connector, component);
            }
            if component.is_digestion_component() {
                self.digestion_layer.process_component(connector, component);
            }
        }
    }
}

pub struct CoreNervousLayerManager<O: Organism> {
    core_layer: CoreLayer<O>,
    nervous_layer: NervousLayer<O>,
}

impl<O: Organism + 'static> CoreNervousLayerManager<O> {
    pub fn new() -> Self {
        Self {
            core_layer: CoreLayer::new(),
            nervous_layer: NervousLayer::new(),
        }
    }

    pub fn update(&mut self, component_registry: &mut ComponentRegistry<O>, connector: &mut SimConnector) {
    
        self.core_layer.update(connector);
        self.nervous_layer.update(connector);
        let mut update_list = Vec::new();

        for component in component_registry.all_components_mut() {
            if 
                ( component.is_core_component() &&
                    self.core_layer.check_component(component) ) ||
                ( component.is_nervous_component() &&
                    self.nervous_layer.check_component(component) ) {
                update_list.push(component);
            }
        }

        for component in update_list {
            // Prepare the component with each of the associated layers
    
            if component.is_core_component() {
                self.core_layer.prepare_component(connector, component);
            }
            if component.is_nervous_component() {
                self.nervous_layer.prepare_component(connector, component);
            }

            // Execute component logic
            component.run();

    
            if component.is_core_component() {
                self.core_layer.process_component(connector, component);
            }
            if component.is_nervous_component() {
                self.nervous_layer.process_component(connector, component);
            }
        }
    }
}

pub struct CoreLayerManager<O: Organism> {
    core_layer: CoreLayer<O>,
}

impl<O: Organism + 'static> CoreLayerManager<O> {
    pub fn new() -> Self {
        Self {
            core_layer: CoreLayer::new(),
        }
    }

    pub fn update(&mut self, component_registry: &mut ComponentRegistry<O>, connector: &mut SimConnector) {
    
        self.core_layer.update(connector);
        let mut update_list = Vec::new();

        for component in component_registry.all_components_mut() {
            if 
                 component.is_core_component() &&
                    self.core_layer.check_component(component)  {
                update_list.push(component);
            }
        }

        for component in update_list {
            // Prepare the component with each of the associated layers
    
            if component.is_core_component() {
                self.core_layer.prepare_component(connector, component);
            }

            // Execute component logic
            component.run();

    
            if component.is_core_component() {
                self.core_layer.process_component(connector, component);
            }
        }
    }
}

