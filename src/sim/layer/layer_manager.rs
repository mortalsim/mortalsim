
use std::collections::HashSet;

use strum::VariantArray;

use crate::sim::{Organism, SimConnector};
use crate::sim::layer::SimLayer;
use crate::sim::component::registry::{ComponentWrapper, ComponentRegistry};
use crate::sim::component::{SimComponent, SimComponentProcessor};

use super::layer_processor::LayerProcessor;
use super::LayerType;
use super::LayerType::*;

pub struct LayerManager<O: Organism + ?Sized> {
    registry: ComponentRegistry<O>,
    layers: Vec<LayerProcessor<O>>,
    missing_layers: Vec<&'static LayerType>,
}

impl<O: Organism + ?Sized + 'static> LayerManager<O> {
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            layers: vec![
                LayerProcessor::new(Core),
                LayerProcessor::new(Circulation),
                LayerProcessor::new(Digestion),
                LayerProcessor::new(Nervous),
            ],
            missing_layers: Vec::new(),
        }
    }

    pub fn new_custom(mut layer_types: HashSet<LayerType>) -> Self {
        // always include Core
        let mut layers = vec![LayerProcessor::new(Core)];

        // Make sure we don't add Core twice
        layer_types.remove(&Core);

        layers.extend(layer_types.iter().map(|lt| LayerProcessor::new(*lt)));

        Self {
            registry: ComponentRegistry::new(),
            layers,
            missing_layers: LayerType::VARIANTS
                .into_iter()
                .filter(|lt| !(matches!(lt, Core) || layer_types.contains(lt)))
                .collect(),
        }
    }

    pub fn add_component(&mut self, component: impl SimComponent<O>) -> anyhow::Result<()> {
        let wrapper = self.registry.add_component(component)?;
        if !self.missing_layers.is_empty() {
            if self.missing_layers.iter().any(|lt| wrapper.has_layer(lt)) {
                return Err(anyhow!("Layer types [{:?}] are not supported for this Sim!", self.missing_layers));
            }
        }
        Ok(())
    }

    pub fn attach_component(&mut self, attach_fn: impl FnOnce(&mut ComponentRegistry<O>)) {
        attach_fn(&mut self.registry)
    }
    
    pub fn remove_component(&mut self, component_id: &str) -> anyhow::Result<&'static str> {
        match self.registry.remove_component(component_id) {
            Ok(c) => Ok(c.id()),
            Err(msg) => Err(msg),
        }
    }

    pub fn components(&self) -> impl Iterator<Item=&'static str> + '_ {
        self.registry.all_components().map(|c| c.id())
    }

    pub fn has_component(&self, component_id: &str) -> bool {
        self.registry.has_component(component_id)
    }

    pub fn update(&mut self, connector: &mut SimConnector) {

        for layer in self.layers.iter_mut() {
            layer.pre_exec(connector);
        }
    
        let mut update_list = Vec::new();

        for component in self.registry.all_components_mut() {
            let mut check_list = self.layers.iter_mut().filter(|l| component.has_layer(&l.layer_type()));

            // If any of the supported layers indicate the component should be
            // triggered, add the component to the update list
            if check_list.any(|l| l.check_component(component)) {
                update_list.push(component);
            }
        }

        for component in update_list {
            // Prepare the component with each of the associated layers
            // have to collect here to avoid conflicting borrows of component
            let mut layer_list: Vec<&mut LayerProcessor<O>> = 
                self.layers.iter_mut().filter(|l| component.has_layer(&l.layer_type())).collect();
    
            for layer in layer_list.iter_mut() {
                layer.prepare_component(connector, component);
            }

            // Execute component logic
            component.run();
            
            // Execute post run processing
            for layer in layer_list.iter_mut() {
                layer.process_component(connector, component);
            }
        }
    
        for layer in self.layers.iter_mut() {
            layer.post_exec(connector);
        }
    }
}
