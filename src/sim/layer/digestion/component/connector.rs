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
    consumable: Consumable,
    /// Time which the consumable entered the component
    entry_time: SimTime,
    /// Time which the consumable will exit the component
    exit_time: SimTime,
    /// Where the consumable should go after this component is done with it (Default is FORWARD)
    exit_direction: ConsumableExitDirection,
    /// Local map of previous changes to this consumable
    change_map: HashMap<Substance, Vec<IdType>>,
}

pub(crate) struct ConsumedMeta {
    /// Time which the consumable entered the component
    pub entry_time: SimTime,
    /// Time which the consumable will exit the component
    pub exit_time: SimTime,
    /// Where the consumable should go after this component is done with it (Default is FORWARD)
    pub exit_direction: ConsumableExitDirection,
    /// Local map of previous changes to this consumable
    pub change_map: HashMap<Substance, Vec<IdType>>,
}

impl ConsumedMeta {
    pub fn extract(consumed: Consumed) -> (Consumable, ConsumedMeta) {
        (consumed.consumable, ConsumedMeta {
            entry_time: consumed.entry_time,
            exit_time: consumed.exit_time,
            exit_direction: consumed.exit_direction,
            change_map: consumed.change_map,
        })
    }
    pub fn enclose(meta: ConsumedMeta, consumable: Consumable) -> Consumed {
        Consumed {
            consumable: consumable,
            entry_time: meta.entry_time,
            exit_time: meta.exit_time,
            exit_direction: meta.exit_direction,
            change_map: meta.change_map,
        }
    }
}

impl Consumed {
    substance_store_wrapper!(consumable.store, change_map);
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
