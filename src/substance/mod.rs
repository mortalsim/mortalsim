mod change;
mod store;
mod substance;
mod concentration_tracker;
pub mod substance_wrapper;
use std::sync::OnceLock;

pub use change::SubstanceChange;
pub use store::SubstanceStore;
pub use substance::Substance;
pub use concentration_tracker::ConcentrationTracker;
use crate::units::chemical::Concentration;

pub type SubstanceConcentration = Concentration<f64>;
