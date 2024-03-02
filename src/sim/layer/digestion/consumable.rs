
use either::Either;

use crate::sim::SimTime;
use crate::substance::substance_wrapper::substance_store_wrapper;
use crate::substance::{Substance, SubstanceConcentration, SubstanceStore};
use crate::units::base::{Amount, Mass};
use crate::units::geometry::Volume;
use crate::util::IdType;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::Sub;
use std::sync::{Mutex, RwLock};

/// An item to be consumed by a `Sim`'s digestive system
/// 
/// ```
/// use mortalsim::substance::{Substance, SubstanceStore};
/// use mortalsim::units::geometry::Volume;
/// use mortalsim::substance::SubstanceConcentration;
/// use mortalsim::sim::Consumable;
/// 
/// fn main() {
///     let mut bite1 = Consumable::new("Food".to_string(), Volume::from_mL(250.0));
///     bite1.set_volume_composition(Substance::Amylose, 0.15).unwrap();
///     bite1.set_volume_composition(Substance::Amylopectin, 0.65).unwrap();
///     bite1.set_volume_composition(Substance::Retinal, 0.01).unwrap();
///     bite1.set_volume_composition(Substance::Thiamine, 0.02).unwrap();
///     bite1.set_volume_composition(Substance::GLN, 0.001).unwrap();
///     bite1.set_volume_composition(Substance::PRO, 0.003).unwrap();

///     let bite2 = Consumable::new_custom(
///         bite1.name().to_string(),
///         Volume::from_mL(200.0),
///         Substance::H2O,
///         bite1.clone_store(),
///     );
/// }
/// 
/// ```
#[derive(Clone, Debug)]
pub struct Consumable {
    /// Name of the `Consumable``
    name: String,
    /// Solvent of the solution (water by default)
    solvent: Substance,
    /// Total volume of the `Consumable`
    volume: Volume<f64>,
    /// Total mass of the `Consumable`, calculated each time step
    mass: Mass<f64>,
    /// Volume of solutes in the solution
    solute_volume: Volume<f64>,
    /// For temporarily tracking composition percentages
    composition_parts: Vec<(Substance, f64)>,
    /// Store of substances in the `Consumable`
    pub(crate) store: SubstanceStore,
}

impl Consumable {
    pub fn new(name: String, volume: Volume<f64>) -> Self {
        Self::new_custom(name, volume, Substance::H2O, SubstanceStore::new())
    }

    pub fn new_with_solute(name: String, volume: Volume<f64>, solvent: Substance) -> Self {
        Self::new_custom(name, volume, solvent, SubstanceStore::new())
    }

    pub fn new_custom(name: String, volume: Volume<f64>, solvent: Substance, store: SubstanceStore) -> Self {
        Self {
            name: String::from(name),
            volume: volume,
            solvent: solvent,
            mass: solvent.density() * volume,
            solute_volume: Volume::from_L(0.0),
            composition_parts: Vec::new(),
            store: store,
        }
    }

    pub(crate) fn advance(&mut self, sim_time: SimTime) {
        self.store.advance(sim_time);
        self.mass = self.calc_mass();
        self.solute_volume = self.calc_volume_of_solutes();
    }

    fn calc_mass(&self) -> Mass<f64> {
        let mut calc_mass = Mass::from_g(0.0);
        for (substance, concentration) in self.store.get_composition().iter() {
            let amt_substance = concentration * self.volume();
            let mass_substance = amt_substance * substance.molar_mass();
            calc_mass += mass_substance;
        }
        let solvent_vol = self.volume - self.solute_volume;
        calc_mass += solvent_vol * self.solvent.density();
        calc_mass
    }

    fn calc_volume_of_solutes(&self) -> Volume<f64> {
        let mut solute_vol = Volume::from_L(0.0);
        for (substance, concentration) in self.store.get_composition().iter() {
            let part_vol = concentration * self.volume() * (substance.molar_mass() / substance.density());
            solute_vol += part_vol;
            print!("{:?}: {:?}", substance, part_vol)
        }
        solute_vol
    }

    fn check_solute_volume(&mut self) -> anyhow::Result<()> {
        if self.solute_volume > self.volume() {
            self.distill();
            self.mass = self.calc_mass();
            return Err(anyhow!(
                "Invalid volume (cannot fit solutes in {:?}), setting to minimum / distilled volume.",
                self.volume()
            ))
        }
        Ok(())
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn volume(&self) -> Volume<f64> {
        self.volume
    }

    pub fn mass(&self) -> Mass<f64> {
        self.mass
    }

    pub fn amount_of(&self, substance: &Substance) -> Amount<f64> {
        self.store.concentration_of(substance) * self.volume()
    }

    pub fn mass_of(&self, substance: &Substance) -> Mass<f64> {
        self.amount_of(substance) * substance.molar_mass()
    }

    pub fn volume_of(&self, substance: &Substance) -> Volume<f64> {
        self.amount_of(substance) * (substance.molar_mass() / substance.density())
    }

    pub fn set_volume(&mut self, volume: Volume<f64>) -> anyhow::Result<()> {
        if volume <= Volume::from_L(0.0) {
            return Err(anyhow!(
                "Consumable volume cannot be less than or equal to zero (set to {:?})",
                volume
            ));
        }
        self.volume = volume;
        self.solute_volume = self.calc_volume_of_solutes();
        self.check_solute_volume()
    }

    pub fn distill(&mut self) {
        self.volume = self.solute_volume
    }

    pub fn set_concentration(&mut self, substance: Substance, concentration: SubstanceConcentration) -> anyhow::Result<()> {
        let prev = self.store.concentration_of(&substance);
        let diff = concentration - prev;
        let part_vol = diff * self.volume() * (substance.molar_mass() / substance.density());
        self.solute_volume += part_vol;
        self.store.set_concentration(substance, concentration);
        self.check_solute_volume()
    }

    pub fn set_volume_composition(&mut self, substance: Substance, percent_volume: f64) -> anyhow::Result<()>{
        // gpcc / gpmol = molpcc
        let concentration = (substance.density() / substance.molar_mass()) * percent_volume;
        self.set_concentration(substance, concentration)
    }

    pub fn clone_store(&self) -> SubstanceStore {
        self.store.clone()
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
        let mut bite1 = Consumable::new("Food".to_string(), Volume::from_mL(250.0));
        bite1.set_volume_composition(Substance::Amylose, 0.15).unwrap();
        bite1.set_volume_composition(Substance::Amylopectin, 0.65).unwrap();
        bite1.set_volume_composition(Substance::Retinal, 0.01).unwrap();
        bite1.set_volume_composition(Substance::Thiamine, 0.02).unwrap();
        bite1.set_volume_composition(Substance::GLN, 0.001).unwrap();
        bite1.set_volume_composition(Substance::PRO, 0.003).unwrap();

        let bite2 = Consumable::new_custom(
            bite1.name().to_string(),
            Volume::from_mL(200.0),
            Substance::H2O,
            bite1.clone_store(),
        );

        let expected_val = 0.65 * bite2.volume();
        let threshold = Volume::from_nL(1.0);
        assert!(
            (expected_val-threshold..expected_val+threshold)
                .contains(&bite2.volume_of(&Substance::Amylopectin))
        );

    }
}
