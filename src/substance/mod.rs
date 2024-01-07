mod change;
mod store;
mod substance;
mod concentration_tracker;
pub mod substance_wrapper;
pub use change::SubstanceChange;
pub use store::SubstanceStore;
pub use substance::Substance;
pub use concentration_tracker::ConcentrationTracker;
use crate::units::chemical::Concentration;

pub type SubstanceConcentration = Concentration<f64>;


lazy_static! {
    #[derive(Debug)]
    static ref ZERO_CONCENTRATION: SubstanceConcentration = Concentration::from_M(0.0);
}
