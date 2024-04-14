pub mod organism;
pub mod component;
pub mod layer;
pub mod sim;
pub mod sim_state;
pub mod time_manager;
mod impl_sim;

use std::sync::Arc;

pub use sim::Sim;
pub use sim_state::SimState;
pub use time_manager::TimeManager;
pub use layer::Consumable;

pub use organism::{Organism, AnatomicalRegion};
pub use impl_sim::impl_sim;

pub use crate::{SimTime, SimTimeSpan};
use crate::event::Event;

pub struct SimConnector {
    pub state: SimState,
    pub time_manager: TimeManager,
    pub active_events: Vec<Arc<dyn Event>>,
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
