use std::collections::{HashSet, HashMap};
use crate::substance::{Substance, MolarConcentration, Volume, SubstanceStore};
use crate::core::sim::SimModuleInitializer;
use crate::event::Event;
use super::super::vessel::BloodVessel;

pub struct ClosedCircInitializer<V: BloodVessel> {
    pub(crate) vessel_connections: HashMap<V, SubstanceStore>,
    pub(crate) substance_notifies: HashMap<V, HashMap<Substance, MolarConcentration>>,
    pub(crate) any_substance_notifies: HashSet<V>,
    pub(crate) attach_all: bool,
}

impl<V: BloodVessel> ClosedCircInitializer<V> {
    pub fn new() -> ClosedCircInitializer<V> {
        ClosedCircInitializer {
            vessel_connections: HashMap::new(),
            substance_notifies: HashMap::new(),
            any_substance_notifies: HashSet::new(),
            attach_all: false,
        }
    }

    pub fn notify_composition_change(&mut self, vessel: V, substance: Substance, threshold: MolarConcentration) {
        self.vessel_connections.insert(vessel, SubstanceStore::new());
        let substance_map = self.substance_notifies.entry(vessel).or_insert(HashMap::new());
        substance_map.insert(substance, threshold);
    }
    
    pub fn attach_vessel(&mut self, vessel: V) {
        self.vessel_connections.insert(vessel, SubstanceStore::new());
    }
    
    pub fn manage_all_vessels(&mut self) {
        self.attach_all = true;
    }
}
