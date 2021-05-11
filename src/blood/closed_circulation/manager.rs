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
use crate::core::sim::{SimConnector, Organism, SimComponent, SimComponentInitializer};
use crate::core::sim::extension::SimExtension;
use crate::substance::{SubstanceStore, Volume, Substance, MolarConcentration, AmountOfSubstance};
use crate::event::{BloodCompositionChange, BloodVolumeChange};
use super::super::{BloodVessel, BloodVesselType, VesselIter};
use super::{BloodNode, BloodEdge, ClosedCirculatorySystem, ClosedCircVesselIter,
    ClosedCircConnector, ClosedCircSimConnector, ClosedCircSimComponent,
    ClosedCircComponentInitializer, ClosedCircInitializer, COMPONENT_REGISTRY};

struct ComponentContext<V: BloodVessel> {
    connected_vessels: HashSet<V>,
    substance_notify_map: HashMap<V, Vec<Substance>>,
}

pub struct ClosedCirculationManager<V: BloodVessel> {
    manager_id: Uuid,
    graph: Graph<BloodNode<V>, BloodEdge>,
    node_map: HashMap<V, NodeIndex>,
    pre_capillaries: HashSet<V>,
    post_capillaries: HashSet<V>,
    connector: Option<SimConnector>,
    component_context_map: HashMap<&'static str, ComponentContext<V>>,
    connector_map: HashMap<&'static str, SimConnector>,
    blood_notify_map: HashMap<V, HashMap<Substance, Vec<(MolarConcentration, &'static str)>>>,
    active_components: HashMap<&'static str, Box<dyn ClosedCircSimComponent<VesselType = V>>>,
    depth: u8,
}

impl<V: BloodVessel + 'static> ClosedCirculationManager<V> {
    /// Creates a ClosedCirculationManager from a Graph representing the circulatory structure
    pub fn new(circulation: ClosedCirculatorySystem<V>) -> ClosedCirculationManager<V> {
        ClosedCirculationManager {
            manager_id: Uuid::new_v4(),
            graph: circulation.graph,
            node_map: circulation.node_map,
            pre_capillaries: circulation.pre_capillaries,
            post_capillaries: circulation.post_capillaries,
            connector: None,
            component_context_map: HashMap::new(),
            connector_map: HashMap::new(),
            blood_notify_map: HashMap::new(),
            active_components: HashMap::new(),
            depth: circulation.depth,
        }
    }

    fn init_components(&mut self, component_names: HashSet<&'static str>, mut organism: Organism) -> HashSet<&'static str> {
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
                    let mut ccc_initializer = ClosedCircComponentInitializer::new();
                    component.init(&mut ccc_initializer);
                    
                    self.active_components.insert(component_name, component);

                    let mut connector = SimConnector::new();

                    // perform base organism component portion setup
                    organism.setup_component(component_name, ccc_initializer.initializer, &mut connector);
                    self.connector_map.insert(component_name, connector);

                    // perform closed circulation component portion setup
                    self.setup_component(component_name, ccc_initializer.cc_initializer);
                }
            }
        }

        // Set up event notifications for this extension
        organism.notify_extension::<BloodCompositionChange<V>>(self.manager_id);
        organism.notify_extension::<BloodVolumeChange<V>>(self.manager_id);
        
        remaining_components
    }

    fn setup_component(&mut self, component_name: &'static str, initializer: ClosedCircInitializer<V>) {
        let vessel_connections = initializer.vessel_connections;
        let mut component_vessel_map = HashMap::new();
        for (vessel, substance_map) in initializer.substance_notifies {
            let mut substance_list = Vec::new();
            for (substance, threshold) in substance_map {
                substance_list.push(substance);
                let vsubstance_map = self.blood_notify_map.entry(vessel).or_insert(HashMap::new());
                let notify_list = vsubstance_map.entry(substance).or_insert(Vec::new());
                notify_list.push((threshold, component_name));
            }
            component_vessel_map.insert(vessel, substance_list);
        }

        self.component_context_map.insert(component_name, ComponentContext {
            connected_vessels: vessel_connections,
            substance_notify_map: component_vessel_map,
        });
    }

    pub(crate) fn update(&mut self, update_list: impl Iterator<Item = &'static str>, organism: &mut Organism) -> impl Iterator<Item = &'static str> {

        // Process any blood composition change events
        for evt in organism.extension_events::<BloodCompositionChange<V>>(&self.manager_id) {
            // Remove the index from the map so we have ownership of it
            let node_idx = self.node_map.remove(&evt.vessel).unwrap();

            // Get the BloodNode out of the graph and update its concentration
            let node = &mut self.graph[node_idx];
            let cur_amt = node.composition.concentration_of(&evt.substance).unwrap_or(MolarConcentration::new::<mole_per_liter>(0.0));
            node.composition.set_concentration(evt.substance, cur_amt + evt.change);

            // Insert the node index back into the map
            self.node_map.insert(evt.vessel, node_idx);
        }

        // Process any blood volume change events
        for evt in organism.extension_events::<BloodVolumeChange<V>>(&self.manager_id) {
            // Remove the index from the map so we have ownership of it
            let node_idx = self.node_map.remove(&evt.vessel).unwrap();

            // Get the BloodNode out of the graph and update its concentration
            let node = &mut self.graph[node_idx];
            node.composition.set_volume(node.composition.volume() + evt.change);

            // Insert the node index back into the map
            self.node_map.insert(evt.vessel, node_idx);
        }

        let mut remaining_components = Vec::new();
        for component_name in update_list {
            match self.active_components.remove(component_name) {
                None => {
                    remaining_components.push(component_name);
                },
                Some(mut component) => {
                    let mut connector = self.connector_map.remove(component_name).unwrap();

                    // Run the circulation component
                    self.set_active_connector(connector);
                    component.run(self);
                    connector = self.take_active_connector().unwrap();

                    // Process the base connector portion
                    organism.process_connector(&mut connector);

                    // Insert the connector and component back into their maps
                    self.connector_map.insert(component_name, connector);
                    self.active_components.insert(component_name, component);
                }
            }
        }
        remaining_components.into_iter()
    }

    pub(crate) fn set_active_connector(&mut self, connector: SimConnector) {
        // Update connector before component execution
        self.connector = Some(connector)
    }

    pub(crate) fn take_active_connector(&mut self) -> Option<SimConnector> {
        self.connector.take()
    }

    /// Internal function for retrieving a vessel connection iterator
    /// (upstream or downstream)
    fn vessel_connections(&self, vessel: V, dir: Direction) -> ClosedCircVesselIter<V> {
        match self.node_map.get(&vessel) {
            Some(node_idx) => {
                ClosedCircVesselIter {
                    graph: &self.graph,
                    idx_iter: Some(self.graph.neighbors_directed(node_idx.clone(), dir))
                }
            },
            None => {
                ClosedCircVesselIter {
                    graph: &self.graph,
                    idx_iter: None
                }
            }
        }
    }
}

impl<V: BloodVessel + 'static> SimExtension for ClosedCirculationManager<V> {
    fn notify_events(&self) -> Vec<TypeId> {
        vec!(TypeId::of::<BloodCompositionChange<V>>(), TypeId::of::<BloodVolumeChange<V>>())
    }
    fn connectors(&mut self) -> Vec<(&'static str, &mut SimConnector)> {
        self.connector_map.iter_mut().map(|x| (*x.0, x.1)).collect()
    }
}


// impl<V: BloodVessel + 'static> SimComponent for ClosedCirculationManager<V> 
// where <V as std::str::FromStr>::Err: std::fmt::Debug {
//     fn init(&mut self, initializer: &mut SimComponentInitializer) {
//         // Insert dummy defaults which we'll never use
//         initializer.notify_prioritized(100, BloodCompositionChange {
//             vessel: V::source(),
//             substance: Substance::H2O,
//             previous_value: MolarConcentration::new::<mole_per_liter>(0.0),
//             new_value: MolarConcentration::new::<mole_per_liter>(0.0),
//         });
//         initializer.notify_prioritized(100, BloodVolumeRatioChange {
//             vessel: V::source(),
//             previous_value: 0.0,
//             new_value: 0.0,
//         });
//     }

//     fn run(&mut self, connector: &mut SimConnector) {
//         for trigger in connector.trigger_events() {
//             match trigger.clone().downcast_arc::<BloodCompositionChange<V>>() {
//                 Ok(change_evt) => {

//                 }
//             }
//         }
//     }
// }

impl<V: BloodVessel> ClosedCircConnector<V> for ClosedCirculationManager<V> {
    fn composition(&self, vessel: V) -> &SubstanceStore {
        let node_idx = self.node_map.get(&vessel).unwrap();
        &self.graph[*node_idx].composition
    }

    fn composition_mut(&mut self, vessel: V) -> &mut SubstanceStore {
        let node_idx = self.node_map.get(&vessel).unwrap();
        &mut self.graph[*node_idx].composition
    }
}

impl<V: BloodVessel + 'static> ClosedCircSimConnector<V> for ClosedCirculationManager<V> {
    fn depth(&self) -> u32 {
        self.depth as u32
    }

    fn vessel_type(&self, vessel: V) -> BloodVesselType {
        let node_idx = self.node_map.get(&vessel).unwrap();
        self.graph[*node_idx].vessel_type
    }

    fn is_pre_capillary(&self, vessel: &V) -> bool {
        self.pre_capillaries.contains(vessel)
    }
    
    fn is_post_capillary(&self, vessel: &V) -> bool {
        self.post_capillaries.contains(vessel)
    }

    fn pre_capillaries(&self) -> VesselIter<V> {
        self.pre_capillaries.iter().into()
    }
    
    fn post_capillaries(&self) -> VesselIter<V> {
        self.post_capillaries.iter().into()
    }

    fn downstream(&self, vessel: V) -> ClosedCircVesselIter<V> {
        self.vessel_connections(vessel, Direction::Outgoing)
    }
    
    fn upstream(&self, vessel: V) -> ClosedCircVesselIter<V> {
        self.vessel_connections(vessel, Direction::Incoming)
    }

    fn connector(&mut self) -> &mut SimConnector {
        self.connector.as_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::ClosedCirculationManager;
    #[test]
    fn test_manager() {

    }
}
