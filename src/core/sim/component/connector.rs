use std::rc::{Rc, Weak};
use std::collections::HashMap;
use std::cell::{Ref, RefCell, Cell};
use std::any::TypeId;
use std::io;
use std::convert::From;
use anyhow::{Error, Result};
use crate::util::id_gen::IdType;
use crate::core::sim::SimState;
use crate::core::hub::EventHub;
use crate::core::sim::TimeManager;
use crate::event::Event;

pub struct InitBioConnector {
    pub(in super::super) local_state: SimState,
    pub(in super::super) notify_events: HashMap<TypeId, i32>,
}

impl<'a> InitBioConnector {
    pub fn new() -> InitBioConnector {
        InitBioConnector {
            local_state: SimState::new(),
            notify_events: HashMap::new(),
        }
    }

    pub fn emit_initial<T: Event>(&mut self, evt: T) {
        self.local_state.set_state(evt);
    }

    pub fn notify_on<T: Event>(&mut self, default: T) {
        self.local_state.set_state_quiet(default);
        self.notify_events.insert(TypeId::of::<T>(), 0);
    }
    
    pub fn notify_prioritized_on<T: Event>(&mut self, default: T, priority: i32) {
        self.local_state.set_state_quiet(default);
        self.notify_events.insert(TypeId::of::<T>(), priority);
    }
}

pub struct BioConnector<'a> {
    pub(in super::super) local_state: SimState,
    time_manager: Rc<RefCell<TimeManager<'a>>>,
    hub: Rc<RefCell<EventHub<'a>>>,
    trigger_event: Option<Rc<dyn Event>>,
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

    pub(super) fn set_trigger_event(&mut self, evt: Rc<dyn Event>) {
        self.trigger_event = Some(evt);
    }

    pub fn get<T: Event>(&self) -> Option<&T> {
        self.local_state.get_state::<T>()
    }
}
