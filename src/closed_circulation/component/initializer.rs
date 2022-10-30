use std::collections::{HashSet, HashMap};
use crate::substance::{Substance, MolarConcentration};
use crate::core::sim::SimComponentInitializer;
use crate::event::Event;
use super::super::vessel::BloodVessel;

pub struct ClosedCircInitializer<V: BloodVessel> {
    pub(crate) vessel_connections: HashSet<V>,
    pub(crate) substance_notifies: HashMap<V, HashMap<Substance, MolarConcentration>>,
    pub(crate) any_substance_notifies: HashSet<V>,
}

impl<V: BloodVessel> ClosedCircInitializer<V> {
    pub fn new() -> ClosedCircInitializer<V> {
        ClosedCircInitializer {
            vessel_connections: HashSet::new(),
            substance_notifies: HashMap::new(),
            any_substance_notifies: HashSet::new(),
        }
    }

    pub fn notify_composition_change(&mut self, vessel: V, substance: Substance, threshold: MolarConcentration) {
        self.vessel_connections.insert(vessel);
        let substance_map = self.substance_notifies.entry(vessel).or_insert(HashMap::new());
        substance_map.insert(substance, threshold);
    }
    
    pub fn notify_any_composition_change(&mut self, vessel: V) {
        self.vessel_connections.insert(vessel);
        self.any_substance_notifies.insert(vessel);
    }

    pub fn attach_vessel(&mut self, vessel: V) {
        self.vessel_connections.insert(vessel);
    }
}
