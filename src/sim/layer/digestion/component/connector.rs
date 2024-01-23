use std::collections::HashMap;
use std::marker::PhantomData;

use crate::sim::layer::digestion::DigestionDirection;
use crate::units::geometry::Volume;
use crate::sim::{SimTime, Organism};
use crate::sim::layer::digestion::consumable::Consumable;
use crate::substance::substance_wrapper::substance_store_wrapper;
use crate::substance::Substance;
use crate::util::IdType;

pub struct Consumed {
    /// Consumable accessible by the current module
    pub(crate) consumable: Consumable,
    /// Time which the consumable entered the component
    pub(crate) entry_time: SimTime,
    /// Where the consumable should go after this component is done with it (Default is FORWARD)
    pub(crate) entry_direction: DigestionDirection,
    /// Time which the consumable will exit the component.
    /// (Default is 60s after entry time)
    /// Note: if there are any lingering substance changes, they will be
    /// cancelled at exit time
    pub(crate) exit_time: SimTime,
    /// Where the consumable should go after this component is done with it (Default is FORWARD)
    pub(crate) exit_direction: DigestionDirection,
    /// Local map of previous changes to this consumable
    pub(crate) change_map: HashMap<Substance, Vec<IdType>>,
}

impl Consumed {
    substance_store_wrapper!(consumable.store, change_map);

    pub(crate) fn new(consumable: Consumable) -> Self {
        Self {
            consumable,
            entry_time: SimTime::from_s(0.0),
            entry_direction: DigestionDirection::FORWARD,
            exit_time: SimTime::from_s(60.0),
            exit_direction: DigestionDirection::FORWARD,
            change_map: HashMap::new(),
        }
    }

    pub fn volume(&self) -> Volume<f64> {
        self.consumable.volume()
    }

    pub fn set_volume(&mut self, volume: Volume<f64>) -> anyhow::Result<()> {
        self.consumable.set_volume(volume)
    }

    pub fn entry_time(&self) -> SimTime {
        self.entry_time
    }

    pub fn set_exit(&mut self, exit_time: SimTime, exit_direction: DigestionDirection) -> anyhow::Result<()> {
        if exit_time < self.entry_time {
            Err(anyhow!("Digestion component exit_time cannot be less than entry time!"))
        }
        else {
            self.exit_time = exit_time;
            self.exit_direction = exit_direction;
            Ok(())
        }
    }

    pub(crate) fn exit(mut self) -> (Consumable, DigestionDirection) {
        for (substance, change_ids) in self.change_map.drain() {
            for change_id in change_ids {
                self.consumable.store.unschedule_change(&substance, &change_id);
            }
        }
        (self.consumable, self.exit_direction)
    }
    
    pub(crate) fn advance(&mut self, sim_time: SimTime) {
        self.consumable.advance(sim_time)
    }
}

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
    pub fn get_time(&self) -> SimTime {
        self.sim_time
    }

    /// Iterates through consumed items
    pub fn consumed(&mut self) -> impl Iterator<Item = &mut Consumed> {
        self.consumed_list.iter_mut()
    }
}

#[cfg(test)]
pub mod test {
    
}
