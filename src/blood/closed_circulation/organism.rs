use std::collections::{HashSet, HashMap};
use std::sync::Mutex;
use std::any::{Any, TypeId};
use anyhow::Result;
use petgraph::graph::{Graph, NodeIndex};
use crate::core::sim::{SimOrganism, Organism, SimConnector};
use crate::substance::{SubstanceStore, Time, Substance, MolarConcentration};
use crate::event::Event;
use crate::util::IdType;
use super::super::BloodVessel;
use super::{BloodNode, ClosedCirculationManager, ClosedCircComponentInitializer, ClosedCircInitializer, ClosedCircSimComponent};

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
    blood_index_map: HashMap<&'static str, (HashSet<V>, HashMap<V, Vec<Substance>>)>,
    blood_notify_map: HashMap<V, HashMap<Substance, Vec<(MolarConcentration, &'static str)>>>,
    active_components: HashMap<&'static str, Box<dyn ClosedCircSimComponent<VesselType = V>>>,
    connector_map: HashMap<&'static str, SimConnector>,
}

impl<V: BloodVessel + 'static> ClosedCirculationOrganism<V> {
    fn init_components(&mut self, component_names: HashSet<&'static str>) {
        let mut registry = COMPONENT_REGISTRY.lock().unwrap();
        let vessel_registry: &mut HashMap<&'static str, Box<dyn Any + Send>> = registry.entry(TypeId::of::<V>()).or_insert(HashMap::new());

        let mut remaining_components = HashSet::new();

        // Initialize each component
        for component_name in component_names.into_iter() {
            log::debug!("Initializing component \"{}\" on ClosedCirculation", component_name);
            match vessel_registry.get_mut(component_name) {
                None => {
                    remaining_components.insert(component_name);
                },
                Some(factory_box) => {
                    let factory = factory_box.downcast_mut::<Box<dyn FnMut() -> Box<dyn ClosedCircSimComponent<VesselType = V>>>>().unwrap();
                    let mut component = factory();
                    let mut ccc_initializer = ClosedCircComponentInitializer::new();
                    component.init(&mut ccc_initializer);
                    
                    self.active_components.insert(component_name, component);

                    // Need to create a SimConnector for each of our closed circ components so they
                    // have access to the base sim capabilities as well
                    let mut connector = SimConnector::new();

                    // perform base organism component portion setup
                    self.organism.setup_component(component_name, ccc_initializer.initializer, &mut connector);
                    self.connector_map.insert(component_name, connector);

                    // perform closed circulation component portion setup
                    self.setup_component(component_name, ccc_initializer.cc_initializer);
                }
            }
        }
        
        // Remaining components should be generic SimComponents
        // so we let the base organism handle those
        self.init_base_components(remaining_components);
    }

    pub(crate) fn init_base_components(&mut self, component_names: HashSet<&'static str>) {
        self.organism.init_components(component_names);
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

        self.blood_index_map.insert(component_name, (vessel_connections, component_vessel_map));
    }

    fn execute_time_step(&mut self) {
        self.organism.execute_events();
        let update_list = self.organism.update_list();
    }

    pub(crate) fn advance_time(&mut self) {
        self.organism.advance_time();
    }
    
    pub(crate) fn advance_time_by(&mut self, time_step: Time) {
        self.organism.advance_time_by(time_step);
    }
}

impl<V: BloodVessel + 'static> SimOrganism for ClosedCirculationOrganism<V> {

    /// Returns the current simulation time
    fn time(&self) -> Time {
        self.organism.time()
    }

    fn has_component(&self, component_name: &'static str) -> bool {
        if self.active_components.contains_key(component_name) {
            true
        }
        else {
            self.organism.has_component(component_name)
        }
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
        self.init_components(component_names);
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
        self.advance_time();
        self.execute_time_step();
    }

    /// Advances simulation time by the provided time step
    /// 
    /// If a negative value is provided, time will immediately jump to
    /// the next scheduled Event, if any.
    /// 
    /// ### Arguments
    /// * `time_step` - Amount of time to advance by
    fn advance_by(&mut self, time_step: Time) {
        self.advance_time_by(time_step);
        self.execute_time_step();
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