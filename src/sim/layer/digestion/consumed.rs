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

    /// Creates a new Consumed to wrap a Consumable.
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

    /// Volume of the solution
    pub fn volume(&self) -> Volume<f64> {
        self.consumable.volume()
    }

    /// Amount of the given substance in the solution
    pub fn amount_of(&self, substance: &Substance) -> Amount<f64> {
        self.consumable.amount_of(substance)
    }
    
    /// Volume of the given substance in the solution
    pub fn volume_of(&self, substance: &Substance) -> Volume<f64> {
        self.consumable.volume_of(substance)
    }

    /// Mass of the given substance in the solution
    pub fn mass_of(&self, substance: &Substance) -> Mass<f64> {
        self.consumable.mass_of(substance)
    }

    /// Time since the `Consumed` entered the current component
    pub fn time_since_entry(&self) -> SimTime {
        self.entry_time - self.sim_time
    }

    /// Schedules a future change in solution volume with a sigmoidal shape.
    ///
    /// Note that all substance and volume changes will be
    /// unset when the `Consumed` moves on to the next component.
    ///
    /// ### Arguments
    /// * `amount`   - magnitude of the volume change
    /// * `delay`    - delay in simulation time before starting the change
    /// * `duration` - amount of time over which the change takes place
    pub fn schedule_volume_change(&mut self,
        amount: Volume<f64>,
        delay: SimTime,
        duration: SimTime,
        ) -> IdType {
            self.schedule_custom_volume_change(amount, delay, duration, BoundFn::Sigmoid)
    }

    /// Schedules a future change in solution volume with a custom shape.
    ///
    /// Note that all substance and volume changes will be
    /// unset when the `Consumed` moves on to the next component.
    ///
    /// ### Arguments
    /// * `amount`   - magnitude of the volume change
    /// * `delay`    - delay in simulation time before starting the change
    /// * `duration` - amount of time over which the change takes place
    /// * `bound_fn` - the shape of the function
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

    /// Sets the exit time and direction of the `Consumed`
    ///
    /// ### Arguments
    /// * `delay`          - delay in simulation time before the `Consumed` exits
    /// * `exit_direction` - direction in which the `Consumed` is exiting
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

    /// Remove all pending changes, and extract the consumable and its exit direction
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

    /// Internal execution of volume changes on each advance
    fn execute_volume_changes(&mut self) {
        for change in self.volume_changes.iter() {
            if change.start < self.sim_time && change.end > self.sim_time {
                let result = change.function.call(
                    self.sim_time.s - change.start.s,
                    change.end.s - change.start.s,
                    change.amount.m3,
                );
                // Set unchecked here, since the consumable will check automatically
                // when advancing
                self.consumable.set_volume_unchecked(self.volume() + Volume::from_m3(result));
            }
        }
        // Pop off any volume changes which have completed
        while self.volume_changes.front().is_some_and(|c| c.end < self.sim_time) {
            self.volume_changes.pop_front();
        }
    }

    /// Advance simulation time to the given value.
    ///
    /// Volume changes are executed before the consumable advances,
    /// at which time mass and solute volume are calculated, and validity
    /// of the solution volume is checked.
    pub(crate) fn advance(&mut self, sim_time: SimTime) {
        self.execute_volume_changes();
        self.consumable.advance(sim_time);
    }
}
