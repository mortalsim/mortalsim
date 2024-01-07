use std::collections::HashMap;

use crate::sim::layer::digestion::consumable::Consumable;
use crate::sim::SimTime;
use crate::substance::substance_wrapper::substance_store_wrapper;
use crate::substance::{Substance, SubstanceChange};
use crate::util::IdType;

pub enum ConsumableExitDirection {
    BACK,
    FORWARD,
    EXHAUSTED,
}

pub struct Consumed {
    /// Consumable accessible by the current module
    pub(crate) consumable: Consumable,
    /// Time which the consumable entered the component
    pub(crate) entry_time: SimTime,
    /// Time which the consumable will exit the component
    pub(crate) exit_time: SimTime,
    /// Where the consumable should go after this component is done with it (Default is FORWARD)
    pub(crate) exit_direction: ConsumableExitDirection,
}

impl Consumed {
    substance_store_wrapper!(consumable.store);
}


/// Provides methods for Digestion modules to interact with the simulation
pub struct DigestionConnector {
    /// Copy of the current simulation time
    pub(crate) sim_time: SimTime,
    /// Consumable which is accessible by the current module
    pub(crate) consumed_list: Vec<Consumed>,
}

impl DigestionConnector {
    /// Creates a new CoreConnector
    pub fn new() -> DigestionConnector {
        DigestionConnector {
            sim_time: SimTime::from_s(0.0),
            consumed_list: Vec::new(),
        }
    }

    /// Retrieves the current simulation time
    pub fn get_time(&self) -> SimTime {
        self.sim_time
    }
}

#[cfg(test)]
pub mod test {
    
}
