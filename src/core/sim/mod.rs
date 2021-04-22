mod time_manager;
mod sim_state;
pub mod component;
pub mod sim;
pub use component::{SimComponentInitializer, SimConnector, SimComponent};
pub use time_manager::{TimeManager, Time};
pub use sim_state::SimState;
pub use sim::Sim;

pub trait HumanSim {}
pub struct Human {}
impl HumanSim for Human {}

pub enum SimType {
    Human(Human)
}