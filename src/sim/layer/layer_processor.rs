use crate::sim::component::{registry::ComponentWrapper, SimComponentProcessorSync};
use crate::sim::component::SimComponentProcessor;
use crate::sim::Organism;

use super::{CirculationLayer, CoreLayer, DigestionLayer, LayerType, NervousLayer, SimLayer, SimLayerSync};

pub enum LayerProcessor<O: Organism> {
    Core(CoreLayer<O>),
    Circulation(CirculationLayer<O>),
    Digestion(DigestionLayer<O>),
    Nervous(NervousLayer<O>),
}

impl<O: Organism + 'static> LayerProcessor<O> {
    pub fn new(layer_type: LayerType) -> Self {
        match layer_type {
            LayerType::Core => Self::Core(CoreLayer::new()),
            LayerType::Circulation => Self::Circulation(CirculationLayer::new()),
            LayerType::Digestion => Self::Digestion(DigestionLayer::new()),
            LayerType::Nervous => Self::Nervous(NervousLayer::new()),
        }
    }
    pub fn layer_type(&self) -> LayerType {
        match self {
            Self::Core(_) => LayerType::Core,
            Self::Circulation(_) => LayerType::Circulation,
            Self::Digestion(_) => LayerType::Digestion,
            Self::Nervous(_) => LayerType::Nervous,
        }
    }
}

impl<O: Organism> SimLayer for LayerProcessor<O> {
    fn pre_exec(&mut self, connector: &mut crate::sim::SimConnector) {
        match self {
            Self::Core(layer) => layer.pre_exec(connector),
            Self::Circulation(layer) => layer.pre_exec(connector),
            Self::Digestion(layer) => layer.pre_exec(connector),
            Self::Nervous(layer) => layer.pre_exec(connector),
        }
    }
    fn post_exec(&mut self, connector: &mut crate::sim::SimConnector) {
        match self {
            Self::Core(layer) => layer.post_exec(connector),
            Self::Circulation(layer) => layer.post_exec(connector),
            Self::Digestion(layer) => layer.post_exec(connector),
            Self::Nervous(layer) => layer.post_exec(connector),
        }
    }
}

impl<O: Organism, T: ComponentWrapper<O>> SimComponentProcessor<O, T> for LayerProcessor<O> {
    fn setup_component(&mut self, connector: &mut crate::sim::SimConnector, component: &mut T) {
        match self {
            Self::Core(layer) => layer.setup_component(connector, component),
            Self::Circulation(layer) => layer.setup_component(connector, component),
            Self::Digestion(layer) => layer.setup_component(connector, component),
            Self::Nervous(layer) => layer.setup_component(connector, component),
        }
    }
    
    fn check_component(&mut self, component: &T) -> bool {
        match self {
            Self::Core(layer) => layer.check_component(component),
            Self::Circulation(layer) => layer.check_component(component),
            Self::Digestion(layer) => layer.check_component(component),
            Self::Nervous(layer) => layer.check_component(component),
        }
    }

    fn prepare_component(&mut self, connector: &mut crate::sim::SimConnector, component: &mut T) {
        match self {
            Self::Core(layer) => layer.prepare_component(connector, component),
            Self::Circulation(layer) => layer.prepare_component(connector, component),
            Self::Digestion(layer) => layer.prepare_component(connector, component),
            Self::Nervous(layer) => layer.prepare_component(connector, component),
        }
    }

    fn process_component(&mut self, connector: &mut crate::sim::SimConnector, component: &mut T) {
        match self {
            Self::Core(layer) => layer.process_component(connector, component),
            Self::Circulation(layer) => layer.process_component(connector, component),
            Self::Digestion(layer) => layer.process_component(connector, component),
            Self::Nervous(layer) => layer.process_component(connector, component),
        }
    }
}


pub enum LayerProcessorSync<O: Organism> {
    Core(CoreLayer<O>),
    Circulation(CirculationLayer<O>),
    Digestion(DigestionLayer<O>),
    Nervous(NervousLayer<O>),
}

impl<O: Organism + 'static> LayerProcessorSync<O> {
    pub fn new(layer_type: LayerType) -> Self {
        match layer_type {
            LayerType::Core => Self::Core(CoreLayer::new()),
            LayerType::Circulation => Self::Circulation(CirculationLayer::new()),
            LayerType::Digestion => Self::Digestion(DigestionLayer::new()),
            LayerType::Nervous => Self::Nervous(NervousLayer::new()),
        }
    }
    pub fn layer_type(&self) -> LayerType {
        match self {
            Self::Core(_) => LayerType::Core,
            Self::Circulation(_) => LayerType::Circulation,
            Self::Digestion(_) => LayerType::Digestion,
            Self::Nervous(_) => LayerType::Nervous,
        }
    }
}

impl<O: Organism> SimLayerSync for LayerProcessorSync<O> {
    fn pre_exec_sync(&mut self, connector: &mut crate::sim::SimConnector) {
        match self {
            Self::Core(layer) => layer.pre_exec_sync(connector),
            Self::Circulation(layer) => layer.pre_exec_sync(connector),
            Self::Digestion(layer) => layer.pre_exec_sync(connector),
            Self::Nervous(layer) => layer.pre_exec_sync(connector),
        }
    }
    fn post_exec_sync(&mut self, connector: &mut crate::sim::SimConnector) {
        match self {
            Self::Core(layer) => layer.post_exec_sync(connector),
            Self::Circulation(layer) => layer.post_exec_sync(connector),
            Self::Digestion(layer) => layer.post_exec_sync(connector),
            Self::Nervous(layer) => layer.post_exec_sync(connector),
        }
    }
}

impl<O: Organism, T: ComponentWrapper<O>> SimComponentProcessorSync<O, T> for LayerProcessorSync<O> {
    fn setup_component_sync(&mut self, connector: &mut crate::sim::SimConnector, component: &mut T) {
        match self {
            Self::Core(layer) => layer.setup_component_sync(connector, component),
            Self::Circulation(layer) => layer.setup_component_sync(connector, component),
            Self::Digestion(layer) => layer.setup_component_sync(connector, component),
            Self::Nervous(layer) => layer.setup_component_sync(connector, component),
        }
    }

    fn check_component_sync(&mut self, component: &T) -> bool {
        match self {
            Self::Core(layer) => layer.check_component_sync(component),
            Self::Circulation(layer) => layer.check_component_sync(component),
            Self::Digestion(layer) => layer.check_component_sync(component),
            Self::Nervous(layer) => layer.check_component_sync(component),
        }
    }

    fn prepare_component_sync(&mut self, connector: &mut crate::sim::SimConnector, component: &mut T) {
        match self {
            Self::Core(layer) => layer.prepare_component_sync(connector, component),
            Self::Circulation(layer) => layer.prepare_component_sync(connector, component),
            Self::Digestion(layer) => layer.prepare_component_sync(connector, component),
            Self::Nervous(layer) => layer.prepare_component_sync(connector, component),
        }
    }

    fn process_component_sync(&mut self, connector: &mut crate::sim::SimConnector, component: &mut T) {
        match self {
            Self::Core(layer) => layer.process_component_sync(connector, component),
            Self::Circulation(layer) => layer.process_component_sync(connector, component),
            Self::Digestion(layer) => layer.process_component_sync(connector, component),
            Self::Nervous(layer) => layer.process_component_sync(connector, component),
        }
    }
}
