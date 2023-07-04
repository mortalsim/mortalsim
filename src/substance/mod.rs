mod change;
mod store;
mod substance;
pub use change::SubstanceChange;
pub use store::SubstanceStore;
pub use substance::Substance;
use crate::units::chemical::Concentration;

type SubstanceConcentration = Concentration<f64>;

lazy_static! {
    #[derive(Debug)]
    static ref ZERO_CONCENTRATION: SubstanceConcentration = Concentration::from_M(0.0);
}
