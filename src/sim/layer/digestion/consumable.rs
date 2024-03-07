
use either::Either;
use log::Log;

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

/// A homogeneous chemical solution to be consumed by a `Sim`'s
/// digestive system
/// 
/// ```
/// use mortalsim::substance::{Substance, SubstanceStore};
/// use mortalsim::units::geometry::Volume;
/// use mortalsim::substance::SubstanceConcentration;
/// use mortalsim::sim::Consumable;
/// 
/// fn main() {
///     // Create our first bite of 250mL, with water (default) as the solvent
///     let mut bite1 = Consumable::new(Volume::from_mL(250.0));
///
///     // Add some starch
///     bite1.set_volume_composition(Substance::Amylose, 0.15).unwrap();
///     bite1.set_volume_composition(Substance::Amylopectin, 0.65).unwrap();
///
///     // A couple of vitamins
///     bite1.set_volume_composition(Substance::Retinal, 0.01).unwrap();
///     bite1.set_volume_composition(Substance::Thiamine, 0.02).unwrap();
///
///     // Some proteins
///     bite1.set_volume_composition(Substance::GLN, 0.001).unwrap();
///     bite1.set_volume_composition(Substance::PRO, 0.003).unwrap();
///
///     // create a second bite with a different volume, but the same
///     // composition, by cloning the store of solutes
///     let bite2 = Consumable::new_custom(
///         Volume::from_mL(200.0),
///         Substance::H2O,
///         bite1.store().clone(),
///     );
/// }
/// 
/// ```
#[derive(Clone, Debug)]
pub struct Consumable {
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
    /// Create a new pure water Consumable with given `Volume`.
    ///
    /// ### Arguments
    /// * `volume` - Initial volume of the solution
    pub fn new(volume: Volume<f64>) -> Self {
        Self::new_custom(volume, Substance::H2O, SubstanceStore::new())
    }

    /// Create a new Consumable with given `Volume` and solvent
    ///
    /// ### Arguments
    /// * `volume` - Initial volume of the solution
    /// * `solvent` - Base in which solutes are dissolved to form a solution
    pub fn new_with_solvent(volume: Volume<f64>, solvent: Substance) -> Self {
        Self::new_custom(volume, solvent, SubstanceStore::new())
    }

    /// Create a new Consumable with given `Volume`, solvent, and initial set
    /// of solutes
    ///
    /// ### Arguments
    /// * `volume` - Initial volume of the solution
    /// * `solvent` - Base in which solutes are dissolved to form a solution
    /// * `store` - `SubstanceStore` of solutes
    pub fn new_custom(volume: Volume<f64>, solvent: Substance, store: SubstanceStore) -> Self {
        let mut item = Self {
            volume: volume,
            solvent: solvent,
            mass: solvent.density() * volume,
            solute_volume: Volume::from_L(0.0),
            composition_parts: Vec::new(),
            store: store,
        };

        item.calc();
        item
    }

    fn calc(&mut self) {
        self.solute_volume = self.calc_volume_of_solutes();
        // If volume has become too small to fit the solutes
        // log as a warning and continue on
        if let Err(e) = self.check_solute_volume() {
            log::warn!("{}", e);
        }
        self.mass = self.calc_mass();
    }

    /// Advance simulation time to the given value.
    ///
    /// Total mass and solute volume are determined at each step
    /// of the simulation. 
    pub(crate) fn advance(&mut self, sim_time: SimTime) {
        self.store.advance(sim_time);
        self.calc();
    }

    /// Calculates the total mass of the solution based on the current
    /// chemical composition
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

    /// Calculates the total volume of all solutes in the solution
    fn calc_volume_of_solutes(&self) -> Volume<f64> {
        let mut solute_vol = Volume::from_L(0.0);
        for (substance, concentration) in self.store.get_composition().iter() {
            let part_vol = concentration * self.volume() * substance.molar_volume();
            solute_vol += part_vol;
        }
        solute_vol
    }

    /// Checks the validity of total solute volume (must be less than total volume)
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

    /// Total volume of the solution
    pub fn volume(&self) -> Volume<f64> {
        self.volume
    }

    /// Total mass of the solution
    pub fn mass(&self) -> Mass<f64> {
        self.mass
    }

    /// Amount of the given substance in the solution
    pub fn amount_of(&self, substance: &Substance) -> Amount<f64> {
        if substance == &self.solvent {
            return (self.volume() - self.solute_volume) / self.solvent.molar_volume()
        }
        self.store.concentration_of(substance) * self.volume()
    }

    /// Mass of the given substance in the solution
    pub fn mass_of(&self, substance: &Substance) -> Mass<f64> {
        self.amount_of(substance) * substance.molar_mass()
    }

    /// Volume of the given substance in the solution
    pub fn volume_of(&self, substance: &Substance) -> Volume<f64> {
        self.amount_of(substance) * substance.molar_volume()
    }

    /// Sets the volume of the `Consumable` without any additional checks
    pub(crate) fn set_volume_unchecked(&mut self, volume: Volume<f64>) {
        self.volume = volume;
    }

    /// Sets the volume of the `Consumable`
    ///
    /// Volume is checked for validity. If the given value is less than
    /// the total volume of solutes in the solution, an Err will be returned
    pub fn set_volume(&mut self, volume: Volume<f64>) -> anyhow::Result<()> {
        self.volume = volume;
        self.solute_volume = self.calc_volume_of_solutes();
        self.check_solute_volume()
    }

    /// Distills the solution.
    ///
    /// Effectively removes the solvent, creating a "dry" solution.
    /// Note that solvent will be reintroduced to fill the vacant space
    /// if solutes are removed.
    pub fn distill(&mut self) {
        self.volume = self.solute_volume
    }

    /// Sets the concentration of a given Substance in the solution.
    ///
    /// Validity of the solution volume is checked against the change. If
    /// the new total solute volume is invalid, an Err will be returned
    /// and the volume of the solution will be set to the minimum "dry"
    /// value based on the current composition.
    ///
    /// ### Arguments
    /// * `substance` - Substance to set the concentration for
    /// * `concentration` - concentration to set for the Substance
    pub fn set_concentration(&mut self, substance: Substance, concentration: SubstanceConcentration) -> anyhow::Result<()> {
        let prev = self.store.concentration_of(&substance);
        let diff = concentration - prev;

        // calculate change in solute volume
        let part_vol = diff * self.volume() * substance.molar_volume();
        self.solute_volume += part_vol;

        // Calculate mass change based on the amount of solvent displaced
        self.mass += part_vol*substance.density() - part_vol*self.solvent.density();

        self.store.set_concentration(substance, concentration);
        self.check_solute_volume()
    }

    /// Sets the concentration of a given Substance in the solution based
    /// on the percent volume of that Substance.
    ///
    /// Validity of the solution volume is checked against the change. If
    /// the new total solute volume is invalid, an Err will be returned
    /// and the volume of the solution will be set to the minimum "dry"
    /// value based on the current composition.
    ///
    /// ### Arguments
    /// * `substance` - Substance to set the concentration for
    /// * `concentration` - concentration to set for the Substance
    pub fn set_volume_composition(&mut self, substance: Substance, percent_volume: f64) -> anyhow::Result<()>{
        // gpcc / gpmol = molpcc
        let concentration = percent_volume / substance.molar_volume();
        self.set_concentration(substance, concentration)
    }

    pub fn store(&self) -> &SubstanceStore {
        &self.store
    }

}

#[cfg(test)]
pub mod test {
    use crate::sim::SimTime;
    use crate::substance::{Substance, SubstanceConcentration, SubstanceStore};
    use crate::units::geometry::Volume;
    use crate::units::base::Mass;
    use crate::util::{mmol_per_L, secs};

    use super::Consumable;

    #[test]
    fn test_consumable() {
        let mut bite1 = Consumable::new(Volume::from_mL(250.0));
        bite1.set_volume_composition(Substance::Amylose, 0.15).unwrap();
        bite1.set_volume_composition(Substance::Amylopectin, 0.65).unwrap();
        bite1.set_volume_composition(Substance::Retinal, 0.01).unwrap();
        bite1.set_volume_composition(Substance::Thiamine, 0.02).unwrap();
        bite1.set_volume_composition(Substance::GLN, 0.001).unwrap();
        bite1.set_volume_composition(Substance::PRO, 0.003).unwrap();


        let expected_mass =
            bite1.mass_of(&Substance::Amylose) + 
            bite1.mass_of(&Substance::Amylopectin) + 
            bite1.mass_of(&Substance::Retinal) + 
            bite1.mass_of(&Substance::Thiamine) + 
            bite1.mass_of(&Substance::GLN) + 
            bite1.mass_of(&Substance::PRO) +
            bite1.mass_of(&Substance::H2O);

        let threshold_m = Mass::from_pg(1.0);
        assert!(
            (expected_mass-threshold_m..expected_mass+threshold_m)
                .contains(&bite1.mass()),
            "{} != {}", expected_mass, bite1.mass()
        );
        
        let bite2 = Consumable::new_custom(
            Volume::from_mL(200.0),
            Substance::H2O,
            bite1.store().clone(),
        );

        let expected_vol = 0.65 * bite2.volume();
        let threshold = Volume::from_nL(1.0);
        assert!(
            (expected_vol-threshold..expected_vol+threshold)
                .contains(&bite2.volume_of(&Substance::Amylopectin)),
            "{} != {}", expected_vol, bite2.volume()
        );

    }
    
    #[test]
    fn test_consumable_set_concentration() {
        let mut sugar = Consumable::new(Volume::from_mL(250.0));
        let original_mass = sugar.mass();

        sugar.set_concentration(Substance::GLC, mmol_per_L!(3.0)).unwrap();

        let displaced_volume = (mmol_per_L!(3.0)*Volume::from_mL(250.0))*Substance::GLC.molar_volume();
        let expected_mass_change = displaced_volume*Substance::GLC.density() - displaced_volume*Substance::H2O.density();
        let expected_mass = original_mass + expected_mass_change;
        let threshold_m = Mass::from_pg(1.0);
        assert!(
            (expected_mass-threshold_m..expected_mass+threshold_m)
                .contains(&sugar.mass()),
            "{} != {}", expected_mass, sugar.mass()
        );
    }
}
