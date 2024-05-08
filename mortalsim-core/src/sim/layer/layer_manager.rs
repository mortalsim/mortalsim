use std::any::TypeId;
use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::sync::Mutex;
use std::thread::{scope, Scope};

use strum::VariantArray;
use rand::distributions::{Alphanumeric, DistString};

use crate::sim::component::registry::{ComponentRegistry, ComponentWrapper};
use crate::sim::component::{ComponentFactory, SimComponent, SimComponentProcessor, SimComponentProcessorSync};
use crate::sim::layer::SimLayer;
use crate::sim::{Organism, SimConnector};

use super::layer_processor::{LayerProcessor, LayerProcessorSync};
use super::{LayerType, SimLayerSync};
use super::LayerType::*;

pub struct LayerManager<O: Organism> {
    id: String,
    registry: ComponentRegistry<O>,
    layers: Vec<LayerProcessor<O>>,
    layers_sync: Vec<Mutex<LayerProcessorSync<O>>>,
    missing_layers: Vec<&'static LayerType>,
    first_update: bool,
}

impl<O: Organism> LayerManager<O> {

    fn create(
        layers: Vec<LayerProcessor<O>>,
        layers_sync: Vec<Mutex<LayerProcessorSync<O>>>,
        missing_layers: Vec<&'static LayerType>,
    ) -> Self {
        Self {
            id: Alphanumeric.sample_string(&mut rand::thread_rng(), 16),
            registry: ComponentRegistry::new(),
            first_update: false,
            layers,
            layers_sync,
            missing_layers: missing_layers,
        }
    }

    /// Creates a sequential LayerManager with all layers
    pub fn new() -> Self {
        Self::create(
            vec![
                LayerProcessor::new(Core),
                LayerProcessor::new(Circulation),
                LayerProcessor::new(Digestion),
                LayerProcessor::new(Nervous),
            ],
            Vec::new(),
            Vec::new(),
        )
    }
    
    /// Creates a threaded LayerManager with all layers
    pub fn new_threaded() -> Self {
        Self::create(
            Vec::new(),
            vec![
                Mutex::new(LayerProcessorSync::new(Core)),
                Mutex::new(LayerProcessorSync::new(Circulation)),
                Mutex::new(LayerProcessorSync::new(Digestion)),
                Mutex::new(LayerProcessorSync::new(Nervous)),
            ],
            Vec::new(),
        )
    }

    /// Creates a sequential LayerManager with a specified set of layers
    pub fn new_custom(mut layer_types: HashSet<LayerType>) -> Self {
        // always include Core
        let mut layers = vec![LayerProcessor::new(Core)];

        // Make sure we don't add Core twice
        layer_types.remove(&Core);

        layers.extend(layer_types.iter().map(|lt| LayerProcessor::new(*lt)));

        Self::create(
            layers,
            Vec::new(),
            LayerType::VARIANTS
                .into_iter()
                .filter(|lt| !(matches!(lt, Core) || layer_types.contains(lt)))
                .collect(),
        )
    }

    /// Creates a threaded LayerManager with a specified set of layers
    pub fn new_custom_threaded(mut layer_types: HashSet<LayerType>) -> Self {
        // always include Core
        let mut layers = vec![Mutex::new(LayerProcessorSync::new(Core))];

        // Make sure we don't add Core twice
        layer_types.remove(&Core);

        layers.extend(layer_types.iter().map(|lt| Mutex::new(LayerProcessorSync::new(*lt))));

        Self::create(
            Vec::new(),
            layers,
            LayerType::VARIANTS
                .into_iter()
                .filter(|lt| !(matches!(lt, Core) || layer_types.contains(lt)))
                .collect(),
        )
    }

    /// Whether the first update has occurred
    pub fn first_update(&self) -> bool {
        self.first_update
    }

    /// Whether this LayerManager is threaded or not
    pub fn is_threaded(&self) -> bool {
        self.layers.is_empty()
    }

    /// Checks whether the given component uses any layers
    /// that are not supported by this LayerManager
    fn check_layers(
        missing_layers: &Vec<&'static LayerType>,
        component: &mut Box<dyn ComponentWrapper<O>>,
    ) -> anyhow::Result<()>{
        if !missing_layers.is_empty() {
            if missing_layers.iter().any(|lt| component.has_layer(lt)) {
                return Err(anyhow!(
                    "Layer types [{:?}] are not supported for this Sim!",
                    missing_layers
                ));
            }
        }
        Ok(())
    }

    /// Initial setup for a component
    fn setup_component(
        layers: &mut Vec<LayerProcessor<O>>,
        layers_sync: &mut Vec<Mutex<LayerProcessorSync<O>>>,
        connector: &mut SimConnector,
        wrapper: &mut Box<dyn ComponentWrapper<O>>
    ) {
        if layers_sync.is_empty() {
            for layer in layers.iter_mut() {
                if wrapper.has_layer(&layer.layer_type()) {
                    log::debug!("Setting up layer {:?} for component {}", layer.layer_type(), wrapper.id());
                    layer.setup_component(connector, wrapper);
                }
            }
        }
        else {
            for layer in layers_sync.iter_mut() {
                let mut locked_layer = layer.lock().unwrap();
                if wrapper.has_layer(&locked_layer.layer_type()) {
                    log::debug!("Setting up layer {:?} (sync) for component {}", locked_layer.layer_type(), wrapper.id());
                    locked_layer.setup_component_sync(connector, wrapper);
                }
            }
        }
    }

    fn process_removal(
        layers: &mut Vec<LayerProcessor<O>>,
        layers_sync: &mut Vec<Mutex<LayerProcessorSync<O>>>,
        connector: &mut SimConnector,
        wrapper: &mut Box<dyn ComponentWrapper<O>>) {
        if layers_sync.is_empty() {
            for layer in layers.iter_mut() {
                if wrapper.has_layer(&layer.layer_type()) {
                    log::debug!("Removing component {} from layer {:?}", wrapper.id(), layer.layer_type());
                    layer.remove_component(connector, wrapper);
                }
            }
        }
        else {
            for layer in layers_sync.iter_mut() {
                let mut locked_layer = layer.lock().unwrap();
                if wrapper.has_layer(&locked_layer.layer_type()) {
                    log::debug!("Removing component {} from layer {:?}", wrapper.id(), locked_layer.layer_type());
                    locked_layer.remove_component_sync(connector, wrapper);
                }
            }
        }
    }

    /// Registers and initializes a new component with this LayerManager
    pub fn add_component(
        &mut self, connector: &mut SimConnector,
        component: impl SimComponent<O>
    ) -> anyhow::Result<&'_ mut Box<dyn ComponentWrapper<O>>> {
        let wrapper = self.registry.add_component(component)?;
        Self::check_layers(&self.missing_layers, wrapper)?;
        Self::setup_component(&mut self.layers, &mut self.layers_sync, connector, wrapper);
        Ok(wrapper)
    }

    /// Registers and initializes a new component with this LayerManager from
    /// the given ComponentFactory
    pub fn add_component_from_factory<'a>(
        &mut self,
        connector: &mut SimConnector,
        factory: &mut ComponentFactory<'a, O>,
    ) -> anyhow::Result<&'_ mut Box<dyn ComponentWrapper<O>>> {
        let wrapper = factory.attach(&mut self.registry);
        Self::check_layers(&self.missing_layers, wrapper)?;
        Self::setup_component(&mut self.layers, &mut self.layers_sync, connector, wrapper);
        Ok(wrapper)
    }

    /// Unregisters and removes a component from this LayerManager
    pub fn remove_component(&mut self, connector: &mut SimConnector, component_id: &str) -> anyhow::Result<Box<dyn ComponentWrapper<O>>> {
        match self.registry.remove_component(component_id) {
            Ok(mut wrapper) => {
                Self::process_removal(&mut self.layers, &mut self.layers_sync, connector, &mut wrapper);
                Ok(wrapper)
            },
            Err(msg) => Err(msg),
        }
    }

    /// Retrieves an iterator of all registered components
    pub fn components(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.registry.all_components().map(|c| c.id())
    }

    /// Checks whether the given id corresponds to a registered component
    pub fn has_component(&self, component_id: &str) -> bool {
        self.registry.has_component(component_id)
    }

    fn update_sequential(&mut self, connector: &mut SimConnector) {
        log::trace!("Running sequential update");
        for layer in self.layers.iter_mut() {
            log::trace!("Running pre_exec for layer {:?}", layer.layer_type());
            layer.pre_exec(connector);
        }

        let mut update_list;

        if !self.first_update {
            // If we haven't executed the first update,
            // let ALL components run
            log::trace!("Staging all components for initial run");
            update_list = self.registry.all_components_mut().collect();
        }
        else {
            update_list = Vec::new();
            for component in self.registry.all_components_mut() {
                log::trace!("Checking component {}", component.id());
                let mut check_list = self
                    .layers
                    .iter_mut()
                    .filter(|l| component.has_layer(&l.layer_type()));

                // If any of the supported layers indicate the component should be
                // triggered, add the component to the update list
                if check_list.any(|l| l.check_component(component)) {
                    log::trace!("Component {} staged for a run", component.id());
                    update_list.push(component);
                }
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
                log::trace!("Preparing component {} with layer {:?}", component.id(), layer.layer_type());
                layer.prepare_component(connector, component);
            }

            // Execute component logic
            log::trace!("Executing component {}", component.id());
            component.run();

            // Execute post run processing
            for layer in layer_list.iter_mut() {
                log::trace!("Processing component {} with layer {:?}", component.id(), layer.layer_type());
                layer.process_component(connector, component);
            }
        }

        for layer in self.layers.iter_mut() {
            log::trace!("Running post_exec for layer {:?}", layer.layer_type());
            layer.post_exec(connector);
        }
    }

    fn update_threaded(&mut self, connector: &mut SimConnector) {
        log::trace!("Running threaded update");
        for layer in self.layers_sync.iter_mut() {
            let mut locked_layer = layer.lock().unwrap();
            log::trace!("Running pre_exec_sync for layer {:?}", locked_layer.layer_type());
            locked_layer.pre_exec_sync(connector);
        }

        let mut update_list;

        if !self.first_update {
            // If we haven't executed the first update,
            // let ALL components run
            log::trace!("Staging all components for initial run");
            update_list = self.registry.all_components_mut().collect();
        }
        else {
            update_list = Vec::new();

            for component in self.registry.all_components_mut() {
                log::trace!("Checking component {}", component.id());
                let mut check_list = self
                    .layers_sync
                    .iter_mut()
                    .filter(|l| component.has_layer(&l.lock().unwrap().layer_type()));

                // If any of the supported layers indicate the component should be
                // triggered, add the component to the update list
                if check_list.any(|l| l.lock().unwrap().check_component_sync(component)) {
                    log::trace!("Component {} staged for a run", component.id());
                    update_list.push(component);
                }
            }
        }

        let layers = &self.layers_sync;
        let mconnector = Mutex::new(connector);

        scope(|s| {
            for component in update_list {
                s.spawn(|| {
                    // Prepare the component with each of the associated layers
                    // have to collect here to avoid conflicting borrows of component
                    let mut layer_list: Vec<&Mutex<LayerProcessorSync<O>>> = layers
                        .iter()
                        .filter(|l| component.has_layer(&l.lock().unwrap().layer_type()))
                        .collect();

                    for layer in layer_list.iter_mut() {
                        let mut locked_layer = layer.lock().unwrap();
                        log::trace!("Preparing component {} with layer {:?}", component.id(), locked_layer.layer_type());
                        locked_layer.prepare_component_sync(mconnector.lock().unwrap().borrow_mut(), component);
                    }

                    // Execute component logic
                    log::trace!("Executing component {}", component.id());
                    component.run();

                    // Execute post run processing
                    for layer in layer_list.iter_mut() {
                        let mut locked_layer = layer.lock().unwrap();
                        log::trace!("Processing component {} with layer {:?}", component.id(), locked_layer.layer_type());
                        locked_layer.process_component_sync(mconnector.lock().unwrap().borrow_mut(), component);
                    }
                });
            }
        });

        let reclaimed_connector = mconnector.into_inner().unwrap();
        for layer in self.layers_sync.iter_mut() {
            let mut locked_layer = layer.lock().unwrap();
            log::trace!("Running post_exec_sync for layer {:?}", locked_layer.layer_type());
            locked_layer.post_exec_sync(reclaimed_connector);
        }
    }

    /// Executes an update across all layers and registered components
    pub fn update(&mut self, connector: &mut SimConnector) {
        if self.is_threaded() {
            self.update_threaded(connector)
        }
        else {
            self.update_sequential(connector)
        }
    }
}
