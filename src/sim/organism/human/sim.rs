use super::circulation::HumanBloodManager;
use super::module::{HumanModuleInitializer, HumanSimConnector, HumanSimModule};
use super::{HumanCirculatorySystem, HUMAN_CIRCULATION_FILEPATH};
use crate::closed_circulation::{
    BloodVessel, ClosedCircConnector, ClosedCircInitializer, ClosedCirculatorySystem,
};
use crate::core::sim::{CoreSim, Sim, SimConnector, SimModule};
use crate::event::Event;
use crate::substance::{MolarConcentration, Substance, SubstanceStore, Time};
use crate::util::IdType;
use anyhow::Result;
use petgraph::graph::{Graph, NodeIndex};
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

lazy_static! {
    static ref COMPONENT_REGISTRY: Mutex<HashMap<&'static str, Box<dyn FnMut() -> Box<dyn HumanSimModule> + Send>>> =
        Mutex::new(HashMap::new());
}

/// Registers a Sim module which interacts with a Human simulation. By default, the module will be
/// added to all newly created Human objects
///
/// ### Arguments
/// * `module_name` - String name for the module
/// * `factory`        - Factory function which creates an instance of the module
pub fn register_module(
    module_name: &'static str,
    factory: impl FnMut() -> Box<dyn HumanSimModule> + Send + 'static,
) {
    log::debug!("Registering human module: {}", module_name);
    COMPONENT_REGISTRY
        .lock()
        .unwrap()
        .insert(module_name, Box::new(factory));
}

pub struct HumanSim {
    core: CoreSim,
    blood_manager: HumanBloodManager,
    active_modules: HashMap<&'static str, Box<dyn HumanSimModule>>,
    connector_map: HashMap<&'static str, HumanSimConnector>,
}

impl HumanSim {
    /// Creates a new `HumanSim` object
    pub fn new() -> HumanSim {
        HumanSim {
            core: CoreSim::new(),
            blood_manager: HumanBloodManager::new(HumanCirculatorySystem::new()),
            active_modules: HashMap::new(),
            connector_map: HashMap::new(),
        }
    }

    fn init_modules(&mut self, module_names: HashSet<&'static str>) {
        let mut registry = COMPONENT_REGISTRY.lock().unwrap();

        let mut remaining_modules = HashSet::new();

        // Initialize each module
        for module_name in module_names.into_iter() {
            log::debug!("Initializing module \"{}\" on HumanSim", module_name);

            match registry.get_mut(module_name) {
                None => {
                    remaining_modules.insert(module_name);
                }
                Some(factory) => {
                    let mut module = factory();
                    let mut human_initializer = HumanModuleInitializer::new();
                    module.init(&mut human_initializer.initializer);

                    self.core
                        .setup_module(module_name, human_initializer.initializer);
                    self.active_modules.insert(module_name, module);

                    let cc_connector = self
                        .blood_manager
                        .setup_module(module_name, human_initializer.cc_initializer);

                    let human_connector = HumanSimConnector::new(SimConnector::new(), cc_connector);
                    self.connector_map.insert(module_name, human_connector);
                }
            }
        }

        // Initialize any blood modules
        // self.blood_manager.init_modules(remaining_modules, &mut self.core);

        for (name, module) in self.active_modules.iter_mut() {
            self.core.init_module(name, module.as_core_module());
        }
    }

    fn execute_time_step(&mut self) {
        let pending_updates: Vec<&str> = self.core.pending_updates().collect();
        for module_name in pending_updates {
            if self.connector_map.contains_key(module_name) {
                // TODO execution logic
            }
        }
    }
}

impl Sim for HumanSim {
    /// Returns the current simulation time
    fn get_time(&self) -> Time {
        self.core.get_time()
    }

    fn has_module(&self, module_name: &'static str) -> bool {
        if self.active_modules.contains_key(module_name) {
            true
        } else {
            self.core.has_module(module_name)
        }
    }

    /// Retrieves the set of names of modules which are active on this Sim
    fn active_modules(&self) -> HashSet<&'static str> {
        self.core.active_modules()
    }

    /// Adds modules to this Sim. Panics if any module names are invalid
    ///
    /// ### Arguments
    /// * `module_names` - Set of modules to add
    fn add_modules(&mut self, module_names: HashSet<&'static str>) {
        self.init_modules(module_names);
    }

    /// Removes a module from this Sim. Panics if any of the module names
    /// are invalid.
    ///
    /// ### Arguments
    /// * `module_names` - Set of modules to remove
    fn remove_modules(&mut self, module_names: HashSet<&'static str>) {
        self.core.remove_modules(module_names)
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
    use crate::core::sim::{Sim, Time};
    use uom::si::time::second;

    #[test]
    fn test_human_sim() {
        let mut sim = HumanSim::new();
        assert_eq!(sim.get_time(), Time::from_s(0.0));
        sim.advance_by(Time::from_s(1.0));
        assert_eq!(sim.get_time(), Time::from_s(1.0));
    }
}
