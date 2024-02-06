mod change;
mod concentration_tracker;
mod store;
mod substance;
pub mod substance_wrapper;
use std::sync::OnceLock;

use crate::units::chemical::Concentration;
pub use change::SubstanceChange;
pub use concentration_tracker::ConcentrationTracker;
pub use store::SubstanceStore;
pub use substance::Substance;

pub type SubstanceConcentration = Concentration<f64>;
