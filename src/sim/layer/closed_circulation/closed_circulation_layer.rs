use std::collections::HashMap;

use crate::sim::organism::Organism;
use crate::sim::{SimTime, SimConnector};
use crate::sim::component::SimComponentProcessor;
use crate::substance::{Substance, SubstanceConcentration, SubstanceStore};
use crate::util::IdType;

use super::{
    ClosedCircInitializer, ClosedCircComponent, BloodStore
};

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

    pub fn advance(&mut self, sim_time: SimTime) {
        for (_, store) in self.composition_map.iter_mut() {
            store.advance(sim_time);
        }
    }
}

impl<O: Organism, T: ClosedCircComponent<O>> SimComponentProcessor<O, T> for ClosedCirculationLayer<O> {

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

        self.component_settings.insert(component.id(), initializer);
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
    use crate::sim::{SimConnector, SimTime};
    use crate::sim::layer::closed_circulation::{BloodStore, ClosedCircComponent};
    use crate::sim::layer::closed_circulation::vessel::test::TestBloodVessel;
    use crate::sim::organism::test::TestSim;
    use crate::sim::layer::closed_circulation::component::test::TestCircComponentA;
    use crate::sim::component::{SimComponentProcessor, SimComponent};
    use crate::substance::Substance;
    use crate::util::mmol_per_L;
    use super::ClosedCirculationLayer;

    #[test]
    fn test_layer() {
        ClosedCirculationLayer::<TestSim>::new();
    }

    #[test]
    fn test_layer_process() {
        let mut layer = ClosedCirculationLayer::<TestSim>::new();
        let mut component = TestCircComponentA::new();
        let mut connector = SimConnector::new();
        layer.setup_component(&mut connector, &mut component);

        component.cc_connector().vessel_map.insert(TestBloodVessel::VenaCava, BloodStore::new());

        layer.prepare_component(&mut connector, &mut component);
        component.run();
        layer.process_component(&mut connector, &mut component);

        layer.advance(SimTime::from_s(2.0));

        let glc = layer.composition_map.get(&TestBloodVessel::VenaCava).unwrap().concentration_of(&Substance::GLC);
        let expected = mmol_per_L!(1.0);
        let threshold = mmol_per_L!(0.0001);
        assert!(glc > expected - threshold && glc < expected + threshold, "GLC not within {} of {}", threshold, expected);
        
        layer.advance(SimTime::from_s(2.0));

        let glc = layer.composition_map.get(&TestBloodVessel::VenaCava).unwrap().concentration_of(&Substance::GLC);
        let expected = mmol_per_L!(1.0);
        let threshold = mmol_per_L!(0.0001);
        assert!(glc > expected - threshold && glc < expected + threshold, "GLC not within {} of {}", threshold, expected);
    }
}
