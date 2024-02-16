use std::collections::HashMap;
use std::marker::PhantomData;

use crate::sim::layer::digestion::consumable::Consumable;
use crate::sim::layer::digestion::consumed::Consumed;
use crate::sim::layer::digestion::DigestionDirection;
use crate::sim::{Organism, SimTime};
use crate::substance::substance_wrapper::substance_store_wrapper;
use crate::substance::Substance;
use crate::units::geometry::Volume;
use crate::util::IdType;

/// Provides methods for Digestion modules to interact with the simulation
pub struct DigestionConnector<O: Organism> {
    pd: PhantomData<O>,
    /// Copy of the current simulation time
    pub(crate) sim_time: SimTime,
    /// Consumable which is accessible by the current module
    pub(crate) consumed_list: Vec<Consumed>,
}

impl<O: Organism> DigestionConnector<O> {
    /// Creates a new CoreConnector
    pub fn new() -> Self {
        Self {
            pd: PhantomData,
            sim_time: SimTime::from_s(0.0),
            consumed_list: Vec::new(),
        }
    }

    /// Retrieves the current simulation time
    pub fn sim_time(&self) -> SimTime {
        self.sim_time
    }

    /// Iterates through consumed items
    pub fn consumed(&mut self) -> impl Iterator<Item = &mut Consumed> {
        self.consumed_list.iter_mut()
    }
}

#[cfg(test)]
pub mod test {}
