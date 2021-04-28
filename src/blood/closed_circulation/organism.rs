use std::collections::{HashSet, HashMap};
use std::sync::Mutex;
use anyhow::Result;
use crate::core::sim::{SimOrganism, Organism};
use crate::substance::{SubstanceStore, Time};
use crate::event::Event;
use crate::util::IdType;
use super::super::{BloodManager, BloodVessel};
use super::ClosedCircSimComponent;

lazy_static! {
    static ref COMPONENT_REGISTRY: Mutex<HashMap<&'static str, Box<dyn FnMut() -> Box<dyn ClosedCircSimComponent> + Send>>> = Mutex::new(HashMap::new());
}

/// Registers a Closed Circulation Sim component. By default, the component will be added to all newly created
/// ClosedCirculationSimOrganism objects
///
/// ### Arguments
/// * `component_name` - String name for the component
/// * `factory`        - Factory function which creates an instance of the component
fn register_component(component_name: &'static str, factory: impl FnMut() -> Box<dyn ClosedCircSimComponent> + Send + 'static) {
    log::debug!("Registering component {}", component_name);
    COMPONENT_REGISTRY.lock().unwrap().insert(component_name, Box::new(factory));
}

pub trait ClosedCirculationSimOrganism: SimOrganism {}

pub struct ClosedCirculationOrganism<T: BloodVessel> {
    organism: Organism,
    blood_manager: BloodManager<T>,
}

impl<T: BloodVessel> ClosedCirculationOrganism<T> {
    fn init_components(&mut self, _component_names: HashSet<&'static str>) {
    }
}

impl<T: BloodVessel> SimOrganism for ClosedCirculationOrganism<T> {

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