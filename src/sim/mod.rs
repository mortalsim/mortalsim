use std::sync::{Arc, Mutex};

mod component;
mod layer;
mod sim_state;
mod time_manager;
pub use sim_state::SimState;
pub use time_manager::{SimTime, TimeManager};

pub struct SimConnector {
    sim_time: SimTime,
    time_manager: TimeManager,
    state: Arc<Mutex<SimState>>,
}
