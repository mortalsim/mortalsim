use std::collections::HashMap;

use crate::sim::layer::SimLayer;
use crate::sim::organism::Organism;
use crate::sim::SimConnector;
use crate::sim::component::SimComponentProcessor;
use crate::substance::{Substance, SubstanceConcentration, SubstanceStore};
use crate::util::IdType;

use super::{
    CirculationInitializer, CirculationComponent, BloodStore
};

pub struct CirculationLayer<O: Organism> {
    blood_notify_map: HashMap<O::VesselType, HashMap<Substance, Vec<(SubstanceConcentration, &'static str)>>>,
    composition_map: HashMap<O::VesselType, SubstanceStore>,
    component_settings: HashMap<&'static str, CirculationInitializer<O>>,
    component_change_maps: HashMap<&'static str, HashMap<O::VesselType, HashMap<Substance, Vec<IdType>>>>,
}

impl<O: Organism + 'static> CirculationLayer<O> {
    /// Creates a CirculationLayer from a Graph representing the circulatory structure
    pub fn new() -> CirculationLayer<O> {
        CirculationLayer {
            blood_notify_map: HashMap::new(),
            composition_map: HashMap::new(),
            component_settings: HashMap::new(),
            component_change_maps: HashMap::new(),
        }
    }

    pub fn as_processor<T: CirculationComponent<O>>(&mut self) -> &mut dyn SimComponentProcessor<O, T> {
        self
    }

}

impl<O: Organism> SimLayer for CirculationLayer<O> {
    
    fn pre_exec(&mut self, connector: &mut SimConnector) {
        for (_, store) in self.composition_map.iter_mut() {
            store.advance(connector.sim_time());
        }
    }

    fn post_exec(&mut self, _connector: &mut SimConnector) {
        // Nohing to do here
    }
}

impl<O: Organism, T: CirculationComponent<O>> SimComponentProcessor<O, T> for CirculationLayer<O> {
    fn setup_component(&mut self, _connector: &mut SimConnector, component: &mut T) {
        let mut initializer = CirculationInitializer::new();
        component.circulation_init(&mut initializer);

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

    fn check_component(&mut self, component: &T) -> bool {
        let comp_settings = self.component_settings.get_mut(component.id()).unwrap();

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
        
        trigger
    }

    fn prepare_component(&mut self, connector: &mut SimConnector, component: &mut T) {
        let comp_id = component.id();
        let comp_settings = self.component_settings.get_mut(component.id()).unwrap();
        let circulation_connector = component.circulation_connector();
        circulation_connector.sim_time = connector.sim_time();

        if comp_settings.attach_all {
            for (vessel, store) in self.composition_map.drain() {
                let changes = self.component_change_maps.entry(comp_id).or_default().remove(&vessel).unwrap_or_default();
                circulation_connector.vessel_map.insert(vessel, BloodStore::build(store, changes));
            }
        }
        else {
            for vessel in comp_settings.vessel_connections.iter() {
                let store = self.composition_map.remove(&vessel).unwrap();
                let changes = self.component_change_maps.entry(comp_id).or_default().remove(&vessel).unwrap_or_default();
                circulation_connector.vessel_map.insert(*vessel, BloodStore::build(store, changes));
            }
        }
    }

    fn process_component(&mut self, _: &mut SimConnector, component: &mut T) {
        let comp_id = component.id();
        let circulation_connector = component.circulation_connector();
        for (vessel, blood_store) in circulation_connector.vessel_map.drain() {
            let (store, change_map) = blood_store.extract();
            self.composition_map.insert(vessel, store);
            self.component_change_maps.entry(comp_id).or_default().insert(vessel, change_map);
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::sim::{SimConnector, SimTime};
    use crate::sim::layer::SimLayer;
    use crate::sim::layer::circulation::{BloodStore, CirculationComponent};
    use crate::sim::layer::circulation::vessel::test::TestBloodVessel;
    use crate::sim::organism::test::TestSim;
    use crate::sim::layer::circulation::component::test::TestCircComponentA;
    use crate::sim::component::{SimComponentProcessor, SimComponent};
    use crate::substance::Substance;
    use crate::util::mmol_per_L;
    use super::CirculationLayer;

    #[test]
    fn test_layer() {
        CirculationLayer::<TestSim>::new();
    }

    #[test]
    fn test_layer_process() {
        let mut layer = CirculationLayer::<TestSim>::new();
        let mut component = TestCircComponentA::new();
        let mut connector = SimConnector::new();
        layer.setup_component(&mut connector, &mut component);

        component.circulation_connector().vessel_map.insert(TestBloodVessel::VenaCava, BloodStore::new());

        layer.prepare_component(&mut connector, &mut component);
        component.run();
        layer.process_component(&mut connector, &mut component);

        connector.time_manager.advance_by(SimTime::from_s(2.0));
        layer.pre_exec(&mut connector);

        let glc = layer.composition_map.get(&TestBloodVessel::VenaCava).unwrap().concentration_of(&Substance::GLC);
        let expected = mmol_per_L!(1.0);
        let threshold = mmol_per_L!(0.0001);
        assert!(glc > expected - threshold && glc < expected + threshold, "GLC not within {} of {}", threshold, expected);
        
        connector.time_manager.advance_by(SimTime::from_s(2.0));
        layer.pre_exec(&mut connector);

        let glc = layer.composition_map.get(&TestBloodVessel::VenaCava).unwrap().concentration_of(&Substance::GLC);
        let expected = mmol_per_L!(1.0);
        let threshold = mmol_per_L!(0.0001);
        assert!(glc > expected - threshold && glc < expected + threshold, "GLC not within {} of {}", threshold, expected);
    }
}
