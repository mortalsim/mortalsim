use std::collections::{HashSet, HashMap};
use std::sync::Mutex;
use std::any::{Any, TypeId};
use anyhow::Result;
use petgraph::graph::{Graph, NodeIndex};
use crate::core::sim::{SimOrganism, Organism};
use crate::substance::{SubstanceStore, Time};
use crate::event::Event;
use crate::util::IdType;
use super::super::BloodVessel;
use super::{BloodNode, ClosedCirculationManager, ClosedCircComponentInitializer, ClosedCircSimComponent};

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

struct CcComponentContext<V: BloodVessel> {
    component: Box<dyn ClosedCircSimComponent<VesselType = V>>,
    node_indices: Vec<NodeIndex>,
}

pub trait ClosedCirculationSimOrganism: SimOrganism {}

pub struct ClosedCirculationOrganism<V: BloodVessel> {
    organism: Organism,
    blood_manager: ClosedCirculationManager<V>,
    active_components: HashMap<&'static str, CcComponentContext<V>>,
}

impl<V: BloodVessel + 'static> ClosedCirculationOrganism<V> {
    fn init_cc_components(&mut self, component_names: HashSet<&'static str>) -> HashSet<&'static str> {
        let mut registry = COMPONENT_REGISTRY.lock().unwrap();
        let vessel_registry: &mut HashMap<&'static str, Box<dyn Any + Send>> = registry.entry(TypeId::of::<V>()).or_insert(HashMap::new());

        let mut remaining_components = HashSet::new();

        // Initialize each component
        for component_name in component_names.into_iter() {
            log::debug!("Initializing component \"{}\" on Sim", component_name);
            match vessel_registry.get_mut(component_name) {
                None => {
                    remaining_components.insert(component_name);
                },
                Some(factory_box) => {
                    let factory = factory_box.downcast_mut::<Box<dyn FnMut() -> Box<dyn ClosedCircSimComponent<VesselType = V>>>>().unwrap();
                    let mut ctx = CcComponentContext {
                        component: factory(),
                        node_indices: Vec::new(),
                    };

                    self.setup_component_io(&mut ctx);
                    self.active_components.insert(component_name, ctx);
                }
            }
        }

        // TODO: Any finishing initializations
        remaining_components
    }

    fn setup_component_io(&mut self, ctx: &mut CcComponentContext<V>) {
        let mut initializer = ClosedCircComponentInitializer::new();
        ctx.component.init(&mut initializer);

        // TODO: Process initialization
    }
}

impl<V: BloodVessel + 'static> SimOrganism for ClosedCirculationOrganism<V> {

    /// Returns the current simulation time
    fn time(&self) -> Time {
        self.organism.time()
    }

    /// Retrieves the set of names of components which are active on this Sim
    fn active_components(&self) -> HashSet<&'static str> {
        self.organism.active_components()
    }

    /// Adds components to this Sim. Panics if any component names are invalid
    ///
    /// ### Arguments
    /// * `component_names` - Set of components to add
    fn add_components(&mut self, component_names: HashSet<&'static str>) {
        self.init_cc_components(component_names);
    }

    /// Removes a component from this Sim. Panics if any of the component names
    /// are invalid.
    ///
    /// ### Arguments
    /// * `component_names` - Set of components to remove
    fn remove_components(&mut self, component_names: HashSet<&'static str>) {
        self.organism.remove_components(component_names)
    }


    /// Advances simulation time to the next `Event` or listener in the queue, if any.
    /// 
    /// If there are no Events or listeners in the queue, time will remain unchanged
    fn advance(&mut self) {
        self.organism.advance()
    }

    /// Advances simulation time by the provided time step
    /// 
    /// If a negative value is provided, time will immediately jump to
    /// the next scheduled Event, if any.
    /// 
    /// ### Arguments
    /// * `time_step` - Amount of time to advance by
    fn advance_by(&mut self, time_step: Time) {
        self.organism.advance_by(time_step)
    }


    /// Schedules an `Event` for future emission on this simulation
    /// 
    /// ### Arguments
    /// * `wait_time` - amount of simulation time to wait before emitting the Event
    /// * `event` - Event instance to emit
    /// 
    /// Returns the schedule ID
    fn schedule_event(&mut self, wait_time: Time, event: impl Event) -> IdType {
        self.organism.schedule_event(wait_time, event)
    }

    /// Unschedules a previously scheduled `Event`
    /// 
    /// ### Arguments
    /// * `schedule_id` - Schedule ID returned by `schedule_event`
    /// 
    /// Returns an Err Result if the provided ID is invalid
    fn unschedule_event(&mut self, schedule_id: &IdType) -> Result<()> {
        self.organism.unschedule_event(schedule_id)
    }
}