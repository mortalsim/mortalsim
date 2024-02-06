pub mod component;
pub mod layer;
pub mod organism;
pub mod sim;
pub mod sim_state;
pub mod time_manager;
use std::sync::Arc;

pub use sim::Sim;
pub use sim_state::SimState;
pub use time_manager::{SimTime, TimeManager};

pub use organism::*;

use crate::event::Event;

pub struct SimConnector {
    state: SimState,
    time_manager: TimeManager,
    active_events: Vec<Box<dyn Event>>,
}

impl SimConnector {
    pub fn new() -> Self {
        SimConnector {
            state: SimState::new(),
            time_manager: TimeManager::new(),
            active_events: Vec::new(),
        }
    }

    pub fn sim_time(&self) -> SimTime {
        self.time_manager.get_time()
    }
}
