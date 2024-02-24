use simple_si_units::base::Amount;

use crate::sim::SimTime;
use crate::substance::substance_wrapper::substance_store_wrapper;
use crate::substance::{Substance, SubstanceStore};
use crate::units::geometry::Volume;
use crate::util::IdType;
use std::collections::HashMap;

/// An item to be consumed by a `Sim`'s digestive system
/// 
/// ```
/// use crate::substance::{Substance, SubstanceStore};
/// use crate::units::geometry::Volume;
/// use crate::sim::Consumable;
/// 
/// fn main() {
///     let store = SubstanceStore::new();
/// }
/// 
/// ```
#[derive(Clone, Debug)]
pub struct Consumable {
    /// Name of the `Consumable``
    name: String,
    /// Total volume of the `Consumable`
    volume: Volume<f64>,
    /// Store of substances in the `Consumable`
    pub(super) store: SubstanceStore,
}

impl Consumable {
    pub fn new(name: String, volume: Volume<f64>, store: SubstanceStore) -> Consumable {
        Consumable {
            name: name,
            volume: volume,
            store: store,
        }
    }

    pub fn advance(&mut self, sim_time: SimTime) {
        self.store.advance(sim_time)
    }

    pub fn volume(&self) -> Volume<f64> {
        self.volume
    }

    pub fn amount_of(&self, substance: Substance) -> Amount<f64> {
        self.store.concentration_of(&substance) * self.volume
    }

    pub fn set_volume(&mut self, volume: Volume<f64>) -> anyhow::Result<()> {
        if volume <= Volume::from_L(0.0) {
            return Err(anyhow!(
                "Consumable volume cannot be less than zero (set to {:?})",
                volume
            ));
        }
        self.volume = volume;
        Ok(())
    }
}

#[cfg(test)]
pub mod test {
    use crate::sim::SimTime;
    use crate::substance::{Substance, SubstanceConcentration, SubstanceStore};
    use crate::units::geometry::Volume;
    use crate::util::{mmol_per_L, secs};

    use super::Consumable;

    #[test]
    fn test_new_consumable() {
        let mut store = SubstanceStore::new();
        store.set_concentration(Substance::Retinal, SubstanceConcentration::from_nM(0.349));
        store.set_concentration(Substance::Thiamine, SubstanceConcentration::from_nM(0.119));
        store.set_concentration(Substance::GLN, SubstanceConcentration::from_uM(57.0));
        store.set_concentration(Substance::PRO, SubstanceConcentration::from_uM(26.1));
        Consumable::new(String::new(), Volume::from_L(0.5), SubstanceStore::new());
    }
}
