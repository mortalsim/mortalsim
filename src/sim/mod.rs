use std::sync::{Arc, Mutex};

mod component;
mod layer;
mod sim_state;
mod time_manager;
pub use sim_state::SimState;
pub use time_manager::{Time, TimeManager};

pub struct SimConnector {
    sim_time: Time,
    time_manager: TimeManager,
    state: Arc<Mutex<SimState>>,
}
