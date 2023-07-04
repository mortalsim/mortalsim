use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Mutex;

mod graph;
mod component;
mod closed_circulation_layer;
mod system;
mod vessel;

pub use graph::{BloodEdge, BloodNode};
pub use component::{ClosedCircConnector, ClosedCircInitializer, ClosedCircComponent};
pub use closed_circulation_layer::ClosedCirculationLayer;
pub use system::{ClosedCircVesselIter, ClosedCirculatorySystem};
pub use vessel::{BloodVessel, VesselIter};

lazy_static! {
    static ref COMPONENT_REGISTRY: Mutex<HashMap<TypeId, HashMap<&'static str, Box<dyn Any + Send>>>> =
        Mutex::new(HashMap::new());
}

/// Registers a Sim module which interacts with an organism's closed circulatory system. By default, the module will be
/// added to all newly created ClosedCirculationSim objects
///
/// ### Arguments
/// * `module_name` - String name for the module
/// * `factory`        - Factory function which creates an instance of the module
fn register_module<V: BloodVessel + 'static>(
    module_name: &'static str,
    factory: impl FnMut() -> Box<dyn ClosedCircComponent<VesselType = V>> + Send + 'static,
) {
    log::debug!("Registering closed circulation module: {}", module_name);

    let mut registry = COMPONENT_REGISTRY.lock().unwrap();
    let vessel_registry = registry.entry(TypeId::of::<V>()).or_insert(HashMap::new());

    // Gotta box it into an Any since we have different structs based on the BloodVessel type
    vessel_registry.insert(module_name, Box::new(factory));
}
