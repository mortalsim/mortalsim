use std::collections::{HashSet, HashMap};
use std::sync::Mutex;
use std::any::{Any, TypeId};
use anyhow::Result;
use petgraph::graph::{Graph, NodeIndex};
use crate::core::sim::{Sim, CoreSim, SimConnector};
use crate::substance::{SubstanceStore, Time, Substance, MolarConcentration};
use crate::event::Event;
use crate::util::IdType;
use crate::closed_circulation::{BloodVessel, ClosedCircInitializer, ClosedCircConnector, ClosedCirculatorySystem};
use super::{HUMAN_CIRCULATION_FILEPATH, HumanCirculatorySystem};
use super::circulation::HumanBloodManager;
use super::component::{HumanSimComponent, HumanSimConnector, HumanComponentInitializer};

lazy_static! {
    static ref COMPONENT_REGISTRY: Mutex<HashMap<&'static str, Box<dyn FnMut() -> Box<dyn HumanSimComponent> + Send>>> = Mutex::new(HashMap::new());
}

/// Registers a Sim component which interacts with a Human simulation. By default, the component will be
/// added to all newly created Human objects
///
/// ### Arguments
/// * `component_name` - String name for the component
/// * `factory`        - Factory function which creates an instance of the component
fn register_component(component_name: &'static str, factory: impl FnMut() -> Box<dyn HumanSimComponent> + Send + 'static) {
    log::debug!("Registering human component: {}", component_name);
    COMPONENT_REGISTRY.lock().unwrap().insert(component_name, Box::new(factory));
}

pub struct HumanSim {
    core: CoreSim,
    blood_manager: HumanBloodManager,
    active_components: HashMap<&'static str, Box<dyn HumanSimComponent>>,
    connector_map: HashMap<&'static str, HumanSimConnector>,
}

impl HumanSim {
    /// Creates a new `HumanSim` object
    pub fn new() -> HumanSim {
        HumanSim {
            core: CoreSim::new(),
            blood_manager: HumanBloodManager::new(HumanCirculatorySystem::new()),
            active_components: HashMap::new(),
            connector_map: HashMap::new()
        }
    }

    fn init_components(&mut self, component_names: HashSet<&'static str>) {
        let mut registry = COMPONENT_REGISTRY.lock().unwrap();

        let mut remaining_components = HashSet::new();

        // Initialize each component
        for component_name in component_names.into_iter() {
            log::debug!("Initializing component \"{}\" on HumanSim", component_name);

            match registry.get_mut(component_name) {
                None => {
                    remaining_components.insert(component_name);
                },
                Some(factory) => {
                    let mut component = factory();
                    let mut human_initializer = HumanComponentInitializer::new();
                    component.init(&mut human_initializer);

                    self.active_components.insert(component_name, component);

                    let connector = self.core.setup_component(component_name, human_initializer.initializer);
                    let cc_connector = self.blood_manager.setup_component(component_name, human_initializer.cc_initializer);

                    let human_connector = HumanSimConnector::new(connector, cc_connector);
                    self.connector_map.insert(component_name, human_connector);
                }
            }
        }

        // Initialize any blood components
        remaining_components = self.blood_manager.init_components(remaining_components, &mut self.core);
        
        // All remaining components should be core components
        self.core.init_components(remaining_components);
    }

    fn execute_time_step(&mut self) {
        let pending_updates: Vec<&str> = self.core.pending_updates().collect();
        for component_name in pending_updates {
            if self.connector_map.contains_key(component_name) {
                // TODO execution logic
            }
        }
    }
}

impl Sim for HumanSim {

    /// Returns the current simulation time
    fn time(&self) -> Time {
        self.core.time()
    }

    fn has_component(&self, component_name: &'static str) -> bool {
        if self.active_components.contains_key(component_name) {
            true
        }
        else {
            self.core.has_component(component_name)
        }
    }

    /// Retrieves the set of names of components which are active on this Sim
    fn active_components(&self) -> HashSet<&'static str> {
        self.core.active_components()
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
        self.core.remove_components(component_names)
    }

    /// Advances simulation time to the next `Event` or listener in the queue, if any.
    /// 
    /// If there are no Events or listeners in the queue, time will remain unchanged
    fn advance(&mut self) {
        self.core.advance();
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
        self.core.advance_by(time_step);
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
        self.core.schedule_event(wait_time, event)
    }

    /// Unschedules a previously scheduled `Event`
    /// 
    /// ### Arguments
    /// * `schedule_id` - Schedule ID returned by `schedule_event`
    /// 
    /// Returns an Err Result if the provided ID is invalid
    fn unschedule_event(&mut self, schedule_id: &IdType) -> Result<()> {
        self.core.unschedule_event(schedule_id)
    }
}

#[cfg(test)]
mod tests {
    use super::HumanSim;
    use crate::core::sim::{Time, Sim};
    use uom::si::time::second;

    #[test]
    fn test_human_sim() {
        let mut sim = HumanSim::new();
        assert_eq!(sim.time(), Time::new::<second>(0.0));
        sim.advance_by(Time::new::<second>(1.0));
        assert_eq!(sim.time(), Time::new::<second>(1.0));
    }
}
