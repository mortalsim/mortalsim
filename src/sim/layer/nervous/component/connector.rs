use super::super::vessel::{BloodVessel, BloodVesselType, VesselIter};
use crate::sim::SimTime;
use crate::sim::organism::Organism;
use crate::substance::substance_wrapper::substance_store_wrapper;
use crate::substance::{SubstanceConcentration, Substance, SubstanceStore, SubstanceChange, ConcentrationTracker};
use crate::util::{BoundFn, IdType, IdGenerator};
use super::ClosedCircInitializer;
use anyhow::{Result, Error};
use petgraph::Direction;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct BloodStore {
    store: SubstanceStore,
    change_map: HashMap<Substance, Vec<IdType>>,
}

impl BloodStore {
    pub fn new() -> BloodStore {
        BloodStore {
            store: SubstanceStore::new(),
            change_map: HashMap::new(),
        }
    }

    pub fn build(store: SubstanceStore, change_map: HashMap<Substance, Vec<IdType>>) -> BloodStore {
        BloodStore {
            store,
            change_map,
        }
    }

    pub(crate) fn extract(self) -> (SubstanceStore, HashMap<Substance, Vec<IdType>>) {
        (self.store, self.change_map)
    }

    pub(crate) fn advance(&mut self, sim_time: SimTime) {
        self.store.advance(sim_time)
    }

    substance_store_wrapper!(store, change_map);
}

pub struct ClosedCircConnector<O: Organism> {
    /// Mapping of `BloodVessel`s to their corresponding `SubstanceStore`
    pub(crate) vessel_map: HashMap<O::VesselType, BloodStore>,
    /// Copy of the current simulation time
    pub(crate) sim_time: SimTime,
    /// Whether all changes should be unscheduled before each run
    /// NOTE: If this is set to false, the component is responsible for
    /// tracking and unscheduling preexisting changes, if necessary
    pub(crate) unschedule_all: bool,
}

impl<O: Organism> ClosedCircConnector<O> {
    pub fn new() -> ClosedCircConnector<O> {
        ClosedCircConnector {
            vessel_map: HashMap::new(),
            sim_time: SimTime::from_s(0.0),
            unschedule_all: true,
        }
    }

    /// Retrieves the blood store for the associated vessel
    pub fn blood_store(&mut self, vessel: &O::VesselType) -> Option<&mut BloodStore> {
        self.vessel_map.get_mut(vessel)
    }

    /// Retrieves the current simulation time
    pub fn sim_time(&self) -> SimTime {
        self.sim_time
    }
    
    /// Retrieves the current simulation time
    pub fn unschedule_all(&mut self, value: bool) {
        self.unschedule_all = value
    }
}

#[cfg(test)]
pub mod test {

    use std::collections::HashMap;

    use simple_si_units::chemical::Concentration;
    use crate::sim::layer::closed_circulation::component::connector::BloodStore;
    use crate::sim::organism::test::{TestSim, TestBloodVessel};
    use crate::sim::SimTime;
    use crate::substance::{SubstanceStore, Substance};
    use crate::util::mmol_per_L;

    use super::ClosedCircConnector;

    #[test]
    fn test_get_concentration() {
        let store = BloodStore {
            store: SubstanceStore::new(),
            change_map: HashMap::new(),
        };
        assert_eq!(store.concentration_of(&Substance::GLC), Concentration::from_M(0.0));
    }

    #[test]
    fn test_schedule_change() {
        let mut store = BloodStore {
            store: SubstanceStore::new(),
            change_map: HashMap::new(),
        };
        store.schedule_change(Substance::GLC, mmol_per_L!(1.0), SimTime::from_s(1.0));
    }

    #[test]
    fn test_schedule_custom_change() {
        let mut store = BloodStore {
            store: SubstanceStore::new(),
            change_map: HashMap::new(),
        };
        store.schedule_custom_change(Substance::GLC, mmol_per_L!(1.0), SimTime::from_s(1.0), SimTime::from_s(1.0), crate::util::BoundFn::Linear);
    }

    #[test]
    fn test_unschedule() {
        let mut store = BloodStore {
            store: SubstanceStore::new(),
            change_map: HashMap::new(),
        };
        let id = store.schedule_change(Substance::GLC, mmol_per_L!(1.0), SimTime::from_s(1.0));
        assert!(store.unschedule_change(&Substance::GLC, &id).is_some());
    }

    #[test]
    fn test_bad_unschedule() {
        let mut store = BloodStore {
            store: SubstanceStore::new(),
            change_map: HashMap::new(),
        };
        assert!(store.unschedule_change(&Substance::GLC, &1).is_none());
    }

}
