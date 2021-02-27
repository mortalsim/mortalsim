use std::rc::Rc;
use crate::core::sim::SimState;
use crate::core::hub::EventHub;
use crate::core::sim::TimeManager;
use crate::event::Event;

pub struct BioConnector<'a> {
    state: Rc<SimState>,
    hub: Rc<EventHub<'a>>,
    time_manager: Rc<TimeManager<'a>>,
}

impl<'a> BioConnector<'a> {
    pub fn get<T: Event>(&self) -> Option<&T> {
        self.state.get_state::<T>()
    }
}