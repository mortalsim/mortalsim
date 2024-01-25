
use std::{collections::hash_set, path::Component};

pub mod core;
pub mod circulation;
pub mod digestion;
pub mod nervous;

pub use self::core::component::*;
use self::{circulation::CirculationLayer, core::CoreLayer, digestion::digestion_layer::DigestionLayer, nervous::nervous_layer::NervousLayer};
pub use circulation::component::*;
pub use digestion::component::*;
pub use nervous::component::*;

use super::{component::{registry::ComponentRegistry, SimComponentProcessor}, Organism};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum SimLayer {
    Core,
    Circulation,
    Digestion,
    Nervous,
}

pub struct AnatomicalRegionIter<'a, T: Clone>(pub hash_set::Iter<'a, T>);

impl<'a, T: Clone> Iterator for AnatomicalRegionIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        Some(self.0.next()?.clone())
    }
}

pub struct LayerManager<O: Organism + 'static> {
    component_registry: ComponentRegistry<O>,
    core_layer: CoreLayer<O>,
    circulation_layer: CirculationLayer<O>,
    digestion_layer: DigestionLayer<O>,
    nervous_layer: NervousLayer<O>,
}

impl<O: Organism> LayerManager<O> {
    pub fn new() -> Self {
        Self {
            component_registry: ComponentRegistry::new(),
            core_layer: CoreLayer::new(),
            circulation_layer: CirculationLayer::new(),
            digestion_layer: DigestionLayer::new(),
            nervous_layer: NervousLayer::new(),
        }
    }
}
