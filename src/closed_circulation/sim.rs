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
use uom::si::amount_of_substance::mole;
use crate::core::sim::{SimConnector, CoreSim, SimComponent, SimComponentInitializer};
use crate::substance::{SubstanceStore, Volume, Substance, MolarConcentration, AmountOfSubstance};
use crate::event::{BloodCompositionChange, BloodVolumeChange};
use super::vessel::{BloodVessel, BloodVesselType, VesselIter};
use super::{BloodNode, BloodEdge, ClosedCirculatorySystem, ClosedCircVesselIter,
    ClosedCircConnector, ClosedCircSimComponent,
    ClosedCircInitializer, COMPONENT_REGISTRY};

pub struct ClosedCirculationSim<V: BloodVessel + 'static> {
    manager_id: Uuid,
    active_components: HashMap<&'static str, Box<dyn ClosedCircSimComponent<VesselType = V>>>,
    system: Rc<ClosedCirculatorySystem<V>>,
    blood_notify_map: HashMap<V, HashMap<Substance, Vec<(MolarConcentration, &'static str)>>>,
    composition_map: HashMap<V, SubstanceStore>,
}

impl<V: BloodVessel + 'static> ClosedCirculationSim<V> {
    /// Creates a ClosedCirculationSim from a Graph representing the circulatory structure
    pub fn new(system: ClosedCirculatorySystem<V>) -> ClosedCirculationSim<V> {
        ClosedCirculationSim {
            manager_id: Uuid::new_v4(),
            active_components: HashMap::new(),
            system: Rc::new(system),
            blood_notify_map: HashMap::new(),
            composition_map: HashMap::new(),
        }
    }

    fn get_system(&self) -> &ClosedCirculatorySystem<V> {
        self.system.as_ref()
    }

    pub(crate) fn init_components(&mut self, component_names: HashSet<&'static str>) {
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
                    let mut cc_initializer = ClosedCircInitializer::new();
                    component.init_cc(&mut cc_initializer);
                    
                    // perform closed circulation component portion setup
                    self.setup_component(component_name, cc_initializer);
                    
                    self.active_components.insert(component_name, component);
                }
            }
        }
    }

    pub(crate) fn setup_component(&mut self, component_name: &'static str, cc_initializer: ClosedCircInitializer<V>) -> ClosedCircConnector<V> {

        let mut cc_connector = ClosedCircConnector::new(self.system.clone(), cc_initializer);

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

    pub(crate) fn update(&mut self, update_list: impl Iterator<Item = &'static str>) {

        // // Process any blood composition change events
        // for evt in organism.extension_events::<BloodCompositionChange<V>>(&self.manager_id) {
        //     // Remove the index from the map so we have ownership of it
        //     let node_idx = self.get_system_mut().node_map.remove(&evt.vessel).unwrap();

        //     // Get the BloodNode out of the graph and update its concentration
        //     let node = &mut self.get_system_mut().graph[node_idx];
        //     let cur_amt = node.composition.concentration_of(&evt.substance).unwrap_or(MolarConcentration::new::<mole_per_liter>(0.0));
        //     node.composition.set_concentration(evt.substance, cur_amt + evt.change);

        //     // Insert the node index back into the map
        //     self.get_system_mut().node_map.insert(evt.vessel, node_idx);
        // }

        // // Process any blood volume change events
        // for evt in organism.extension_events::<BloodVolumeChange<V>>(&self.manager_id) {
        //     // Remove the index from the map so we have ownership of it
        //     let node_idx = self.get_system_mut().node_map.remove(&evt.vessel).unwrap();

        //     // Get the BloodNode out of the graph and update its concentration
        //     let node = &mut self.get_system_mut().graph[node_idx];
        //     node.composition.set_volume(node.composition.volume() + evt.change);

        //     // Insert the node index back into the map
        //     self.get_system_mut().node_map.insert(evt.vessel, node_idx);
        // }

        for component_name in update_list {
            let component = self.active_components.remove(component_name).unwrap();

            // Insert the component back into its map
            self.active_components.insert(component_name, component);
        }
    }

    pub(crate) fn prepare_component(&mut self, component: &mut dyn ClosedCircSimComponent<VesselType = V>) {
        let cc_connector = component.get_cc_sim_connector();

        for (vessel, store) in cc_connector.vessel_connections.iter() {
            store.merge_all(self.composition_map.get(vessel).unwrap());
        }
    }

    pub(crate) fn process_component(&mut self, _connector: &mut ClosedCircConnector<V>) {
        // ... Nothing to do here for now
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
