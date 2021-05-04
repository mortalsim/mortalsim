use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::collections::hash_set;
use std::rc::Rc;
use std::string;
use std::any::{Any, TypeId};
use petgraph::Direction;
use petgraph::graph::{Graph, NodeIndex, Neighbors};
use crate::core::sim::{SimConnector, Organism};
use crate::substance::{SubstanceStore, Volume, Substance, MolarConcentration};
use super::super::{BloodVessel, BloodVesselType, VesselIter};
use super::{BloodNode, BloodEdge, ClosedCirculatorySystem, ClosedCircVesselIter,
    ClosedCircConnector, ClosedCircSimConnector, ClosedCircSimComponent,
    ClosedCircComponentInitializer, ClosedCircInitializer, COMPONENT_REGISTRY};

struct ComponentContext<V: BloodVessel> {
    connected_vessels: HashSet<V>,
    substance_notify_map: HashMap<V, Vec<Substance>>,
}

pub struct ClosedCirculationManager<V: BloodVessel> {
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

impl<V: BloodVessel> ClosedCircConnector<V> for ClosedCirculationManager<V> {
    fn composition(&self, vessel: V) -> &SubstanceStore {
        let node_idx = self.node_map.get(&vessel).unwrap();
        &self.graph[*node_idx].composition
    }

    fn composition_mut(&mut self, vessel: V) -> &mut SubstanceStore {
        let node_idx = self.node_map.get(&vessel).unwrap();
        &mut self.graph[*node_idx].composition
    }

    fn connector(&mut self) -> &mut SimConnector {
        self.connector.as_mut().unwrap()
    }
}

impl<V: BloodVessel + 'static> ClosedCircSimConnector<V> for ClosedCirculationManager<V> {
    /// Retrieves the maximum depth of the circulation tree (from root to capillary)
    fn depth(&self) -> u32 {
        self.depth as u32
    }

    /// Returns the BloodVesselType for the given vessel. Panics if the vessel is invalid
    fn vessel_type(&self, vessel: V) -> BloodVesselType {
        let node_idx = self.node_map.get(&vessel).unwrap();
        self.graph[*node_idx].vessel_type
    }

    /// Determines whether the given vessel is a pre-capillary vessel
    /// (Artery with no more downstream arteries, only veins)
    fn is_pre_capillary(&self, vessel: &V) -> bool {
        self.pre_capillaries.contains(vessel)
    }
    
    /// Determines whether the given vessel is a post-capillary vessel
    /// (Vein with no more upstream veins, only arteries)
    fn is_post_capillary(&self, vessel: &V) -> bool {
        self.post_capillaries.contains(vessel)
    }

    /// Retrieves an iterator of pre-capillary vessels
    /// (Arteries with no more downstream arteries, only veins)
    fn pre_capillaries(&self) -> VesselIter<V> {
        self.pre_capillaries.iter().into()
    }
    
    /// Retrieves an iterator of post-capillary vessels
    /// (Veins with no more upstream veins, only arteries)
    fn post_capillaries(&self) -> VesselIter<V> {
        self.post_capillaries.iter().into()
    }

    /// Retrieves an iterator over all downstream vessels from
    /// the provided vessel
    fn downstream(&self, vessel: V) -> ClosedCircVesselIter<V> {
        self.vessel_connections(vessel, Direction::Outgoing)
    }
    
    /// Retrieves an iterator over all upstream vessels from
    /// the provided vessel
    fn upstream(&self, vessel: V) -> ClosedCircVesselIter<V> {
        self.vessel_connections(vessel, Direction::Incoming)
    }
}

#[cfg(test)]
mod tests {
    use super::ClosedCirculationManager;
    #[test]
    fn test_manager() {

    }
}
