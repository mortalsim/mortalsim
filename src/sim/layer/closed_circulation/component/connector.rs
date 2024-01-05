use super::super::vessel::{BloodVessel, BloodVesselType, VesselIter};
use crate::sim::SimTime;
use crate::sim::organism::Organism;
use crate::substance::{SubstanceConcentration, Substance, SubstanceStore, SubstanceChange, ConcentrationTracker};
use crate::util::{BoundFn, IdType, IdGenerator};
use super::ClosedCircInitializer;
use anyhow::{Result, Error};
use petgraph::Direction;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct ClosedCircConnector<O: Organism> {
    /// Local generator for this connector
    id_gen: IdGenerator,
    /// Mapping of local ids to `SubstanceStore` changes
    pub change_map: HashMap<IdType, (O::VesselType, Substance, IdType)>,
    /// Mapping of `BloodVessel`s to their corresponding `SubstanceStore`
    pub vessel_map: HashMap<O::VesselType, SubstanceStore>,
    /// Copy of the current simulation time
    pub(crate) sim_time: SimTime,
    /// Whether all vessels are attached to the associated component
    pub(crate) all_attached: bool,
    /// Set of vessel connections for the associated component
    pub(crate) vessel_connections: HashSet<O::VesselType>,
    /// Map of thresholds for changes to vessels and substances that should trigger
    /// the associated component
    pub(crate) substance_notifies: HashMap<O::VesselType, HashMap<Substance, ConcentrationTracker>>,
    /// Whether all changes should be unscheduled before each run
    /// NOTE: If this is set to false, the component is responsible for
    /// tracking and unscheduling preexisting changes, if necessary
    pub(crate) unschedule_all: bool,
}

impl<O: Organism> ClosedCircConnector<O> {
    pub fn new() -> ClosedCircConnector<O> {
        ClosedCircConnector {
            id_gen: IdGenerator::new(),
            change_map: HashMap::new(),
            vessel_map: HashMap::new(),
            sim_time: SimTime::from_s(0.0),
            all_attached: false,
            vessel_connections: HashSet::new(),
            substance_notifies: HashMap::new(),
            unschedule_all: true,
        }
    }

    /// Retrieves the current simulation time
    pub fn sim_time(&self) -> SimTime {
        self.sim_time
    }

    /// Retrieves the concentration of a given Vessel Substance.
    ///
    /// ### Arguments
    /// * `vessel`    - Vessel to retrieve
    /// * `substance` - Substance to retrieve
    ///
    /// Returns the current concentration of the substance on the vessel
    pub fn get_concentration(&self, vessel: &O::VesselType, substance: &Substance) -> Result<SubstanceConcentration> {
        match self.vessel_map.get(vessel) {
            Some(store) => {
                Ok(store.concentration_of(substance))
            }
            None => {
                Err(anyhow!("Vessel '{}' is not attached", vessel))
            }
        }
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
    pub fn schedule_change(&mut self, vessel: O::VesselType, substance: Substance, amount: SubstanceConcentration, duration: SimTime) -> Result<IdType> {
        self.schedule_custom_change(vessel, substance, amount, SimTime::from_s(0.0), duration, BoundFn::Sigmoid)
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
        vessel: O::VesselType,
        substance: Substance,
        amount: SubstanceConcentration,
        delay: SimTime,
        duration: SimTime,
        bound_fn: BoundFn,
    ) -> Result<IdType> {
        match self.vessel_map.get_mut(&vessel) {
            Some(store) => {
                let store_change_id = store.schedule_change(substance, amount, delay, duration, bound_fn);
                let local_id = self.id_gen.get_id();
                self.change_map.insert(local_id, (vessel, substance, store_change_id));
                Ok(local_id)
            }
            None => {
                Err(anyhow!("Vessel '{}' is not attached", vessel))
            }
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
    ) -> Option<SubstanceChange> {
        match self.change_map.remove(change_id) {
            Some((vessel, substance, store_change_id)) => {
                match self.vessel_map.get_mut(&vessel) {
                    Some(store) => {
                        store.unschedule_change(&substance, &store_change_id)
                    }
                    None => None
                }
            }
            None => None
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
    use crate::sim::organism::test::{TestSim, TestBloodVessel};
    use crate::sim::SimTime;
    use crate::substance::{SubstanceStore, Substance};
    use crate::util::mmol_per_L;

    use super::ClosedCircConnector;

    #[test]
    fn test_sim_time() {
        let ccc = ClosedCircConnector::<TestSim>::new();
        assert_eq!(ccc.sim_time(), SimTime::from_s(0.0));
    }

    #[test]
    fn test_get_concentration() {
        let mut ccc = ClosedCircConnector::<TestSim>::new();
        ccc.vessel_map.insert(TestBloodVessel::VenaCava, SubstanceStore::new());
        assert!(ccc.get_concentration(&TestBloodVessel::VenaCava, &Substance::GLC).is_ok());
    }

    #[test]
    fn test_schedule_change() {
        let mut ccc = ClosedCircConnector::<TestSim>::new();
        ccc.vessel_map.insert(TestBloodVessel::VenaCava, SubstanceStore::new());
        assert!(ccc.schedule_change(TestBloodVessel::VenaCava, Substance::GLC, mmol_per_L!(1.0), SimTime::from_s(1.0)).is_ok());
    }

    #[test]
    fn test_bad_schedule_change() {
        let mut ccc = ClosedCircConnector::<TestSim>::new();
        ccc.vessel_map.insert(TestBloodVessel::VenaCava, SubstanceStore::new());
        assert!(ccc.schedule_change(TestBloodVessel::Aorta, Substance::GLC, mmol_per_L!(1.0), SimTime::from_s(1.0)).is_err());
    }

    #[test]
    fn test_schedule_custom_change() {
        let mut ccc = ClosedCircConnector::<TestSim>::new();
        ccc.vessel_map.insert(TestBloodVessel::VenaCava, SubstanceStore::new());
        assert!(ccc.schedule_custom_change(TestBloodVessel::VenaCava, Substance::GLC, mmol_per_L!(1.0), SimTime::from_s(1.0), SimTime::from_s(1.0), crate::util::BoundFn::Linear).is_ok());
    }

    #[test]
    fn test_bad_schedule_custom_change() {
        let mut ccc = ClosedCircConnector::<TestSim>::new();
        ccc.vessel_map.insert(TestBloodVessel::VenaCava, SubstanceStore::new());
        assert!(ccc.schedule_custom_change(TestBloodVessel::Aorta, Substance::GLC, mmol_per_L!(1.0), SimTime::from_s(1.0), SimTime::from_s(1.0), crate::util::BoundFn::Linear).is_err());
    }

    #[test]
    fn test_unschedule() {
        let mut ccc = ClosedCircConnector::<TestSim>::new();
        ccc.vessel_map.insert(TestBloodVessel::VenaCava, SubstanceStore::new());
        let id = ccc.schedule_change(TestBloodVessel::VenaCava, Substance::GLC, mmol_per_L!(1.0), SimTime::from_s(1.0)).unwrap();
        assert!(ccc.unschedule_change(&id).is_some());
    }

    #[test]
    fn test_bad_unschedule() {
        let mut ccc = ClosedCircConnector::<TestSim>::new();
        assert!(ccc.unschedule_change(&1).is_none());
    }

    #[test]
    fn test_unschedule_all() {
        let mut ccc = ClosedCircConnector::<TestSim>::new();
        assert!(ccc.unschedule_all);
        ccc.unschedule_all(false);
        assert!(!ccc.unschedule_all);
    }

}
