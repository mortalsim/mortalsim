
use either::Either;
use log::Log;

use crate::sim::SimTime;
use crate::substance::substance_wrapper::substance_store_wrapper;
use crate::substance::{Substance, SubstanceConcentration, SubstanceStore};
use crate::units::base::{Amount, Mass};
use crate::units::geometry::Volume;
use crate::util::{BoundFn, IdGenerator, IdType};
use std::borrow::Borrow;
use std::collections::{HashMap, VecDeque};
use std::ops::Sub;
use std::sync::{Mutex, RwLock};

#[derive(Debug, Clone)]
pub struct VolumeChange {
    id: IdType,
    amount: Volume<f64>,
    function: BoundFn,
    start: SimTime,
    end: SimTime,
}

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
    /// For temporarily tracking composition percentages
    composition_parts: Vec<(Substance, f64)>,
    /// Id generator for volume change registrations
    id_gen: IdGenerator,
    /// Volume changes
    volume_changes: HashMap<IdType, VolumeChange>,
    /// composite changes (volume & substance)
    composite_changes: HashMap<IdType, (IdType, IdType)>,
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
            composition_parts: Vec::new(),
            id_gen: IdGenerator::new(),
            volume_changes: HashMap::new(),
            composite_changes: HashMap::new(),
            store: store,
        };

        if volume <= Volume::from_L(0.0) {
            panic!("Consumable volume must be a positive, non-zero value!");
        }
        item.mass = item.calc_mass();
        item
    }

    /// Advance simulation time to the given value.
    ///
    /// Volume changes are executed before the store advances.
    /// Total mass is calculated at each step of the simulation. 
    pub(crate) fn advance(&mut self, sim_time: SimTime) {
        self.execute_volume_changes(sim_time);
        self.store.advance(sim_time);
        self.mass = self.calc_mass();
    }

    /// Internal execution of volume changes on each advance
    fn execute_volume_changes(&mut self, sim_time: SimTime) {
        let mut remove_list = Vec::new();
        for (cid, change) in self.volume_changes.iter() {
            if change.start < sim_time && change.end > sim_time {
                let result = change.function.call(
                    sim_time.s - change.start.s,
                    change.end.s - change.start.s,
                    change.amount.m3,
                );
                // Make sure the volume change is valid, and log a warning if it's not
                let new_vol = self.volume() + Volume::from_m3(result);
                if new_vol <= Volume::from_L(0.0) {
                    log::warn!("Scheduled volume change attempted to set invalid volume: {}", new_vol);
                    continue;
                }
                self.volume = new_vol;
            }
            if change.end < sim_time {
                remove_list.push(*cid);
            }
        }
        // Remove any volume changes which have completed
        for cid in remove_list {
            self.volume_changes.remove(&cid);
        }
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
        let solvent_vol = self.volume*self.store.solvent_pct();
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

    /// Total volume of the solution
    pub fn volume(&self) -> Volume<f64> {
        self.volume
    }

    /// Total mass of the solution
    pub fn mass(&self) -> Mass<f64> {
        self.mass
    }

    /// Concentration of the given substance in the solution
    pub fn concentration_of(&self, substance: &Substance) -> SubstanceConcentration {
        self.store.concentration_of(substance)
    }

    /// Amount of the given substance in the solution
    pub fn amount_of(&self, substance: &Substance) -> Amount<f64> {
        if substance == &self.solvent {
            return (self.volume()*self.store.solvent_pct()) / self.solvent.molar_volume()
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

    /// Schedules a future change in solution volume with a sigmoidal shape.
    ///
    /// Note that all substance and volume changes will be
    /// unset when the `Consumed` moves on to the next component.
    ///
    /// ### Arguments
    /// * `amount`   - magnitude of the volume change
    /// * `start`    - simulation time to start the change
    /// * `end`      - simulation time to end the change
    ///
    /// Returns an id corresponding to the change
    pub fn schedule_volume_change(&mut self,
        amount: Volume<f64>,
        start: SimTime,
        end: SimTime,
        ) -> IdType {
            self.schedule_custom_volume_change(amount, start, end, BoundFn::Sigmoid)
    }

    /// Schedules a future change in solution volume with a custom shape.
    ///
    /// Note that all substance and volume changes will be
    /// unset when the `Consumed` moves on to the next component.
    ///
    /// ### Arguments
    /// * `amount`   - magnitude of the volume change
    /// * `start`    - simulation time to start the change
    /// * `end`      - simulation time to end the change
    /// * `bound_fn` - the shape of the function
    ///
    /// Returns an id corresponding to the change
    pub fn schedule_custom_volume_change(&mut self,
        amount: Volume<f64>,
        start: SimTime,
        end: SimTime,
        bound_fn: BoundFn,
        ) -> IdType {
            let change_id = self.id_gen.get_id();
            self.volume_changes.insert(change_id, VolumeChange {
                id: change_id,
                amount: amount,
                function: bound_fn,
                start: start,
                end: end,
            });
            change_id
    }

    /// Unschedules a previously scheduled volume change the `Consumable`
    /// 
    /// ### Arguments
    /// * `change_id`` - change id returned from a previous volume change schedule
    ///
    /// Will return an Err if the change_id is invalid
    pub fn unschedule_volume_change(&mut self, change_id: IdType) -> Option<VolumeChange> {
        self.volume_changes.remove(&change_id)
    }

    /// Sets the volume of the `Consumable`
    ///
    /// Volume is checked for validity. If the given value is less than
    /// the total volume of solutes in the solution, an Err will be returned
    pub fn set_volume(&mut self, volume: Volume<f64>) -> anyhow::Result<()> {
        if volume <= Volume::from_L(0.0) {
            return Err(anyhow!("Volume must be a positive numeric value"));
        }
        self.volume = volume;
        Ok(())
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
        let vol_change = diff * self.volume() * substance.molar_volume();

        // Calculate mass change based on the amount of solvent displaced
        self.mass += vol_change*substance.density() - vol_change*self.solvent.density();

        self.store.set_concentration(substance, concentration)
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
    fn consumable() {
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
    #[should_panic]
    fn zero_consumable() {
        Consumable::new(Volume::from_mL(0.0));
    }

    #[test]
    fn bad_composition() {
        let mut bite1 = Consumable::new(Volume::from_mL(250.0));
        bite1.set_volume_composition(Substance::Amylose, 0.15).unwrap();
        bite1.set_volume_composition(Substance::Amylopectin, 0.65).unwrap();
        // Oops, went over 100%
        assert!(bite1.set_volume_composition(Substance::Retinal, 0.30).is_err());
    }
    
    #[test]
    fn set_concentration() {
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

    #[test]
    fn bad_concentration() {
        // Should Err because 200 M of GLC can't fit in 250mL of Solution
        let mut sugar = Consumable::new(Volume::from_mL(250.0));
        assert!(sugar.set_concentration(Substance::GLC, SubstanceConcentration::from_M(200.0)).is_err());
    }

    #[test]
    fn volume_change() {
        let mut sugar = Consumable::new(Volume::from_mL(250.0));
        sugar.set_concentration(Substance::GLC, mmol_per_L!(3.0)).unwrap();

        let orig_volume = sugar.volume_of(&Substance::GLC);

        // Concentrations should remain the same after a change in volume
        sugar.set_volume(Volume::from_mL(300.0)).unwrap();

        let expected_conc = mmol_per_L!(3.0);
        let threshold = SubstanceConcentration::from_nM(1.0);
        assert!(
            (expected_conc-threshold..expected_conc+threshold).contains(
                &sugar.concentration_of(&Substance::GLC)
            )
        );

        assert!(sugar.volume_of(&Substance::GLC) > orig_volume);
    }

    #[test]
    fn bad_volume_change() {
        let mut sugar = Consumable::new(Volume::from_mL(250.0));
        sugar.set_concentration(Substance::GLC, mmol_per_L!(3.0)).unwrap();

        assert!(sugar.set_volume(Volume::from_mL(0.0)).is_err());
        assert!(sugar.set_volume(Volume::from_mL(-1.0)).is_err());
    }
}
