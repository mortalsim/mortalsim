use std::collections::HashMap;

use crate::{sim::{layer::digestion::consumable::Consumable, SimTime}, util::{IdType, BoundFn}, substance::{Substance, SubstanceChange, SubstanceConcentration}};

pub enum ConsumableExitDirection {
    BACK,
    FORWARD,
    EXHAUSTED,
}

/// Provides methods for Digestion modules to interact with the simulation
pub struct DigestionConnector {
    /// Copy of the current simulation time
    pub(crate) sim_time: SimTime,
    /// Consumable which is accessible by the current module
    pub(crate) consumable: Option<Consumable>,
    /// SimTime duration the consumable should stay with this component
    pub(crate) duration: SimTime,
    /// Map of substance changes to attach to the consumable
    pub(crate) substance_changes: HashMap<Substance, HashMap<IdType, SubstanceChange>>,
    /// Where the consumable should go after this component is done with it (Default is FORWARD)
    pub(crate) exit_direction: ConsumableExitDirection,
}

impl DigestionConnector {
    /// Creates a new CoreConnector
    pub fn new() -> DigestionConnector {
        DigestionConnector {
            sim_time: SimTime::from_s(0.0),
            consumable: None,
            duration: SimTime::from_s(0.0),
            substance_changes: HashMap::new(),
            exit_direction: ConsumableExitDirection::FORWARD,
        }
    }
}

#[cfg(test)]
pub mod test {
    
}
