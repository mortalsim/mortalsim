use std::collections::hash_set;
use std::fmt::Debug;

pub mod circulation;
pub mod core;
pub mod digestion;
pub mod layer_manager;
pub mod layer_processor;
pub mod nervous;

use crate::event::Event;

pub use self::core::{CoreComponent, CoreConnector, CoreInitializer, CoreLayer};
pub use circulation::{
    BloodStore, BloodVessel, CirculationComponent, CirculationConnector, CirculationInitializer,
    CirculationLayer,
};
pub use digestion::{
    DigestionComponent, DigestionConnector, DigestionDirection, DigestionInitializer,
    DigestionLayer,
};
pub use layer_manager::*;
pub use nervous::{
    Nerve, NerveIter, NervousComponent, NervousConnector, NervousInitializer, NervousLayer,
};

use super::SimConnector;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, EnumCount, VariantArray)]
pub enum LayerType {
    Core,
    Circulation,
    Digestion,
    Nervous,
}

/// Trait to outline common methods for all sim layers
pub trait SimLayer: Send {
    /// Process layer actions prior to component processing
    fn pre_exec(&mut self, connector: &mut SimConnector);
    /// Process layer actions after component processing
    fn post_exec(&mut self, connector: &mut SimConnector);
    /// Process layer actions prior to component processing (thread safe)
    fn pre_exec_sync(&mut self, connector: &mut SimConnector);
    /// Process layer actions after component processing (thread safe)
    fn post_exec_sync(&mut self, connector: &mut SimConnector);
}

#[derive(Debug, Clone)]
/// Internal Event used to force layer processing
pub(crate) struct InternalLayerTrigger;

impl Event for InternalLayerTrigger {}

pub struct AnatomicalRegionIter<'a, T: Clone>(pub hash_set::Iter<'a, T>);

impl<'a, T: Clone> Iterator for AnatomicalRegionIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        Some(self.0.next()?.clone())
    }
}
