use std::collections::{HashMap, VecDeque};
use std::result;

use crate::units::base::{Amount, Mass};
use crate::units::geometry::Volume;

use crate::sim::layer::digestion::Consumable;
use crate::sim::layer::digestion::DigestionDirection;
use crate::sim::SimTime;
use crate::substance::substance_wrapper::substance_store_wrapper;
use crate::substance::Substance;
use crate::IdGenerator;
use crate::math::BoundFn;
use crate::IdType;

use super::consumable::VolumeChange;

#[derive(Debug)]
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
    /// Local list of active volume changes to this consumable
    pub(crate) vol_changes: Vec<IdType>,
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
            vol_changes: Vec::new(),
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

    /// Schedules a future change in solution volume with a sigmoidal shape.
    ///
    /// Note that all substance and volume changes will be
    /// unset when the `Consumed` moves on to the next component.
    ///
    /// ### Arguments
    /// * `amount`   - magnitude of the volume change
    /// * `start`    - simulation time to start the change
    /// * `end`      - simulation time to end the change
    ///
    /// Returns an id corresponding to the change
    pub fn schedule_volume_change(&mut self,
        amount: Volume<f64>,
        start: SimTime,
        end: SimTime,
        ) -> IdType {
            self.schedule_custom_volume_change(amount, start, end, BoundFn::Sigmoid)
    }

    /// Schedules a future change in solution volume with a custom shape.
    ///
    /// Note that all substance and volume changes will be
    /// unset when the `Consumed` moves on to the next component.
    ///
    /// ### Arguments
    /// * `amount`   - magnitude of the volume change
    /// * `start`    - simulation time to start the change
    /// * `end`      - simulation time to end the change
    /// * `bound_fn` - the shape of the function
    ///
    /// Returns an id corresponding to the change
    pub fn schedule_custom_volume_change(&mut self,
        amount: Volume<f64>,
        start: SimTime,
        end: SimTime,
        bound_fn: BoundFn,
        ) -> IdType {
            let cid = self.consumable.schedule_custom_volume_change(amount, start, end, bound_fn);
            self.vol_changes.push(cid);
            cid
    }
    
    /// Unschedules a previously scheduled volume change the `Consumable`
    /// 
    /// ### Arguments
    /// * `change_id`` - change id returned from a previous volume change schedule
    ///
    /// Will return the scheduled VolumeChange if the change_id is valid and it hasn't completed yet
    pub fn unschedule_volume_change(&mut self, change_id: IdType) -> Option<VolumeChange> {
        self.consumable.unschedule_volume_change(change_id)
    }

    /// Time since the `Consumed` entered the current component
    pub fn time_since_entry(&self) -> SimTime {
        self.entry_time - self.sim_time
    }

    /// Sets the exit time and direction of the `Consumed`
    ///
    /// ### Arguments
    /// * `exit_time`      - simulation time when the `Consumed` should exit the component
    /// * `exit_direction` - direction in which the `Consumed` is exiting
    pub fn set_exit(
        &mut self,
        exit_time: SimTime,
        exit_direction: DigestionDirection,
    ) -> anyhow::Result<()> {
        if exit_time < self.sim_time {
            Err(anyhow!(
                "Consumed exit time must not be less than the current time!"
            ))
        } else {
            self.exit_time = exit_time;
            self.exit_direction = exit_direction;
            Ok(())
        }
    }

    pub(crate) fn clear_all_changes(&mut self) {
        for (substance, change_ids) in self.change_map.drain() {
            for change_id in change_ids {
                self.consumable
                    .store
                    .unschedule_change(&substance, &change_id);
            }
        }
        for change_id in self.vol_changes.drain(..) {
            self.consumable.unschedule_volume_change(change_id);
        }
    }

    /// Remove all pending changes, and extract the consumable and its exit direction
    pub(crate) fn exit(mut self) -> (Consumable, DigestionDirection) {
        self.clear_all_changes();
        (self.consumable, self.exit_direction)
    }

    /// Advance simulation time to the given value.
    pub(crate) fn advance(&mut self, sim_time: SimTime) {
        self.consumable.advance(sim_time);
    }
}

#[cfg(test)]
pub mod test {
    use crate::sim::layer::digestion::DigestionDirection;
    use crate::units::base::{Amount, Mass};
    use crate::units::geometry::Volume;

    use crate::secs;
    use crate::{sim::Consumable, substance::Substance};

    use super::Consumed;

    #[test]
    fn consumed_workflow() {
        let mut food = Consumable::new(Volume::from_mL(250.0));
        food.set_volume_composition(Substance::Amylopectin, 0.5).unwrap();
        food.set_volume_composition(Substance::Amylose, 0.25).unwrap();
        let mut consumed = Consumed::new(food);

        assert!(consumed.amount_of(&Substance::Amylopectin) > Amount::from_mol(0.0));
        assert!(consumed.volume_of(&Substance::Amylopectin) > Volume::from_L(0.0));
        assert!(consumed.mass_of(&Substance::Amylopectin) > Mass::from_g(0.0));

        consumed.schedule_volume_change(Volume::from_mL(-100.0), secs!(10.0), secs!(50.0));
        consumed.schedule_volume_change(Volume::from_mL(-50.0), secs!(30.0), secs!(50.0));
        consumed.set_exit(secs!(40.0), DigestionDirection::FORWARD).unwrap();

        consumed.advance(secs!(20.0));

        // The first volume change should have started
        assert!(consumed.volume() < Volume::from_mL(250.0));

        consumed.advance(secs!(45.0));
        assert_eq!(consumed.exit_time, secs!(40.0));

        let (mut food, exit_dir) = consumed.exit();
        assert_eq!(exit_dir, DigestionDirection::FORWARD);
        let exit_vol = food.volume();

        // Advance consumable, Volume changes should no longer happen
        food.advance(secs!(120.0));
        assert_eq!(food.volume(), exit_vol);

    }

    #[test]
    fn consumed_bad_exit() {
        let mut food = Consumable::new(Volume::from_mL(250.0));
        food.set_volume_composition(Substance::Amylopectin, 0.5).unwrap();
        food.set_volume_composition(Substance::Amylose, 0.25).unwrap();
        let mut consumed = Consumed::new(food);

        assert!(consumed.set_exit(secs!(-1.0), DigestionDirection::FORWARD).is_err());
    }
}