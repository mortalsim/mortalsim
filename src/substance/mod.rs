
use uom::si::molar_concentration::millimole_per_liter;
mod substance;
mod store;
mod change;
pub use substance::Substance;
pub use store::SubstanceStore;
pub use change::SubstanceChange;
pub use uom::si::f64::*;

lazy_static! {
    #[derive(Debug)]
    static ref ZERO_CONCENTRATION: MolarConcentration = MolarConcentration::new::<millimole_per_liter>(0.0);
}
