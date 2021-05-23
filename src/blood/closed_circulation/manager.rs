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
    tmp_store_map: Option<HashMap<V, SubstanceStore>>,
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
            tmp_store_map: Some(HashMap::new()),
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
                    let mut ccc_initializer = ClosedCircComponentInitializer::new();
                    component.init(&mut ccc_initializer);
                    
                    self.active_components.insert(component_name, component);

                    // perform base core component portion setup
                    let connector = core.setup_component(component_name, ccc_initializer.initializer);
                    self.connector_map.insert(component_name, connector);

                    // perform closed circulation component portion setup
                    self.setup_component(component_name, ccc_initializer.cc_initializer);
                }
            }
        }

        // Set up event notifications for this extension
        core.notify_extension::<BloodCompositionChange<V>>(self.manager_id);
        core.notify_extension::<BloodVolumeChange<V>>(self.manager_id);
        
        remaining_components
    }

    pub(crate) fn setup_component(&mut self, component_name: &'static str, mut initializer: ClosedCircInitializer<V>) -> ClosedCircConnector<V> {

        let mut component_vessel_map = HashMap::new();
        for (vessel, substance_map) in initializer.substance_notifies.drain() {
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
            connected_vessels: initializer.vessel_connections.clone(),
            substance_notify_map: component_vessel_map,
        });
        ClosedCircConnector::new(initializer)
    }

    pub(crate) fn update(&mut self, update_list: impl Iterator<Item = &'static str>, organism: &mut CoreSim) -> impl Iterator<Item = &'static str> {

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
                    self.connector = Some(connector);
                    component.run(self);
                    connector = self.connector.take().unwrap();

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

    pub(crate) fn prepare_connector(&mut self, connector: &mut ClosedCircConnector<V>) {
        for vessel in connector.vessel_connections.iter() {
            let node_idx = self.node_map.get(&vessel).unwrap();
            // TODO: Cloning composition is relatively slow, but dealing with references
            // here is a pain, so we can optimize this later
            connector.stores.insert(*vessel, self.graph[*node_idx].composition.clone());
        }
    }

    pub(crate) fn process_connector(&mut self, _connector: &mut ClosedCircConnector<V>) {
        // ... Nothing to do here for now
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

impl<V: BloodVessel + 'static> ClosedCircSimConnector<V> for ClosedCirculationManager<V> {
    fn depth(&self) -> u32 {
        self.depth as u32
    }

    fn blood_store(&self, vessel: &V) -> Option<&SubstanceStore> {
        let node_idx = self.node_map.get(vessel)?;
        Some(&self.graph[*node_idx].composition)
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
