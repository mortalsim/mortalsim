use std::collections::{HashSet, HashMap};
use crate::substance::{Substance, MolarConcentration};
use crate::core::sim::SimComponentInitializer;
use crate::event::Event;
use super::super::super::BloodVessel;

pub struct ClosedCircInitializer<V: BloodVessel> {
    vessel_connections: HashSet<V>,
    substance_notifies: HashMap<V, HashMap<Substance, MolarConcentration>>
}

impl<V: BloodVessel> ClosedCircInitializer<V> {
    pub fn new() -> ClosedCircInitializer<V> {
        ClosedCircInitializer {
            vessel_connections: HashSet::new(),
            substance_notifies: HashMap::new(),
        }
    }

    pub fn notify_composition_change(&mut self, vessel: V, substance: Substance, threshold: MolarConcentration) {
        self.vessel_connections.insert(vessel);
        let substance_map = self.substance_notifies.entry(vessel).or_insert(HashMap::new());
        substance_map.insert(substance, threshold);
    }

    pub fn attach_vessel(&mut self, vessel: V) {
        self.vessel_connections.insert(vessel);
    }
}

pub struct ClosedCircComponentInitializer<V: BloodVessel> {
    init: SimComponentInitializer,
    cc_init: ClosedCircInitializer<V>,
}

impl<V: BloodVessel> ClosedCircComponentInitializer<V> {
    pub fn new() -> ClosedCircComponentInitializer<V> {
        ClosedCircComponentInitializer {
            init: SimComponentInitializer::new(),
            cc_init: ClosedCircInitializer::new(),
        }
    }
}