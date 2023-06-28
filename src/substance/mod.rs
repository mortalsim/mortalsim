use uom::si::molar_concentration::millimole_per_liter;
mod change;
mod store;
mod substance;
pub use change::SubstanceChange;
pub use store::SubstanceStore;
pub use substance::Substance;
pub use uom::si::f64::*;

lazy_static! {
    #[derive(Debug)]
    static ref ZERO_CONCENTRATION: MolarConcentration = MolarConcentration::new::<millimole_per_liter>(0.0);
}
