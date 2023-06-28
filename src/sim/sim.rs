use super::Time;
use crate::{event::Event, util::IdType};
use anyhow::Result;
use std::collections::HashSet;

pub trait Sim {
    /// Returns the current simulation time
    fn get_time(&self) -> Time;

    /// Determines if the given component name corresponds to an active component
    /// on this Sim
    fn has_component(&self, component_name: &'static str) -> bool;

    /// Retrieves the set of names of components which are active on this Sim
    fn active_components(&self) -> HashSet<&'static str>;

    /// Adds components to this Sim. Panics if any component names are invalid
    ///
    /// ### Arguments
    /// * `component_names` - Set of components to add
    fn add_components(&mut self, component_names: HashSet<&'static str>);

    /// Removes a component from this Sim. Panics if any of the component names
    /// are invalid.
    ///
    /// ### Arguments
    /// * `component_names` - Set of components to remove
    fn remove_components(&mut self, component_names: HashSet<&'static str>);

    /// Advances simulation time to the next `Event` or listener in the queue, if any.
    ///
    /// If there are no Events or listeners in the queue, time will remain unchanged
    fn advance(&mut self);

    /// Advances simulation time by the provided time step
    ///
    /// If a negative value is provided, time will immediately jump to
    /// the next scheduled Event, if any.
    ///
    /// ### Arguments
    /// * `time_step` - Amount of time to advance by
    fn advance_by(&mut self, time_step: Time);

    /// Schedules an `Event` for future emission on this simulation
    ///
    /// ### Arguments
    /// * `wait_time` - amount of simulation time to wait before emitting the Event
    /// * `event` - Event instance to emit
    ///
    /// Returns the schedule ID
    fn schedule_event(&mut self, wait_time: Time, event: impl Event) -> IdType;

    /// Unschedules a previously scheduled `Event`
    ///
    /// ### Arguments
    /// * `schedule_id` - Schedule ID returned by `schedule_event`
    ///
    /// Returns an Err Result if the provided ID is invalid
    fn unschedule_event(&mut self, schedule_id: &IdType) -> Result<()>;
}
