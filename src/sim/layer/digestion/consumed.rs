use std::collections::{HashMap, VecDeque};
use std::result;

use crate::units::base::{Amount, Mass};
use crate::units::geometry::Volume;

use crate::sim::layer::digestion::Consumable;
use crate::sim::layer::digestion::DigestionDirection;
use crate::sim::SimTime;
use crate::substance::substance_wrapper::substance_store_wrapper;
use crate::substance::Substance;
use crate::util::{BoundFn, IdGenerator};
use crate::IdType;

struct VolumeChange {
    id: IdType,
    amount: Volume<f64>,
    function: BoundFn,
    start: SimTime,
    end: SimTime,
}

pub struct Consumed {
    /// Copy of the current Simulation time
    pub(crate) sim_time: SimTime,
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
    /// Id generator for volume change registrations
    id_gen: IdGenerator,
    /// Volume changes
    volume_changes: VecDeque<VolumeChange>,
    /// composite changes (volume & substance)
    composite_changes: HashMap<IdType, (IdType, IdType)>,
}

impl Consumed {
    substance_store_wrapper!(consumable.store, change_map);

    pub(crate) fn new(consumable: Consumable) -> Self {
        Self {
            sim_time: SimTime::from_s(0.0),
            consumable,
            entry_time: SimTime::from_s(0.0),
            entry_direction: DigestionDirection::FORWARD,
            exit_time: SimTime::from_s(60.0),
            exit_direction: DigestionDirection::FORWARD,
            change_map: HashMap::new(),
            id_gen: IdGenerator::new(),
            volume_changes: VecDeque::new(),
            composite_changes: HashMap::new(),
        }
    }

    pub fn volume(&self) -> Volume<f64> {
        self.consumable.volume()
    }

    pub fn amount_of(&self, substance: &Substance) -> Amount<f64> {
        self.consumable.amount_of(substance)
    }
    
    pub fn volume_of(&self, substance: &Substance) -> Volume<f64> {
        self.consumable.volume_of(substance)
    }

    pub fn mass_of(&self, substance: &Substance) -> Mass<f64> {
        self.consumable.mass_of(substance)
    }

    pub fn time_since_entry(&self) -> SimTime {
        self.entry_time - self.sim_time
    }

    pub fn schedule_volume_change(&mut self,
        amount: Volume<f64>,
        delay: SimTime,
        duration: SimTime,
        ) -> IdType {
            self.schedule_custom_volume_change(amount, delay, duration, BoundFn::Sigmoid)
    }

    pub fn schedule_custom_volume_change(&mut self,
        amount: Volume<f64>,
        delay: SimTime,
        duration: SimTime,
        bound_fn: BoundFn,
        ) -> IdType {
            let change_id = self.id_gen.get_id();
            self.volume_changes.push_back(VolumeChange {
                id: change_id,
                amount: amount,
                function: bound_fn,
                start: self.sim_time() + delay,
                end: self.sim_time() + delay + duration,
            });
            change_id
    }

    pub fn schedule_extract(
        &mut self,
        substance: &Substance,
        amount: Amount<f64>,
        duration: SimTime,
    ) -> anyhow::Result<IdType> {
        self.schedule_custom_extract(substance, amount, SimTime::from_s(0.0), duration, BoundFn::Sigmoid)
    }

    pub fn schedule_custom_extract(
        &mut self,
        substance: &Substance,
        amount: Amount<f64>,
        delay: SimTime,
        duration: SimTime,
        bound_fn: BoundFn,
    ) -> anyhow::Result<IdType> {
        let cur_amt_substance = self.amount_of(substance);
        if cur_amt_substance < amount {
            return Err(anyhow!(
                "Cannot extract {} of substance. Only {} remains.",
                amount,
                self.amount_of(substance)
            ))
        }
        let result_volume = (amount - cur_amt_substance) * (substance.molar_mass() / substance.density());
        let change_a = self.schedule_custom_volume_change(result_volume, delay, duration, bound_fn);

        let result_concentration = (amount / self.amount_of(substance)) * self.concentration_of(substance);
        let change_b = self.schedule_custom_change(*substance, result_concentration, delay, duration, bound_fn);

        let comp_id = self.id_gen.get_id();
        self.composite_changes.insert(comp_id, (change_a, change_b));

        Ok(comp_id)
    }

    pub fn set_exit(
        &mut self,
        delay: SimTime,
        exit_direction: DigestionDirection,
    ) -> anyhow::Result<()> {
        if delay <= SimTime::from_s(0.0) {
            Err(anyhow!(
                "Consumed exit delay must be greater than zero!"
            ))
        } else {
            self.exit_time = delay + self.sim_time;
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
        self.volume_changes.drain(..);
        (self.consumable, self.exit_direction)
    }

    fn execute_volume_changes(&mut self) {
        for change in self.volume_changes.iter() {
            if change.start < self.sim_time && change.end > self.sim_time {
                let result = change.function.call(
                    self.sim_time.s - change.start.s,
                    change.end.s - change.start.s,
                    change.amount.m3,
                );
                // Log volume errors as warnings, but continue on
                if let Err(e) = self.consumable.set_volume(self.volume() + Volume::from_m3(result)) {
                    log::warn!("{}", e);
                }
            }
        }
        // Pop off any volume changes which have completed
        while self.volume_changes.front().is_some_and(|c| c.end < self.sim_time) {
            self.volume_changes.pop_front();
        }
    }

    pub(crate) fn advance(&mut self, sim_time: SimTime) {
        self.consumable.advance(sim_time);
        self.execute_volume_changes();
    }
}
