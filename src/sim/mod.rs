use std::sync::{Arc, Mutex};

pub mod component;
pub mod layer;
pub mod sim_state;
pub mod time_manager;
pub mod organism;
pub use sim_state::SimState;
pub use time_manager::{SimTime, TimeManager};

pub use organism::*;

pub struct SimConnector {
    sim_time: SimTime,
    time_manager: TimeManager,
    state: Arc<Mutex<SimState>>,
}

impl SimConnector {
    pub fn new() -> Self {
        SimConnector {
            sim_time: SimTime::from_s(0.0),
            time_manager: TimeManager::new(),
            state: Arc::new(Mutex::new(SimState::new())),
        }
    }
}
