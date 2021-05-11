use std::collections::HashMap;
use std::sync::Mutex;
use std::any::{Any, TypeId};
use petgraph::graph::{Graph, Neighbors};
use super::{BloodVessel};

mod organism;
mod system;
mod component;
mod graph;
mod manager;

pub use system::ClosedCirculatorySystem;
pub use component::{ClosedCircSimComponent, ClosedCircComponentInitializer, ClosedCircInitializer, ClosedCircConnector, ClosedCircSimConnector};
pub use graph::{BloodEdge, BloodNode};
pub use manager::ClosedCirculationManager;
pub use organism::{ClosedCirculationOrganism, ClosedCirculationSimOrganism};

lazy_static! {
    static ref COMPONENT_REGISTRY: Mutex<HashMap<TypeId, HashMap<&'static str, Box<dyn Any + Send>>>> = Mutex::new(HashMap::new());
}

/// Registers a Sim component which interacts with an organism's closed circulatory system. By default, the component will be
/// added to all newly created ClosedCirculationSimOrganism objects
///
/// ### Arguments
/// * `component_name` - String name for the component
/// * `factory`        - Factory function which creates an instance of the component
fn register_component<V: BloodVessel + 'static>(component_name: &'static str, factory: impl FnMut() -> Box<dyn ClosedCircSimComponent<VesselType = V>> + Send + 'static) {
    log::debug!("Registering closed circulation component: {}", component_name);

    let mut registry = COMPONENT_REGISTRY.lock().unwrap();
    let vessel_registry = registry.entry(TypeId::of::<V>()).or_insert(HashMap::new());

    // Gotta box it into an Any since we have different structs based on the BloodVessel type
    vessel_registry.insert(component_name, Box::new(factory));
}

pub struct ClosedCircVesselIter<'a, 'b, V: BloodVessel> {
    graph: &'a Graph<BloodNode<V>, BloodEdge>,
    idx_iter: Option<Neighbors<'b, BloodEdge>>,
}

impl<'a, 'b, V: BloodVessel> Iterator for ClosedCircVesselIter<'a, 'b, V> {
    type Item = V;
    fn next(&mut self) -> Option<V> {
        Some(self.graph[self.idx_iter.as_mut()?.next()?].vessel)
    }
}