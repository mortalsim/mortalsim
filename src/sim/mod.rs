pub mod component;
pub mod layer;
pub mod sim_state;
pub mod time_manager;
pub mod organism;
pub use sim_state::SimState;
pub use time_manager::{SimTime, TimeManager};

pub use organism::*;

pub struct SimConnector {
    state: SimState,
    time_manager: TimeManager,
}

impl SimConnector {
    pub fn new() -> Self {
        SimConnector {
            state: SimState::new(),
            time_manager: TimeManager::new(),
        }
    }

    pub fn sim_time(&self) -> SimTime {
        self.time_manager.get_time()
    }
}

