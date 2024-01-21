use crate::sim::organism::Organism;
use crate::sim::{SimTime, SimConnector};
use crate::sim::component::{SimComponent, SimComponentProcessor};
use crate::substance::{Substance, SubstanceConcentration, SubstanceStore};
use crate::util::IdType;

use super::vessel::{BloodVessel, BloodVesselType, VesselIter};
use super::{
    ClosedCircConnector, ClosedCircInitializer, ClosedCircComponent, BloodStore
};
use petgraph::graph::{Graph, Neighbors, NodeIndex};
use petgraph::Direction;
use simple_si_units::chemical::Concentration;
use std::any::{Any, TypeId};
use std::cmp::Ordering;
use std::collections::hash_set;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::mem::swap;
use std::rc::Rc;
use std::string;
use uuid::Uuid;

pub struct ClosedCirculationLayer<O: Organism + 'static> {
    blood_notify_map: HashMap<O::VesselType, HashMap<Substance, Vec<(SubstanceConcentration, &'static str)>>>,
    composition_map: HashMap<O::VesselType, SubstanceStore>,
    component_settings: HashMap<&'static str, ClosedCircInitializer<O>>,
    component_change_maps: HashMap<&'static str, HashMap<O::VesselType, HashMap<Substance, Vec<IdType>>>>,
}

impl<O: Organism + 'static> ClosedCirculationLayer<O> {
    /// Creates a ClosedCirculationLayer from a Graph representing the circulatory structure
    pub fn new() -> ClosedCirculationLayer<O> {
        ClosedCirculationLayer {
            blood_notify_map: HashMap::new(),
            composition_map: HashMap::new(),
            component_settings: HashMap::new(),
            component_change_maps: HashMap::new(),
        }
    }
}

impl<O: Organism, T: ClosedCircComponent<O>> SimComponentProcessor<O, T> for ClosedCirculationLayer<O> {
    fn advance(&mut self, sim_time: SimTime) {
        for (_, store) in self.composition_map.iter_mut() {
            store.advance(sim_time);
        }
    }

    fn setup_component(&mut self, _: &mut SimConnector, component: &mut T) {
        let mut initializer = ClosedCircInitializer::new();
        component.cc_init(&mut initializer);

        for (vessel, substance_map) in initializer.substance_notifies.drain() {
            let mut substance_list = Vec::new();
            for (substance, tracker) in substance_map {
                substance_list.push(substance);
                let vsubstance_map = self
                    .blood_notify_map
                    .entry(vessel)
                    .or_insert(HashMap::new());
                let notify_list = vsubstance_map.entry(substance).or_insert(Vec::new());
                notify_list.push((tracker.threshold, component.id()));
            }
        }
    }

    fn prepare_component(&mut self, connector: &SimConnector, component: &mut T) -> bool {
        let comp_id = component.id();
        let comp_settings = self.component_settings.get_mut(component.id()).unwrap();
        let cc_connector = component.cc_connector();
        cc_connector.sim_time = connector.sim_time;
        let mut trigger = false;

        // Determine if any substances have changed beyond the threshold
        for (vessel, track_map) in comp_settings.substance_notifies.iter_mut() {
            for (substance, tracker) in track_map.iter_mut() {
                let val = self.composition_map.get(vessel).unwrap().concentration_of(substance);
                if tracker.check(val) {
                    trigger = true;
                    tracker.update(val)
                }
            }
        }
        if trigger {
            if comp_settings.attach_all {
                for (vessel, store) in self.composition_map.drain() {
                    let changes = self.component_change_maps.entry(comp_id).or_default().remove(&vessel).unwrap_or_default();
                    cc_connector.vessel_map.insert(vessel, BloodStore::build(store, changes));
                }
            }
            else {
                for vessel in comp_settings.vessel_connections.iter() {
                    let store = self.composition_map.remove(&vessel).unwrap();
                    let changes = self.component_change_maps.entry(comp_id).or_default().remove(&vessel).unwrap_or_default();
                    cc_connector.vessel_map.insert(*vessel, BloodStore::build(store, changes));
                }
            }
        }
        trigger
    }

    fn process_component(&mut self, _: &mut SimConnector, component: &mut T) {
        let comp_id = component.id();
        let cc_connector = component.cc_connector();
        for (vessel, blood_store) in cc_connector.vessel_map.drain() {
            let (store, change_map) = blood_store.extract();
            self.composition_map.insert(vessel, store);
            self.component_change_maps.entry(comp_id).or_default().insert(vessel, change_map);
        }
    }
}



#[cfg(test)]
mod tests {
    use crate::sim::SimConnector;
    use crate::sim::organism::test::{TestSim, TestBloodVessel};
    use crate::sim::layer::closed_circulation::component::test::TestCircComponentA;
    use crate::sim::component::SimComponentProcessor;
    use super::ClosedCirculationLayer;

    #[test]
    fn test_layer() {
        ClosedCirculationLayer::<TestSim>::new();
    }

    #[test]
    fn test_setup() {
        let mut layer = ClosedCirculationLayer::<TestSim>::new();
        let mut component_a = TestCircComponentA::new();
        let mut connector = SimConnector::new();
        layer.setup_component(&mut connector, &mut component_a);
    }
}
