use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::collections::hash_set;
use std::rc::Rc;
use std::string;
use std::any::{Any, TypeId};
use uuid::Uuid;
use petgraph::Direction;
use petgraph::graph::{Graph, NodeIndex, Neighbors};
use uom::si::molar_concentration::mole_per_liter;
use uom::si::ratio::{Ratio, ratio};
use uom::si::amount_of_substance::mole;
use crate::core::sim::{SimConnector, CoreSim, SimComponent, SimComponentInitializer};
use crate::core::sim::extension::SimExtension;
use crate::substance::{SubstanceStore, Volume, Substance, MolarConcentration, AmountOfSubstance};
use crate::event::{BloodCompositionChange, BloodVolumeChange};
use super::vessel::{BloodVessel, BloodVesselType, VesselIter};
use super::{BloodNode, BloodEdge, ClosedCirculatorySystem, ClosedCircVesselIter,
    ClosedCircConnector, ClosedCircSimComponent,
    ClosedCircInitializer, COMPONENT_REGISTRY};

pub struct ClosedCirculationSim<V: BloodVessel + 'static> {
    manager_id: Uuid,
    connector_map: HashMap<&'static str, (SimConnector, ClosedCircConnector<V>)>,
    active_components: HashMap<&'static str, Box<dyn ClosedCircSimComponent<VesselType = V>>>,
    system: Option<ClosedCirculatorySystem<V>>,
    blood_notify_map: HashMap<V, HashMap<Substance, Vec<(MolarConcentration, &'static str)>>>,
}

impl<V: BloodVessel + 'static> ClosedCirculationSim<V> {
    /// Creates a ClosedCirculationSim from a Graph representing the circulatory structure
    pub fn new(system: ClosedCirculatorySystem<V>) -> ClosedCirculationSim<V> {
        ClosedCirculationSim {
            manager_id: Uuid::new_v4(),
            connector_map: HashMap::new(),
            active_components: HashMap::new(),
            system: Some(system),
            blood_notify_map: HashMap::new(),
        }
    }

    fn get_system(&self) -> &ClosedCirculatorySystem<V> {
        match &self.system {
            None => panic!("ClosedCirculatorySystem was not reclaimed for ClosedCirculationSim!"),
            Some(v) => v
        }
    }
    
    fn get_system_mut(&mut self) -> &mut ClosedCirculatorySystem<V> {
        match &mut self.system {
            None => panic!("ClosedCirculatorySystem was not reclaimed for ClosedCirculationSim!"),
            Some(v) => v
        }
    }

    pub(crate) fn init_components(&mut self, component_names: HashSet<&'static str>, core: &mut CoreSim) -> HashSet<&'static str> {
        let mut registry = COMPONENT_REGISTRY.lock().unwrap();
        let vessel_registry: &mut HashMap<&'static str, Box<dyn Any + Send>> = registry.entry(TypeId::of::<V>()).or_insert(HashMap::new());

        let mut remaining_components = HashSet::new();

        // Initialize each component
        for component_name in component_names.into_iter() {
            match vessel_registry.get_mut(component_name) {
                None => {
                    remaining_components.insert(component_name);
                },
                Some(factory_box) => {
                    log::debug!("Initializing component \"{}\" on ClosedCirculation", component_name);
                    let factory = factory_box.downcast_mut::<Box<dyn FnMut() -> Box<dyn ClosedCircSimComponent<VesselType = V>>>>().unwrap();
                    let mut component = factory();
                    let mut initializer = SimComponentInitializer::new();
                    let mut cc_initializer = ClosedCircInitializer::new();
                    component.init(&mut initializer, &mut cc_initializer);
                    
                    self.active_components.insert(component_name, component);

                    // perform base core component portion setup
                    let connector = core.setup_component(component_name, initializer);

                    // perform closed circulation component portion setup
                    let cc_connector = self.setup_component(component_name, cc_initializer);
                    
                    self.connector_map.insert(component_name, (connector, cc_connector));
                }
            }
        }

        // Set up event notifications for this extension
        core.notify_extension::<BloodCompositionChange<V>>(self.manager_id);
        core.notify_extension::<BloodVolumeChange<V>>(self.manager_id);
        
        remaining_components
    }

    pub(crate) fn setup_component(&mut self, component_name: &'static str, cc_initializer: ClosedCircInitializer<V>) -> ClosedCircConnector<V> {

        let mut cc_connector = ClosedCircConnector::new(cc_initializer);

        for (vessel, substance_map) in cc_connector.substance_notifies.drain() {
            let mut substance_list = Vec::new();
            for (substance, threshold) in substance_map {
                substance_list.push(substance);
                let vsubstance_map = self.blood_notify_map.entry(vessel).or_insert(HashMap::new());
                let notify_list = vsubstance_map.entry(substance).or_insert(Vec::new());
                notify_list.push((threshold, component_name));
            }
        }
        cc_connector
    }

    pub(crate) fn update(&mut self, update_list: impl Iterator<Item = &'static str>, organism: &mut CoreSim) -> impl Iterator<Item = &'static str> {

        // Process any blood composition change events
        for evt in organism.extension_events::<BloodCompositionChange<V>>(&self.manager_id) {
            // Remove the index from the map so we have ownership of it
            let node_idx = self.get_system_mut().node_map.remove(&evt.vessel).unwrap();

            // Get the BloodNode out of the graph and update its concentration
            let node = &mut self.get_system_mut().graph[node_idx];
            let cur_amt = node.composition.concentration_of(&evt.substance).unwrap_or(MolarConcentration::new::<mole_per_liter>(0.0));
            node.composition.set_concentration(evt.substance, cur_amt + evt.change);

            // Insert the node index back into the map
            self.get_system_mut().node_map.insert(evt.vessel, node_idx);
        }

        // Process any blood volume change events
        for evt in organism.extension_events::<BloodVolumeChange<V>>(&self.manager_id) {
            // Remove the index from the map so we have ownership of it
            let node_idx = self.get_system_mut().node_map.remove(&evt.vessel).unwrap();

            // Get the BloodNode out of the graph and update its concentration
            let node = &mut self.get_system_mut().graph[node_idx];
            node.composition.set_volume(node.composition.volume() + evt.change);

            // Insert the node index back into the map
            self.get_system_mut().node_map.insert(evt.vessel, node_idx);
        }

        let mut remaining_components = Vec::new();
        for component_name in update_list {
            match self.active_components.remove(component_name) {
                None => {
                    remaining_components.push(component_name);
                },
                Some(mut component) => {
                    let (mut connector, mut cc_connector) = self.connector_map.remove(component_name).unwrap();

                    // Move the ClosedCirculationSystem into the connector for the component
                    cc_connector.system = self.system.take();

                    // Run the circulation component
                    component.run(&mut connector, &mut cc_connector);
                    
                    // Move the ClosedCirculationSystem back here
                    self.system = cc_connector.system.take();

                    // Process the base connector portion
                    organism.process_connector(&mut connector);

                    // Insert the connector and component back into their maps
                    self.connector_map.insert(component_name, (connector, cc_connector));
                    self.active_components.insert(component_name, component);
                }
            }
        }
        remaining_components.into_iter()
    }

    pub(crate) fn prepare_connector(&mut self, connector: &mut ClosedCircConnector<V>) {
        for vessel in connector.vessel_connections.iter() {
            let node_idx = self.get_system().node_map.get(&vessel).unwrap();
            // TODO: Cloning composition is relatively slow, but dealing with references
            // here is a pain, so we can optimize this later
            connector.stores.insert(*vessel, self.get_system().graph[*node_idx].composition.clone());
        }
    }

    pub(crate) fn process_connector(&mut self, _connector: &mut ClosedCircConnector<V>) {
        // ... Nothing to do here for now
    }

}

impl<V: BloodVessel + 'static> SimExtension for ClosedCirculationSim<V> {
    fn notify_events(&self) -> Vec<TypeId> {
        vec!(TypeId::of::<BloodCompositionChange<V>>(), TypeId::of::<BloodVolumeChange<V>>())
    }
    fn connectors(&mut self) -> Vec<(&'static str, &mut SimConnector)> {
        self.connector_map.iter_mut().map(|(n, (sc, _))| (*n, sc)).collect()
    }
}

// impl<V: BloodVessel + 'static> ClosedCircSimConnector<V> for ClosedCirculationSim<V> {
//     fn depth(&self) -> u32 {
//         self.depth as u32
//     }

//     fn blood_store(&self, vessel: &V) -> Option<&SubstanceStore> {
//         let node_idx = self.node_map.get(vessel)?;
//         Some(&self.graph[*node_idx].composition)
//     }

//     fn vessel_type(&self, vessel: V) -> BloodVesselType {
//         let node_idx = self.node_map.get(&vessel).unwrap();
//         self.graph[*node_idx].vessel_type
//     }

//     fn is_pre_capillary(&self, vessel: &V) -> bool {
//         self.pre_capillaries.contains(vessel)
//     }
    
//     fn is_post_capillary(&self, vessel: &V) -> bool {
//         self.post_capillaries.contains(vessel)
//     }

//     fn pre_capillaries(&self) -> VesselIter<V> {
//         self.pre_capillaries.iter().into()
//     }
    
//     fn post_capillaries(&self) -> VesselIter<V> {
//         self.post_capillaries.iter().into()
//     }

//     fn downstream(&self, vessel: V) -> ClosedCircVesselIter<V> {
//         self.vessel_connections(vessel, Direction::Outgoing)
//     }
    
//     fn upstream(&self, vessel: V) -> ClosedCircVesselIter<V> {
//         self.vessel_connections(vessel, Direction::Incoming)
//     }

//     fn connector(&mut self) -> &mut SimConnector {
//         self.connector.as_mut().unwrap()
//     }
// }

#[cfg(test)]
mod tests {
    use super::ClosedCirculationSim;

    #[test]
    fn test_manager() {

    }
}
