use std::rc::{Rc, Weak};
use std::collections::hash_map::HashMap;
use std::cell::{Ref, RefCell, Cell};
use std::any::TypeId;
use crate::util::id_gen::IdType;
use crate::core::sim::SimState;
use crate::core::hub::EventHub;
use crate::core::sim::TimeManager;
use crate::event::Event;

pub struct BioConnector {
    local_state: SimState,
}

impl BioConnector {
    pub fn new() -> BioConnector {
        BioConnector {
            local_state: SimState::new(),
        }
    }

    pub fn get<T: Event>(&self) -> Option<&T> {
        self.local_state.get_state::<T>()
    }
}