use std::sync::Arc;
use std::rc::{Rc, Weak};
use std::collections::HashMap;
use std::cell::{Ref, RefCell, Cell};
use std::any::TypeId;
use std::io;
use std::convert::From;
use anyhow::{Error, Result};
use crate::util::id_gen::IdType;
use crate::core::hub::EventHub;
use crate::core::sim::{SimState, TimeManager, Time};
use crate::event::Event;

/// Provides methods for `Sim` components to interact with the simulation
pub struct SimConnector<'a> {
    /// State specific to the connected component
    pub(in super::super) local_state: SimState,
    /// Ref to the `Sim`'s `TimeManager` instance for scheduling `Event` objects
    time_manager: Rc<RefCell<TimeManager<'a>>>,
    /// Holds a shared reference to the Event which triggered component execution
    trigger_event: Option<Arc<dyn Event>>,
    /// List of scheduled event identifiers
    schedule_ids: Vec<IdType>,
}

impl<'a> SimConnector<'a> {
    
    /// Creates a new SimConnector
    /// 
    /// ### Arguments
    /// * `time_manager` - Reference to the `Sim` object's `TimeManager` instance
    /// 
    /// returns the newly constructed SimConnector
    pub fn new(time_manager: Rc<RefCell<TimeManager<'a>>>) -> SimConnector<'a> {
        SimConnector {
            local_state: SimState::new(),
            time_manager: time_manager,
            trigger_event: None,
            schedule_ids: Vec::new(),
        }
    }
    
    /// Internal library function to prepare for the corresponding component to
    /// perform its next execution
    /// 
    /// ### Arguments
    /// * `evt` - Reference to the `Event` object which will trigger the component
    pub(super) fn set_trigger(&mut self, evt: Arc<dyn Event>) {
        self.trigger_event = Some(evt);
        self.unschedule_events();
    }
    
    /// Internal function for clearing scheduled events
    fn unschedule_events(&mut self) {
        for schedule_id in self.schedule_ids.iter() {
            // ignore any Err results since it just means the `Event` has already executed
            self.time_manager.borrow_mut().unschedule_event(*schedule_id).unwrap_or_default();
        }

        // Clear the schedule_ids vec for the next run
        self.schedule_ids.clear();
    }

    /// Schedules an `Event` for future emission after a specified delay
    /// 
    /// ### Arguments
    /// * `wait_time` - Amount of time to wait before execution
    /// * `evt` - `Event` to emit after `wait_time` has elapsed
    /// 
    /// Returns a schedule id which can be used to unschedule if needed later
    pub fn schedule_event<T: Event>(&mut self, wait_time: Time, evt: T) {
        self.schedule_ids.push(self.time_manager.borrow_mut().schedule_event(wait_time, evt));
    }

    /// Retrieves the current simulation time
    pub fn get_time(&self) -> Time {
        self.time_manager.borrow().get_time()
    }

    /// Retrieves the current `Event` object from state
    pub fn get<T: Event>(&self) -> Option<&T> {
        self.local_state.get_state::<T>()
    }

    /// Retrieves the current `Event` object from state as an Arc
    pub fn get_arc<T: Event>(&self) -> Option<Arc<T>> {
        match self.local_state.get_state_ref(&TypeId::of::<T>()) {
            None => None,
            Some(evt_rc) => {
                match evt_rc.downcast_arc::<T>() {
                    Err(_) => None,
                    Ok(typed_evt_rc) => {
                        Some(typed_evt_rc)
                    }
                }
            }
        }
    }
    
    /// Retrieves the current `Event` object which triggered the current `run`
    pub fn get_trigger_event(&self) -> Option<&dyn Event> {
        match &self.trigger_event {
            None => None,
            Some(evt) => {
                Some(&**evt)
            }
        }
    }
}
