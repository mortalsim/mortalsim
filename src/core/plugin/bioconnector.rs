use std::rc::{Rc, Weak};
use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::cell::{Ref, RefCell, Cell};
use std::any::TypeId;
use anyhow::{Error, Result};
use crate::util::id_gen::IdType;
use crate::core::sim::SimState;
use crate::core::hub::EventHub;
use crate::core::sim::TimeManager;
use crate::event::Event;

pub struct BioConnector<'a> {
    local_state: SimState,
    notify_events: HashSet<TypeId>,
    time_manager: Rc<RefCell<TimeManager<'a>>>,
    hub: Rc<RefCell<EventHub<'a>>>,
    active: bool,
}

impl<'a> BioConnector<'a> {
    pub fn new(time_manager: Rc<RefCell<TimeManager<'a>>>, hub: Rc<RefCell<EventHub<'a>>>) -> BioConnector<'a> {
        BioConnector {
            local_state: SimState::new(),
            notify_events: HashSet::new(),
            time_manager: time_manager,
            hub: hub,
            active: false,
        }
    }

    pub(super) fn check_notify(&self) -> bool {
        !self.notify_events.is_disjoint(self.local_state.get_tainted())
    }

    pub fn emit_initial<T: Event>(&mut self, evt: T) {
        if !self.active {
            self.hub.borrow_mut().emit(evt);
        }
        panic!("emit_initial can only be called during init!");
    }

    pub fn notify_on<T: Event>(&mut self, default: T) {
        self.local_state.set_state(default);
        self.notify_events.insert(TypeId::of::<T>());
    }

    pub fn get<T: Event>(&self) -> Option<&T> {
        self.local_state.get_state::<T>()
    }
}
