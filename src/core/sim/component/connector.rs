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
pub struct BioConnector<'a> {
    /// State specific to the connected component
    pub(in super::super) local_state: SimState,
    /// Ref to the `Sim`'s `TimeManager` instance for scheduling `Event` objects
    time_manager: Rc<RefCell<TimeManager<'a>>>,
    hub: Rc<RefCell<EventHub<'a>>>,
    trigger_event: Option<Arc<dyn Event>>,
}

impl<'a> BioConnector<'a> {
    pub fn new(initial_state: SimState, time_manager: Rc<RefCell<TimeManager<'a>>>, hub: Rc<RefCell<EventHub<'a>>>) -> BioConnector<'a> {
        BioConnector {
            local_state: initial_state,
            time_manager: time_manager,
            hub: hub,
            trigger_event: None,
        }
    }
    
    pub(super) fn set_trigger_event(&mut self, evt: Arc<dyn Event>) {
        self.trigger_event = Some(evt);
    }

    pub fn schedule_event<T: Event>(&self, wait_time: Time, evt: T) -> IdType {
        self.time_manager.borrow_mut().schedule_event(wait_time, evt)
    }

    pub fn unschedule_event(&self, schedule_id: IdType) -> Result<()> {
        self.time_manager.borrow_mut().unschedule_event(schedule_id)
    }

    pub fn get_time(&self) -> Time {
        self.time_manager.borrow().get_time()
    }

    pub fn get<T: Event>(&self) -> Option<&T> {
        self.local_state.get_state::<T>()
    }
    
    // pub fn get_Arc<T: Event>(&self) -> Option<Arc<T>> {
    //     match self.local_state.get_state_ref(&TypeId::of::<T>()) {
    //         None => None,
    //         Some(evt_rc) => {
    //             match evt_rc.downcast_arc::<T>() {
    //                 Err(_) => None,
    //                 Ok(typed_evt_rc) => {
    //                     Some(typed_evt_rc)
    //                 }
    //             }
    //         }
    //     }
    // }
}
