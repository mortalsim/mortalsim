use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::sync::Mutex;
use std::thread::{scope, Scope};

use strum::VariantArray;

use crate::sim::component::registry::{ComponentRegistry, ComponentWrapper};
use crate::sim::component::{SimComponent, SimComponentProcessor};
use crate::sim::layer::SimLayer;
use crate::sim::{Organism, SimConnector};

use super::layer_processor::LayerProcessor;
use super::LayerType;
use super::LayerType::*;

pub struct LayerManager<O: Organism> {
    registry: ComponentRegistry<O>,
    layers: Vec<LayerProcessor<O>>,
    layers_sync: Vec<Mutex<LayerProcessor<O>>>,
    missing_layers: Vec<&'static LayerType>,
}

impl<O: Organism + 'static> LayerManager<O> {
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            layers: vec![
                LayerProcessor::new(Core),
                LayerProcessor::new(Circulation),
                LayerProcessor::new(Digestion),
                LayerProcessor::new(Nervous),
            ],
            layers_sync: Vec::new(),
            missing_layers: Vec::new(),
        }
    }
    
    pub fn new_threaded() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            layers: Vec::new(),
            layers_sync: vec![
                Mutex::new(LayerProcessor::new(Core)),
                Mutex::new(LayerProcessor::new(Circulation)),
                Mutex::new(LayerProcessor::new(Digestion)),
                Mutex::new(LayerProcessor::new(Nervous)),
            ],
            missing_layers: Vec::new(),
        }
    }

    pub fn is_threaded(&self) -> bool {
        self.layers.is_empty()
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
            layers_sync: Vec::new(),
            missing_layers: LayerType::VARIANTS
                .into_iter()
                .filter(|lt| !(matches!(lt, Core) || layer_types.contains(lt)))
                .collect(),
        }
    }

    pub fn new_custom_threaded(mut layer_types: HashSet<LayerType>) -> Self {
        // always include Core
        let mut layers = vec![Mutex::new(LayerProcessor::new(Core))];

        // Make sure we don't add Core twice
        layer_types.remove(&Core);

        layers.extend(layer_types.iter().map(|lt| Mutex::new(LayerProcessor::new(*lt))));

        Self {
            registry: ComponentRegistry::new(),
            layers: Vec::new(),
            layers_sync: layers,
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
                return Err(anyhow!(
                    "Layer types [{:?}] are not supported for this Sim!",
                    self.missing_layers
                ));
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

    pub fn components(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.registry.all_components().map(|c| c.id())
    }

    pub fn has_component(&self, component_id: &str) -> bool {
        self.registry.has_component(component_id)
    }

    fn update_sequential(&mut self, connector: &mut SimConnector) {
        for layer in self.layers.iter_mut() {
            layer.pre_exec(connector);
        }

        let mut update_list = Vec::new();

        for component in self.registry.all_components_mut() {
            let mut check_list = self
                .layers
                .iter_mut()
                .filter(|l| component.has_layer(&l.layer_type()));

            // If any of the supported layers indicate the component should be
            // triggered, add the component to the update list
            if check_list.any(|l| l.check_component(component)) {
                update_list.push(component);
            }
        }

        for component in update_list {
            // Prepare the component with each of the associated layers
            // have to collect here to avoid conflicting borrows of component
            let mut layer_list: Vec<&mut LayerProcessor<O>> = self
                .layers
                .iter_mut()
                .filter(|l| component.has_layer(&l.layer_type()))
                .collect();

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

    fn update_threaded(&mut self, connector: &mut SimConnector) {
        for layer in self.layers_sync.iter_mut() {
            layer.lock().unwrap().pre_exec_sync(connector);
        }

        let mut update_list = Vec::new();

        for component in self.registry.all_components_mut() {
            let mut check_list = self
                .layers_sync
                .iter_mut()
                .filter(|l| component.has_layer(&l.lock().unwrap().layer_type()));

            // If any of the supported layers indicate the component should be
            // triggered, add the component to the update list
            if check_list.any(|l| l.lock().unwrap().check_component_sync(component)) {
                update_list.push(component);
            }
        }

        let layers = &self.layers_sync;
        let mconnector = Mutex::new(connector);

        scope(|s| {
            for component in update_list {
                s.spawn(|| {
                    // Prepare the component with each of the associated layers
                    // have to collect here to avoid conflicting borrows of component
                    let mut layer_list: Vec<&Mutex<LayerProcessor<O>>> = layers
                        .iter()
                        .filter(|l| component.has_layer(&l.lock().unwrap().layer_type()))
                        .collect();

                    for layer in layer_list.iter_mut() {
                        layer.lock().unwrap().prepare_component_sync(mconnector.lock().unwrap().borrow_mut(), component);
                    }

                    // Execute component logic
                    component.run();

                    // Execute post run processing
                    for layer in layer_list.iter_mut() {
                        layer.lock().unwrap().process_component_sync(mconnector.lock().unwrap().borrow_mut(), component);
                    }
                });
            }
        });

        let reclaimed_connector = mconnector.into_inner().unwrap();
        for layer in self.layers_sync.iter_mut() {
            layer.lock().unwrap().post_exec_sync(reclaimed_connector);
        }
    }

    pub fn update(&mut self, connector: &mut SimConnector) {
        if self.is_threaded() {
            self.update_threaded(connector)
        }
        else {
            self.update_sequential(connector)
        }
    }
}
