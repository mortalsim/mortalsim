use crate::sim::Organism;
use crate::sim::component::SimComponentProcessor;
use crate::sim::component::registry::ComponentWrapper;

use super::{
    SimLayer,
    LayerType,
    CoreLayer,
    CirculationLayer,
    DigestionLayer,
    NervousLayer,
};

pub enum LayerProcessor<O: Organism + ?Sized> {
    Core(CoreLayer<O>),
    Circulation(CirculationLayer<O>),
    Digestion(DigestionLayer<O>),
    Nervous(NervousLayer<O>),
}

impl<O: Organism + ?Sized + 'static> LayerProcessor<O> {
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

impl<O: Organism + ?Sized> SimLayer for LayerProcessor<O> {
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

impl<O: Organism + ?Sized, T: ComponentWrapper<O>> SimComponentProcessor<O, T> for LayerProcessor<O> {
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
