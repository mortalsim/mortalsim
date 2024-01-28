
use std::collections::hash_set;
use std::fmt::Debug;

pub mod core;
pub mod circulation;
pub mod digestion;
pub mod nervous;
pub mod layer_manager;

use crate::event::Event;

pub use self::core::*;
pub use circulation::*;
pub use digestion::*;
pub use nervous::*;
pub use layer_manager::*;

use super::SimConnector;

/// Trait to outline common methods for all sim layers
pub trait SimLayer {
    /// Process layer actions prior to component processing
    fn pre_exec(&mut self, connector: &mut SimConnector);
    /// Process layer actions after component processing
    fn post_exec(&mut self, connector: &mut SimConnector);
}

#[derive(Debug)]
/// Internal Event used to force layer processing
pub(crate) struct InternalLayerTrigger;

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

