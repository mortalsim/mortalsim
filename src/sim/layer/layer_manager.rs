
use crate::sim::{Organism, SimConnector};
use crate::sim::layer::SimLayer;
use crate::sim::component::registry::{ComponentWrapper, ComponentRegistry};
use crate::sim::component::{SimComponent, SimComponentProcessor};

use super::core::CoreLayer;
use super::circulation::CirculationLayer;
use super::digestion::DigestionLayer;
use super::nervous::NervousLayer;



pub struct LayerManager<O: Organism> {
    registry: ComponentRegistry<O>,
    core_layer: CoreLayer<O>,
    circulation_layer: CirculationLayer<O>,
    digestion_layer: DigestionLayer<O>,
    nervous_layer: NervousLayer<O>,

}

impl<O: Organism + 'static> LayerManager<O> {
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            core_layer: CoreLayer::new(),
            circulation_layer: CirculationLayer::new(),
            digestion_layer: DigestionLayer::new(),
            nervous_layer: NervousLayer::new(),

        }
    }

    pub fn add_component(&mut self, component: impl SimComponent<O>) -> anyhow::Result<()> {
        let wrapper = Box::new(ComponentWrapper(component));
        

        self.registry.add_component(wrapper)
    }
    
    pub fn remove_component(&mut self, component_id: &'static str) -> anyhow::Result<()> {
        match self.registry.remove_component(component_id) {
            Ok(_) => Ok(()),
            Err(msg) => Err(msg),
        }
    }

    pub fn update(&mut self, connector: &mut SimConnector) {
    
        self.core_layer.pre_exec(connector);
        self.circulation_layer.pre_exec(connector);
        self.digestion_layer.pre_exec(connector);
        self.nervous_layer.pre_exec(connector);
        let mut update_list = Vec::new();

        for component in self.registry.all_components_mut() {
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
    
        self.core_layer.post_exec(connector);
        self.circulation_layer.post_exec(connector);
        self.digestion_layer.post_exec(connector);
        self.nervous_layer.post_exec(connector);
    }
}

pub struct CoreCirculationDigestionLayerManager<O: Organism> {
    registry: ComponentRegistry<O>,
    core_layer: CoreLayer<O>,
    circulation_layer: CirculationLayer<O>,
    digestion_layer: DigestionLayer<O>,

}

impl<O: Organism + 'static> CoreCirculationDigestionLayerManager<O> {
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            core_layer: CoreLayer::new(),
            circulation_layer: CirculationLayer::new(),
            digestion_layer: DigestionLayer::new(),

        }
    }

    pub fn add_component(&mut self, component: impl SimComponent<O>) -> anyhow::Result<()> {
        let wrapper = Box::new(ComponentWrapper(component));
        
        if wrapper.is_nervous_component() {
            return Err(anyhow!("component types [nervous] are not supported"));
        }

        self.registry.add_component(wrapper)
    }
    
    pub fn remove_component(&mut self, component_id: &'static str) -> anyhow::Result<()> {
        match self.registry.remove_component(component_id) {
            Ok(_) => Ok(()),
            Err(msg) => Err(msg),
        }
    }

    pub fn update(&mut self, connector: &mut SimConnector) {
    
        self.core_layer.pre_exec(connector);
        self.circulation_layer.pre_exec(connector);
        self.digestion_layer.pre_exec(connector);
        let mut update_list = Vec::new();

        for component in self.registry.all_components_mut() {
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
    
        self.core_layer.post_exec(connector);
        self.circulation_layer.post_exec(connector);
        self.digestion_layer.post_exec(connector);
    }
}

pub struct CoreCirculationNervousLayerManager<O: Organism> {
    registry: ComponentRegistry<O>,
    core_layer: CoreLayer<O>,
    circulation_layer: CirculationLayer<O>,
    nervous_layer: NervousLayer<O>,

}

impl<O: Organism + 'static> CoreCirculationNervousLayerManager<O> {
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            core_layer: CoreLayer::new(),
            circulation_layer: CirculationLayer::new(),
            nervous_layer: NervousLayer::new(),

        }
    }

    pub fn add_component(&mut self, component: impl SimComponent<O>) -> anyhow::Result<()> {
        let wrapper = Box::new(ComponentWrapper(component));
        
        if wrapper.is_digestion_component() {
            return Err(anyhow!("component types [digestion] are not supported"));
        }

        self.registry.add_component(wrapper)
    }
    
    pub fn remove_component(&mut self, component_id: &'static str) -> anyhow::Result<()> {
        match self.registry.remove_component(component_id) {
            Ok(_) => Ok(()),
            Err(msg) => Err(msg),
        }
    }

    pub fn update(&mut self, connector: &mut SimConnector) {
    
        self.core_layer.pre_exec(connector);
        self.circulation_layer.pre_exec(connector);
        self.nervous_layer.pre_exec(connector);
        let mut update_list = Vec::new();

        for component in self.registry.all_components_mut() {
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
    
        self.core_layer.post_exec(connector);
        self.circulation_layer.post_exec(connector);
        self.nervous_layer.post_exec(connector);
    }
}

pub struct CoreCirculationLayerManager<O: Organism> {
    registry: ComponentRegistry<O>,
    core_layer: CoreLayer<O>,
    circulation_layer: CirculationLayer<O>,

}

impl<O: Organism + 'static> CoreCirculationLayerManager<O> {
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            core_layer: CoreLayer::new(),
            circulation_layer: CirculationLayer::new(),

        }
    }

    pub fn add_component(&mut self, component: impl SimComponent<O>) -> anyhow::Result<()> {
        let wrapper = Box::new(ComponentWrapper(component));
        
        if wrapper.is_digestion_component() || wrapper.is_nervous_component() {
            return Err(anyhow!("component types [digestion,nervous] are not supported"));
        }

        self.registry.add_component(wrapper)
    }
    
    pub fn remove_component(&mut self, component_id: &'static str) -> anyhow::Result<()> {
        match self.registry.remove_component(component_id) {
            Ok(_) => Ok(()),
            Err(msg) => Err(msg),
        }
    }

    pub fn update(&mut self, connector: &mut SimConnector) {
    
        self.core_layer.pre_exec(connector);
        self.circulation_layer.pre_exec(connector);
        let mut update_list = Vec::new();

        for component in self.registry.all_components_mut() {
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
    
        self.core_layer.post_exec(connector);
        self.circulation_layer.post_exec(connector);
    }
}

pub struct CoreDigestionNervousLayerManager<O: Organism> {
    registry: ComponentRegistry<O>,
    core_layer: CoreLayer<O>,
    digestion_layer: DigestionLayer<O>,
    nervous_layer: NervousLayer<O>,

}

impl<O: Organism + 'static> CoreDigestionNervousLayerManager<O> {
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            core_layer: CoreLayer::new(),
            digestion_layer: DigestionLayer::new(),
            nervous_layer: NervousLayer::new(),

        }
    }

    pub fn add_component(&mut self, component: impl SimComponent<O>) -> anyhow::Result<()> {
        let wrapper = Box::new(ComponentWrapper(component));
        
        if wrapper.is_circulation_component() {
            return Err(anyhow!("component types [circulation] are not supported"));
        }

        self.registry.add_component(wrapper)
    }
    
    pub fn remove_component(&mut self, component_id: &'static str) -> anyhow::Result<()> {
        match self.registry.remove_component(component_id) {
            Ok(_) => Ok(()),
            Err(msg) => Err(msg),
        }
    }

    pub fn update(&mut self, connector: &mut SimConnector) {
    
        self.core_layer.pre_exec(connector);
        self.digestion_layer.pre_exec(connector);
        self.nervous_layer.pre_exec(connector);
        let mut update_list = Vec::new();

        for component in self.registry.all_components_mut() {
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
    
        self.core_layer.post_exec(connector);
        self.digestion_layer.post_exec(connector);
        self.nervous_layer.post_exec(connector);
    }
}

pub struct CoreDigestionLayerManager<O: Organism> {
    registry: ComponentRegistry<O>,
    core_layer: CoreLayer<O>,
    digestion_layer: DigestionLayer<O>,

}

impl<O: Organism + 'static> CoreDigestionLayerManager<O> {
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            core_layer: CoreLayer::new(),
            digestion_layer: DigestionLayer::new(),

        }
    }

    pub fn add_component(&mut self, component: impl SimComponent<O>) -> anyhow::Result<()> {
        let wrapper = Box::new(ComponentWrapper(component));
        
        if wrapper.is_circulation_component() || wrapper.is_nervous_component() {
            return Err(anyhow!("component types [circulation,nervous] are not supported"));
        }

        self.registry.add_component(wrapper)
    }
    
    pub fn remove_component(&mut self, component_id: &'static str) -> anyhow::Result<()> {
        match self.registry.remove_component(component_id) {
            Ok(_) => Ok(()),
            Err(msg) => Err(msg),
        }
    }

    pub fn update(&mut self, connector: &mut SimConnector) {
    
        self.core_layer.pre_exec(connector);
        self.digestion_layer.pre_exec(connector);
        let mut update_list = Vec::new();

        for component in self.registry.all_components_mut() {
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
    
        self.core_layer.post_exec(connector);
        self.digestion_layer.post_exec(connector);
    }
}

pub struct CoreNervousLayerManager<O: Organism> {
    registry: ComponentRegistry<O>,
    core_layer: CoreLayer<O>,
    nervous_layer: NervousLayer<O>,

}

impl<O: Organism + 'static> CoreNervousLayerManager<O> {
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            core_layer: CoreLayer::new(),
            nervous_layer: NervousLayer::new(),

        }
    }

    pub fn add_component(&mut self, component: impl SimComponent<O>) -> anyhow::Result<()> {
        let wrapper = Box::new(ComponentWrapper(component));
        
        if wrapper.is_circulation_component() || wrapper.is_digestion_component() {
            return Err(anyhow!("component types [circulation,digestion] are not supported"));
        }

        self.registry.add_component(wrapper)
    }
    
    pub fn remove_component(&mut self, component_id: &'static str) -> anyhow::Result<()> {
        match self.registry.remove_component(component_id) {
            Ok(_) => Ok(()),
            Err(msg) => Err(msg),
        }
    }

    pub fn update(&mut self, connector: &mut SimConnector) {
    
        self.core_layer.pre_exec(connector);
        self.nervous_layer.pre_exec(connector);
        let mut update_list = Vec::new();

        for component in self.registry.all_components_mut() {
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
    
        self.core_layer.post_exec(connector);
        self.nervous_layer.post_exec(connector);
    }
}

pub struct CoreLayerManager<O: Organism> {
    registry: ComponentRegistry<O>,
    core_layer: CoreLayer<O>,

}

impl<O: Organism + 'static> CoreLayerManager<O> {
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            core_layer: CoreLayer::new(),

        }
    }

    pub fn add_component(&mut self, component: impl SimComponent<O>) -> anyhow::Result<()> {
        let wrapper = Box::new(ComponentWrapper(component));
        
        if wrapper.is_circulation_component() || wrapper.is_digestion_component() || wrapper.is_nervous_component() {
            return Err(anyhow!("component types [circulation,digestion,nervous] are not supported"));
        }

        self.registry.add_component(wrapper)
    }
    
    pub fn remove_component(&mut self, component_id: &'static str) -> anyhow::Result<()> {
        match self.registry.remove_component(component_id) {
            Ok(_) => Ok(()),
            Err(msg) => Err(msg),
        }
    }

    pub fn update(&mut self, connector: &mut SimConnector) {
    
        self.core_layer.pre_exec(connector);
        let mut update_list = Vec::new();

        for component in self.registry.all_components_mut() {
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
    
        self.core_layer.post_exec(connector);
    }
}

