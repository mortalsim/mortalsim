use std::collections::HashMap;

use crate::units::base::{Amount, Mass};
use crate::units::geometry::Volume;

use crate::sim::layer::digestion::Consumable;
use crate::sim::layer::digestion::DigestionDirection;
use crate::sim::SimTime;
use crate::substance::substance_wrapper::substance_store_wrapper;
use crate::substance::Substance;
use crate::IdType;

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

    pub fn amount_of(&self, substance: &Substance) -> Amount<f64> {
        self.consumable.amount_of(substance)
    }

    pub fn mass_of(&self, substance: &Substance) -> Mass<f64> {
        self.consumable.mass_of(substance)
    }

    pub fn entry_time(&self) -> SimTime {
        self.entry_time
    }

    pub fn set_exit(
        &mut self,
        exit_time: SimTime,
        exit_direction: DigestionDirection,
    ) -> anyhow::Result<()> {
        if exit_time < self.entry_time {
            Err(anyhow!(
                "Digestion component exit_time cannot be less than entry time!"
            ))
        } else {
            self.exit_time = exit_time;
            self.exit_direction = exit_direction;
            Ok(())
        }
    }

    pub(crate) fn exit(mut self) -> (Consumable, DigestionDirection) {
        for (substance, change_ids) in self.change_map.drain() {
            for change_id in change_ids {
                self.consumable
                    .store
                    .unschedule_change(&substance, &change_id);
            }
        }
        (self.consumable, self.exit_direction)
    }

    pub(crate) fn advance(&mut self, sim_time: SimTime) {
        self.consumable.advance(sim_time)
    }
}
