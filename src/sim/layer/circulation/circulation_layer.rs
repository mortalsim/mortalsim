use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::swap;
use std::sync::{Arc, Mutex};

use crate::sim::component::{SimComponentProcessor, SimComponentProcessorSync};
use crate::sim::layer::{SimLayer, SimLayerSync};
use crate::sim::organism::Organism;
use crate::sim::SimConnector;
use crate::substance::{Substance, SubstanceConcentration, SubstanceStore};
use crate::util::IdType;

use super::{vessel, BloodStore, CirculationComponent, CirculationInitializer};

pub struct CirculationLayer<O: Organism> {
    blood_notify_map:
        HashMap<O::VesselType, HashMap<Substance, Vec<(SubstanceConcentration, &'static str)>>>,
    composition_map: HashMap<O::VesselType, RefCell<BloodStore>>,
    composition_map_sync: HashMap<O::VesselType, Arc<Mutex<BloodStore>>>,
    component_settings: HashMap<&'static str, CirculationInitializer<O>>,
}

impl<O: Organism> CirculationLayer<O> {
    /// Creates a CirculationLayer from a Graph representing the circulatory structure
    pub fn new() -> CirculationLayer<O> {
        CirculationLayer {
            blood_notify_map: HashMap::new(),
            composition_map: HashMap::new(),
            composition_map_sync: HashMap::new(),
            component_settings: HashMap::new(),
        }
    }
}

impl<O: Organism> SimLayer for CirculationLayer<O> {
    fn pre_exec(&mut self, connector: &mut SimConnector) {
        for (_, store) in self.composition_map.iter() {
            store.borrow_mut().advance(connector.sim_time());
        }
        for (_, store) in self.composition_map_sync.iter() {
            store.lock().unwrap().advance(connector.sim_time());
        }
    }

    fn post_exec(&mut self, _connector: &mut SimConnector) {
        // Nothing to do here
    }
}

impl<O:Organism> SimLayerSync for CirculationLayer<O> {
    fn pre_exec_sync(&mut self, connector: &mut SimConnector) {
        self.pre_exec(connector)
    }

    fn post_exec_sync(&mut self, _connector: &mut SimConnector) {
        // Nothing to do here
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

        println!("adding circ component: {}", component.id());
        self.component_settings.insert(component.id(), initializer);
    }

    fn check_component(&mut self, component: &T) -> bool {
        println!("checking circ component: {}", component.id());
        let comp_settings = self.component_settings.get_mut(component.id()).unwrap();

        let mut trigger = false;

        // Determine if any substances have changed beyond the threshold
        for (vessel, track_map) in comp_settings.substance_notifies.iter_mut() {
            for (substance, tracker) in track_map.iter_mut() {
                let val = self
                    .composition_map
                    .get(vessel)
                    .unwrap()
                    .borrow()
                    .concentration_of(substance);
                if tracker.check(val) {
                    trigger = true;
                    tracker.update(val)
                }
            }
        }

        trigger
    }

    fn prepare_component(&mut self, connector: &mut SimConnector, component: &mut T) {
        let comp_settings = self.component_settings.get_mut(component.id()).unwrap();
        let circulation_connector = component.circulation_connector();
        circulation_connector.sim_time = connector.sim_time();

        if comp_settings.attach_all {
            swap(&mut self.composition_map, &mut circulation_connector.vessel_map);
        } else {
            for vessel in comp_settings.vessel_connections.iter() {
                let store = self.composition_map.remove(vessel).unwrap_or_default();
                circulation_connector
                    .vessel_map
                    .insert(*vessel, store);
            }
        }
    }

    fn process_component(&mut self, _: &mut SimConnector, component: &mut T) {
        let comp_settings = self.component_settings.get(component.id()).unwrap();
        let circulation_connector = component.circulation_connector();

        if comp_settings.attach_all {
            swap(&mut self.composition_map, &mut circulation_connector.vessel_map);
        } else {
            // move stores back from the component
            for vessel in comp_settings.vessel_connections.iter() {
                let store = circulation_connector.vessel_map.remove(vessel).unwrap_or_default();
                self.composition_map.insert(*vessel, store);
            }
        }
    }
}

impl<O: Organism, T: CirculationComponent<O>> SimComponentProcessorSync<O, T> for CirculationLayer<O> {
    fn setup_component_sync(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.setup_component(connector, component);

        // Copy all relevant Arcs to the component's connector
        let comp_id = component.id();
        let comp_settings = self.component_settings.get(component.id()).unwrap();
        let circulation_connector = component.circulation_connector();
        circulation_connector.sim_time = connector.sim_time();

        if comp_settings.attach_all {
            // Clone all of the Arcs into the component's map
            circulation_connector.vessel_map_sync = self.composition_map_sync.clone();
        } else {
            for vessel in self.component_settings.get(comp_id).unwrap().vessel_connections.iter() {

                let store = self.composition_map_sync.entry(*vessel).or_default();
                circulation_connector
                    .vessel_map_sync
                    .entry(*vessel)
                    .or_insert(store.clone());

            }
        }
    }

    fn check_component_sync(&mut self, component: &T) -> bool {
        let comp_settings = self.component_settings.get_mut(component.id()).unwrap();

        let mut trigger = false;

        // Determine if any substances have changed beyond the threshold
        for (vessel, track_map) in comp_settings.substance_notifies.iter_mut() {
            for (substance, tracker) in track_map.iter_mut() {
                let val = self
                    .composition_map_sync
                    .entry(*vessel)
                    .or_default()
                    .lock()
                    .unwrap()
                    .concentration_of(substance);
                if tracker.check(val) {
                    trigger = true;
                    tracker.update(val)
                }
            }
        }

        trigger
    }

    fn prepare_component_sync(&mut self, _connector: &mut SimConnector, _component: &mut T) {
        // Nothing to do here. Everything is done directly on blood store objects
        // which are already shared via Arc & Mutex.
    }

    fn process_component_sync(&mut self, _connector: &mut SimConnector, _component: &mut T) {
        // Nothing to do here. Everything is done directly on blood store objects
        // which are already shared via Arc & Mutex.
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::sync::{Arc, Mutex};
    use std::thread::scope;

    use super::CirculationLayer;
    use crate::sim::component::{SimComponent, SimComponentProcessor, SimComponentProcessorSync};
    use crate::sim::layer::circulation::component::test::TestCircComponentA;
    use crate::sim::layer::circulation::{BloodStore, CirculationComponent};
    use crate::sim::layer::{SimLayer, SimLayerSync};
    use crate::sim::organism::test::{TestBloodVessel, TestOrganism, TestSim};
    use crate::sim::{SimConnector, SimTime};
    use crate::substance::Substance;
    use crate::util::mmol_per_L;

    #[test]
    fn layer() {
        CirculationLayer::<TestOrganism>::new();
    }

    #[test]
    fn layer_process() {
        let mut layer = CirculationLayer::<TestOrganism>::new();
        let mut component = TestCircComponentA::new();
        let mut connector = SimConnector::new();
        layer.setup_component(&mut connector, &mut component);

        component
            .circulation_connector()
            .vessel_map
            .insert(TestBloodVessel::VenaCava, RefCell::new(BloodStore::new()));

        layer.prepare_component(&mut connector, &mut component);
        component.run();
        layer.process_component(&mut connector, &mut component);

        connector.time_manager.advance_by(SimTime::from_s(2.0));
        layer.pre_exec(&mut connector);

        let glc = layer
            .composition_map
            .get(&TestBloodVessel::VenaCava)
            .unwrap()
            .borrow()
            .concentration_of(&Substance::GLC);
        let expected = mmol_per_L!(1.0);
        let threshold = mmol_per_L!(0.0001);
        assert!(
            glc > expected - threshold && glc < expected + threshold,
            "GLC not within {} of {}",
            threshold,
            expected
        );

        connector.time_manager.advance_by(SimTime::from_s(2.0));
        layer.pre_exec(&mut connector);

        let glc = layer
            .composition_map
            .get(&TestBloodVessel::VenaCava)
            .unwrap()
            .borrow()
            .concentration_of(&Substance::GLC);
        let expected = mmol_per_L!(1.0);
        let threshold = mmol_per_L!(0.0001);
        assert!(
            glc > expected - threshold && glc < expected + threshold,
            "GLC not within {} of {}",
            threshold,
            expected
        );
    }

    #[test]
    fn layer_process_sync() {
        let layer = Mutex::new(CirculationLayer::<TestOrganism>::new());
        let connector = Mutex::new(SimConnector::new());
        let mut component = TestCircComponentA::new();
        layer.lock().unwrap().setup_component_sync(&mut connector.lock().unwrap(), &mut component);

        component
            .circulation_connector()
            .vessel_map_sync
            .insert(TestBloodVessel::VenaCava, Arc::new(Mutex::new(BloodStore::new())));

        layer.lock().unwrap().pre_exec_sync(&mut connector.lock().unwrap());

        scope(|s| {
            s.spawn(|| {
                layer.lock().unwrap().prepare_component_sync(&mut connector.lock().unwrap(), &mut component);
                component.run();
                layer.lock().unwrap().process_component_sync(&mut connector.lock().unwrap(), &mut component);
            });
        });

        layer.lock().unwrap().post_exec_sync(&mut connector.lock().unwrap());
    }
}
