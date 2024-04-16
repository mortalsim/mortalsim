use either::Either;

use crate::sim::organism::Organism;
use crate::sim::SimTime;
use crate::substance::substance_wrapper::substance_store_wrapper;
use crate::substance::{Substance, SubstanceStore};
use crate::IdType;
use std::borrow::{Borrow, BorrowMut};
use std::cell::{RefCell, RefMut};
use std::collections::{hash_map, HashMap};
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Default)]
pub struct BloodStore {
    store: SubstanceStore,
    change_id_map: HashMap<Substance, Vec<IdType>>,
}

impl BloodStore {
    pub fn new() -> BloodStore {
        BloodStore {
            store: SubstanceStore::new(),
            change_id_map: HashMap::new(),
        }
    }

    pub fn build(store: SubstanceStore, change_id_map: HashMap<Substance, Vec<IdType>>) -> BloodStore {
        BloodStore { store, change_id_map }
    }

    pub(crate) fn extract(self) -> (SubstanceStore, HashMap<Substance, Vec<IdType>>) {
        (self.store, self.change_id_map)
    }

    pub(crate) fn advance(&mut self, sim_time: SimTime) {
        self.store.advance(sim_time)
    }

    substance_store_wrapper!(store, change_id_map);
}

pub struct CirculationConnector<O: Organism> {
    /// Mapping of `BloodVessel`s to their corresponding `SubstanceStore`
    pub(crate) vessel_map: HashMap<O::VesselType, RefCell<BloodStore>>,
    /// Mapping of `BloodVessel`s to their corresponding `SubstanceStore`
    pub(crate) vessel_map_sync: HashMap<O::VesselType, Arc<Mutex<BloodStore>>>,
    /// indicate whether the Arcs for sync have already been cloned
    pub(crate) synced: bool,
    /// Copy of the current simulation time
    pub(crate) sim_time: SimTime,
    /// Whether all changes should be unscheduled before each run
    /// NOTE: If this is set to false, the component is responsible for
    /// tracking and unscheduling preexisting changes, if necessary
    pub(crate) unschedule_all: bool,
}

impl<O: Organism> CirculationConnector<O> {
    pub fn new() -> CirculationConnector<O> {
        CirculationConnector {
            vessel_map: HashMap::new(),
            vessel_map_sync: HashMap::new(),
            synced: false,
            sim_time: SimTime::from_s(0.0),
            unschedule_all: true,
        }
    }

    /// Retrieves the blood store for the associated vessel
    /// Will panic if the vessel is already being
    /// borrowed by the current component
    pub fn blood_store(&self, vessel: &O::VesselType) -> Option<Either<RefMut<'_, BloodStore>, MutexGuard<'_, BloodStore>>> {
        if let Some(store) = self.vessel_map.get(vessel) {
            return Some(Either::Left(store.borrow_mut()));
        } else if let Some(store) = self.vessel_map_sync.get(vessel) {
            return Some(Either::Right(store.lock().unwrap()));
        }
        None
    }

    /// Retrieves an iterator of all existing blood stores which are connected
    /// to this component.
    pub fn with_blood_stores(&self, mut fcn: impl FnMut(O::VesselType, &mut BloodStore)) {
        if self.vessel_map_sync.is_empty() {
            for (v, s) in self.vessel_map.iter() {
                fcn(*v, &mut *s.borrow_mut())
            }
        } else {
            for (v, s) in self.vessel_map_sync.iter() {
                fcn(*v, &mut *s.lock().unwrap())
            }
        }
    }

    /// Retrieves the current simulation time
    pub fn sim_time(&self) -> SimTime {
        self.sim_time
    }

    /// Whether to unschedule all changes automatically before each run
    /// NOTE: If this is set to false, the component is responsible for
    /// tracking and unscheduling preexisting changes, if necessary
    pub fn unschedule_all(&mut self, value: bool) {
        self.unschedule_all = value
    }
}

#[cfg(test)]
pub mod test {

    use std::collections::HashMap;

    use crate::sim::layer::circulation::component::connector::BloodStore;
    use crate::sim::SimTime;
    use crate::substance::{Substance, SubstanceStore};
    use crate::{mmol_per_L, SimTimeSpan};
    use simple_si_units::chemical::Concentration;

    #[test]
    fn test_get_concentration() {
        let store = BloodStore {
            store: SubstanceStore::new(),
            change_id_map: HashMap::new(),
        };
        assert_eq!(
            store.concentration_of(&Substance::GLC),
            Concentration::from_M(0.0)
        );
    }

    #[test]
    fn test_schedule_change() {
        let mut store = BloodStore {
            store: SubstanceStore::new(),
            change_id_map: HashMap::new(),
        };
        store.schedule_change(Substance::GLC, mmol_per_L!(1.0), SimTimeSpan::from_s(1.0));
    }

    #[test]
    fn test_schedule_custom_change() {
        let mut store = BloodStore {
            store: SubstanceStore::new(),
            change_id_map: HashMap::new(),
        };
        store.schedule_custom_change(
            Substance::GLC,
            mmol_per_L!(1.0),
            SimTime::from_s(1.0),
            SimTimeSpan::from_s(1.0),
            crate::math::BoundFn::Linear,
        );
    }

    #[test]
    fn test_unschedule() {
        let mut store = BloodStore {
            store: SubstanceStore::new(),
            change_id_map: HashMap::new(),
        };
        let id = store.schedule_change(Substance::GLC, mmol_per_L!(1.0), SimTimeSpan::from_s(1.0));
        assert!(store.unschedule_change(&Substance::GLC, &id).is_some());
    }

    #[test]
    fn test_bad_unschedule() {
        let mut store = BloodStore {
            store: SubstanceStore::new(),
            change_id_map: HashMap::new(),
        };
        assert!(store.unschedule_change(&Substance::GLC, &1).is_none());
    }
}
