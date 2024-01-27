
use std::collections::hash_set;
use std::fmt::Debug;

pub mod core;
pub mod circulation;
pub mod digestion;
pub mod nervous;
pub mod layer_manager;

use crate::event::Event;

pub use self::core::component::*;
pub use circulation::component::*;
pub use digestion::component::*;
pub use nervous::component::*;
pub use layer_manager::LayerManager;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum SimLayer {
    Core,
    Circulation,
    Digestion,
    Nervous,
}

#[derive(Debug)]
pub(crate) struct InternalLayerTrigger(SimLayer);

impl InternalLayerTrigger {
    pub fn layer(&self) -> SimLayer {
        self.0
    }
}

impl Event for InternalLayerTrigger {
    fn event_name(&self) -> &str {
        "InternalLayerEvent"
    }
}

pub struct AnatomicalRegionIter<'a, T: Clone>(pub hash_set::Iter<'a, T>);

impl<'a, T: Clone> Iterator for AnatomicalRegionIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        Some(self.0.next()?.clone())
    }
}

