use std::ptr::null;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::any::TypeId;
use anyhow::{Error, Result};
use uom::si::time::second;
use crate::util::id_gen::IdType;
use crate::sim::{SimState, Time};
use crate::event::Event;

/// Provides methods for `Sim` modules to interact with the simulation
pub struct CoreConnector {
    /// State specific to the connected module
    pub(crate) sim_state: Arc<Mutex<SimState>>,
    /// Holds a shared reference to the Event which triggered module execution
    pub(crate) trigger_events: Vec<Arc<dyn Event>>,
    /// Map of scheduled event identifiers
    pub(crate) scheduled_events: HashMap<TypeId, HashMap<IdType, Time>>,
    /// Map of scheduled ids to event types
    pub(crate) schedule_id_type_map: HashMap<IdType, TypeId>,
    /// List of events to schedule
    pub(crate) pending_schedules: Vec<(Time, Box<dyn Event>)>,
    /// List of events to unschedule
    pub(crate) pending_unschedules: Vec<IdType>,
    /// Copy of the current simulation time
    pub(crate) sim_time: Time,
    /// Whether to indicate to the parent Sim that all previously scheduled events should be unscheduled
    pub(crate) unschedule_all: bool,
}

impl CoreConnector {
    
    /// Creates a new CoreConnector
    pub fn new() -> CoreConnector {
        CoreConnector {
            // Temporary empty state which will be replaced by the canonical state
            sim_state: Arc::new(Mutex::new(SimState::new())),
            trigger_events: Vec::new(),
            scheduled_events: HashMap::new(),
            schedule_id_type_map: HashMap::new(),
            pending_schedules: Vec::new(),
            pending_unschedules: Vec::new(),
            sim_time: Time::new::<second>(0.0),
            unschedule_all: true,
        }
    }
    
    /// Schedules an `Event` for future emission after a specified delay
    /// 
    /// ### Arguments
    /// * `wait_time` - Amount of time to wait before execution
    /// * `evt` - `Event` to emit after `wait_time` has elapsed
    pub fn schedule_event(&mut self, wait_time: Time, evt: impl Event) {
        self.pending_schedules.push((wait_time, Box::new(evt)))
    }
    
    /// Whether to unschedule all previously scheduled `Event` objects (default is true)
    /// Set to `false` in order to manually specify which `Event` objects to unschedule
    /// using `unschedule_event`
    pub fn unschedule_all(&mut self, setting: bool) {
        self.unschedule_all = setting;
    }
    
    /// Unschedules an `Event` which has been scheduled previously.
    /// 
    /// ### Arguments
    /// * `schedule_id` - schedule id of the Event to unschedule
    /// 
    /// Returns Ok if the id is valid, and Err otherwise
    pub fn unschedule_event<E: Event>(&mut self, schedule_id: IdType) -> Result<()> {
        let type_id = TypeId::of::<E>();
        match self.scheduled_events.get(&type_id) {
            Some(smap) => {
                if smap.contains_key(&schedule_id) {
                    Ok(())
                }
                else {
                    Err(anyhow!("Invalid id provided for unscheduling"))
                }
            }
            None => Err(anyhow!("Invalid type provided for unscheduling"))
        }
    }

    /// Retrieves a mapping of schedule ids -> execution time for each
    /// instance of the given Event type which has been scheduled previously,
    /// and which has not yet been emitted.
    /// 
    /// Returns a HashMap if any events are scheduled for the given type, and
    /// None otherwise
    pub fn get_scheduled_events<E: Event>(&mut self) -> Option<&HashMap<IdType, Time>> {
        self.scheduled_events.get(&TypeId::of::<E>())
    }

    /// Retrieves the current simulation time
    pub fn get_time(&self) -> Time {
        self.sim_time
    }

    /// Retrieves the current `Event` object from state as an Arc
    pub fn get<E: Event>(&self) -> Option<Arc<E>> {
        match self.sim_state.lock().unwrap().get_state_ref(&TypeId::of::<E>()) {
            None => None,
            Some(evt_rc) => {
                match evt_rc.downcast_arc::<E>() {
                    Err(_) => None,
                    Ok(typed_evt_rc) => {
                        Some(typed_evt_rc)
                    }
                }
            }
        }
    }
    
    /// Retrieves the `Event` object(s) which triggered the current `run`
    pub fn trigger_events<'a>(&'a self) -> impl Iterator<Item = &Arc<dyn Event>> + 'a {
        self.trigger_events.iter()
    }
}