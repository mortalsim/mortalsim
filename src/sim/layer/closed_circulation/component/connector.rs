use super::super::vessel::{BloodVessel, BloodVesselType, VesselIter};
use crate::sim::SimTime;
use crate::substance::{SubstanceConcentration, Substance, SubstanceStore, SubstanceChange};
use crate::util::{BoundFn, IdType, IdGenerator};
use super::ClosedCircInitializer;
use anyhow::{Result, Error};
use petgraph::Direction;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Mutex;

pub struct ClosedCircConnector<V: BloodVessel> {
    id_gen: IdGenerator,
    pub(crate) id_map: HashMap<IdType, IdType>,
    /// Copy of the current simulation time
    pub(crate) sim_time: SimTime,
    pub(crate) all_attached: bool,
    pub(crate) vessel_connections: HashSet<V>,
    pub(crate) substance_notifies: HashMap<V, HashMap<Substance, SubstanceConcentration>>,
    pub(crate) pending_changes: HashMap<V, HashMap<IdType, (Substance, SubstanceChange)>>,
    pub(crate) unschedules: Vec<IdType>,
    pub(crate) unschedule_all: bool,
}

impl<V: BloodVessel> ClosedCircConnector<V> {
    pub fn new() -> ClosedCircConnector<V> {
        ClosedCircConnector {
            id_gen: IdGenerator::new(),
            id_map: HashMap::new(),
            sim_time: SimTime::from_s(0.0),
            all_attached: false,
            vessel_connections: HashSet::new(),
            substance_notifies: HashMap::new(),
            pending_changes: HashMap::new(),
            unschedules: Vec::new(),
            unschedule_all: true,
        }
    }

    /// Retrieves the current simulation time
    pub fn sim_time(&self) -> SimTime {
        self.sim_time
    }

    /// Schedule a substance change on a given Vessel
    /// with a sigmoid shape over the given duration,
    /// startinig immediately.
    /// 
    /// Panics if `duration <= 0`
    ///
    /// ### Arguments
    /// * `substance`  - the substance to change
    /// * `amount`     - total concentration change to take place
    /// * `duration`   - amount of time over which the change takes place
    ///
    /// Returns an id corresponding to this change, if successful
    pub fn schedule_change(&mut self, vessel: V, substance: Substance, amount: SubstanceConcentration, duration: SimTime) -> Result<IdType> {
        if self.all_attached || self.vessel_connections.contains(&vessel) {
            // Constrain the start time to a minimum of the current sim time
            let local_change_id = self.id_gen.get_id();
            self.pending_changes.entry(vessel)
                .or_insert(HashMap::new())
                .insert(local_change_id, (substance, SubstanceChange::new(self.sim_time, amount, duration, crate::util::BoundFn::Sigmoid)));
            Ok(local_change_id)
        }
        else {
            Err(anyhow!("Vessel '{}' is not attached", vessel))
        }
    }

    /// Schedule a substance change on a given Vessel
    /// with a custom shape over the given duration.
    ///
    /// Panics if `delay < 0` or `duration <= 0`
    ///
    /// ### Arguments
    /// * `delay`      - future simulation time to start the change
    /// * `substance`  - the substance to change
    /// * `amount`     - total concentration change to take place
    /// * `duration`   - amount of time over which the change takes place
    /// * `bound_fn`   - the shape of the function
    ///
    /// Returns an id corresponding to this change
    pub fn schedule_custom_change(
        &mut self,
        vessel: V,
        substance: Substance,
        amount: SubstanceConcentration,
        delay: SimTime,
        duration: SimTime,
        bound_fn: BoundFn,
    ) -> Result<IdType> {
        if self.all_attached || self.vessel_connections.contains(&vessel) {
            let x_start_time = {
                if delay.s < 0.0 {
                    panic!("Delay cannot be less than zero!");
                }
                self.sim_time + delay
            };
            let local_change_id = self.id_gen.get_id();
            self.pending_changes.entry(vessel)
                .or_insert(HashMap::new())
                .insert(local_change_id, (substance, SubstanceChange::new(x_start_time, amount, duration, bound_fn)));
            Ok(local_change_id)
        }
        else {
            Err(anyhow!("Vessel '{}' is not attached", vessel))
        }
    }

    /// Unschedule a substance change on this store
    ///
    /// ### Arguments
    /// * `substance` - the substance which was scheduled to be changed
    /// * `change_id` - the id returned from the call to schedule_change
    ///
    /// Returns the provided BoundFn if found and the change hasn't completed, None otherwise
    pub fn unschedule_change(
        &mut self,
        change_id: &IdType,
    ) -> Result<()> {
        // Here we expect that the id_map has been updated to link the local
        // id to the id on the actual SubstanceStore, which is managed elsewhere
        match self.id_map.remove(change_id) {
            Some(store_id) => {
                self.unschedules.push(store_id);
                Ok(())
            }
            None => Err(anyhow!("Invalid id provided : {}", change_id))
        }
    }

    /// Whether to unschedule all previously scheduled substance changes (default is true)
    /// Set to `false` in order to manually specify which substance changes to unschedule
    /// using `unschedule_change`
    pub fn unschedule_all(&mut self, setting: bool) {
        self.unschedule_all = setting;
    }
}

#[cfg(test)]
pub mod test {


    use simple_si_units::chemical::Concentration;

    use crate::{sim::{layer::closed_circulation::vessel::test::TestBloodVessel, SimTime}, substance::{SubstanceStore, Substance}, util::mmol_per_L};

    use super::ClosedCircConnector;

    #[test]
    fn test_sim_time() {
        let ccc = ClosedCircConnector::<TestBloodVessel>::new();
        assert_eq!(ccc.sim_time(), SimTime::from_s(0.0));
    }

    #[test]
    fn test_schedule_change() {
        let mut ccc = ClosedCircConnector::<TestBloodVessel>::new();
        ccc.vessel_connections.insert(TestBloodVessel::VenaCava);
        assert!(ccc.schedule_change(TestBloodVessel::VenaCava, Substance::GLC, mmol_per_L!(1.0), SimTime::from_s(1.0)).is_ok());
        assert!(ccc.pending_changes.contains_key(&TestBloodVessel::VenaCava));
    }

    #[test]
    fn test_bad_schedule_change() {
        let mut ccc = ClosedCircConnector::<TestBloodVessel>::new();
        ccc.vessel_connections.insert(TestBloodVessel::VenaCava);
        assert!(ccc.schedule_change(TestBloodVessel::Aorta, Substance::GLC, mmol_per_L!(1.0), SimTime::from_s(1.0)).is_err());
        assert!(ccc.pending_changes.is_empty());
    }

    #[test]
    fn test_schedule_custom_change() {
        let mut ccc = ClosedCircConnector::<TestBloodVessel>::new();
        ccc.vessel_connections.insert(TestBloodVessel::VenaCava);
        assert!(ccc.schedule_custom_change(TestBloodVessel::VenaCava, Substance::GLC, mmol_per_L!(1.0), SimTime::from_s(1.0), SimTime::from_s(1.0), crate::util::BoundFn::Linear).is_ok());
        assert!(ccc.pending_changes.contains_key(&TestBloodVessel::VenaCava));
    }

    #[test]
    fn test_bad_schedule_custom_change() {
        let mut ccc = ClosedCircConnector::<TestBloodVessel>::new();
        ccc.vessel_connections.insert(TestBloodVessel::VenaCava);
        assert!(ccc.schedule_custom_change(TestBloodVessel::Aorta, Substance::GLC, mmol_per_L!(1.0), SimTime::from_s(1.0), SimTime::from_s(1.0), crate::util::BoundFn::Linear).is_err());
        assert!(ccc.pending_changes.is_empty());
    }

    #[test]
    fn test_unschedule() {
        let mut ccc = ClosedCircConnector::<TestBloodVessel>::new();
        ccc.id_map.insert(1, 10); 
        assert!(ccc.unschedule_change(&1).is_ok());
    }

    #[test]
    fn test_bad_unschedule() {
        let mut ccc = ClosedCircConnector::<TestBloodVessel>::new();
        assert!(ccc.unschedule_change(&1).is_err());
    }

    #[test]
    fn test_unschedule_all() {
        let mut ccc = ClosedCircConnector::<TestBloodVessel>::new();
        assert!(ccc.unschedule_all);
        ccc.unschedule_all(false);
        assert!(!ccc.unschedule_all);
    }

}
