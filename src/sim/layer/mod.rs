use super::{TimeManager, SimState, Time};

pub mod core;

// use std::collections::HashSet;
// use anyhow::Result;
// use crate::{event::Event, util::IdType};
// use super::Time;

// pub trait SimSystem {
//   /// Returns the current simulation time
//   fn get_time(&self) -> Time;
  
//   /// Determines if the given module name corresponds to an active module
//   /// on this Sim
//   fn has_module(&self, module_name: &'static str) -> bool;

//   /// Retrieves the set of names of modules which are active on this Sim
//   fn active_modules(&self) -> HashSet<&'static str>;

//   /// Adds modules to this Sim. Panics if any module names are invalid
//   ///
//   /// ### Arguments
//   /// * `module_names` - Set of modules to add
//   fn add_modules(&mut self, module_names: HashSet<&'static str>);

//   /// Removes a module from this Sim. Panics if any of the module names
//   /// are invalid.
//   ///
//   /// ### Arguments
//   /// * `module_names` - Set of modules to remove
//   fn remove_modules(&mut self, module_names: HashSet<&'static str>);

//   /// Advances simulation time to the next `Event` or listener in the queue, if any.
//   /// 
//   /// If there are no Events or listeners in the queue, time will remain unchanged
//   fn advance(&mut self);

//   /// Advances simulation time by the provided time step
//   /// 
//   /// If a negative value is provided, time will immediately jump to
//   /// the next scheduled Event, if any.
//   /// 
//   /// ### Arguments
//   /// * `time_step` - Amount of time to advance by
//   fn advance_by(&mut self, time_step: Time);

//   /// Schedules an `Event` for future emission on this simulation
//   /// 
//   /// ### Arguments
//   /// * `wait_time` - amount of simulation time to wait before emitting the Event
//   /// * `event` - Event instance to emit
//   /// 
//   /// Returns the schedule ID
//   fn schedule_event(&mut self, wait_time: Time, event: impl Event) -> IdType;

//   /// Unschedules a previously scheduled `Event`
//   /// 
//   /// ### Arguments
//   /// * `schedule_id` - Schedule ID returned by `schedule_event`
//   /// 
//   /// Returns an Err Result if the provided ID is invalid
//   fn unschedule_event(&mut self, schedule_id: &IdType) -> Result<()>;
// }