use super::change::DependentSubstanceChange;
use super::{SubstanceChange, SubstanceConcentration};
use crate::sim::SimTime;
use crate::substance::Substance;
use crate::id_gen::{IdGenerator, IdType};
use crate::math::BoundFn;
use crate::{secs, SimTimeSpan};
use core::panic;
use std::collections::HashMap;
use std::fmt;
use std::mem::swap;
use std::sync::OnceLock;

static ZERO_CONCENTRATION: OnceLock<SubstanceConcentration> = OnceLock::new();

#[derive(Clone)]
/// A storage construct for Substance concentrations in a volume
pub struct SubstanceStore {
    /// Id generator for substance changes
    id_gen: IdGenerator,
    /// Local cache of simulation time
    sim_time: SimTime,
    /// Data structure containing the internal substance concentration
    composition: HashMap<Substance, SubstanceConcentration>,
    /// Keep track of any Substances which are changing
    substance_changes: HashMap<Substance, HashMap<IdType, SubstanceChange>>,
    /// Keep track of any Substances which are changing
    dependent_changes: HashMap<Substance, Vec<DependentSubstanceChange>>,
    /// Keep track of staged changes, which will be "new" on the next advance
    staged_changes: HashMap<Substance, IdType>,
    /// Keep track of newly added change ids
    new_changes: HashMap<Substance, IdType>,
    /// Keep track of the solute percentage to ensure validity
    solute_pct: f64,
    /// whether to track new changes or not
    track_changes: bool,
}

impl fmt::Debug for SubstanceStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SubstanceStore {{composition = {:?}, substance_changes = {:?}}}",
            self.composition, self.substance_changes
        )?;
        Ok(())
    }
}

impl Default for SubstanceStore {
    fn default() -> Self {
        Self::new()
    }
}

static DEFAULT_STORE: OnceLock<SubstanceStore> = OnceLock::new();

impl<'a> Default for &'a SubstanceStore {
    fn default() -> Self {
        DEFAULT_STORE.get_or_init(|| SubstanceStore::new())
    }
}

impl SubstanceStore {
    fn zero_concentration() -> &'static SubstanceConcentration {
        ZERO_CONCENTRATION.get_or_init(|| SubstanceConcentration::from_M(0.0))
    }

    /// Constructs a new Substance store with the given identifier and initial volume
    ///
    /// ### Arguments
    /// * `volume` - initial volume
    pub fn new() -> SubstanceStore {
        SubstanceStore {
            id_gen: IdGenerator::new(),
            sim_time: secs!(0.0),
            composition: HashMap::new(),
            substance_changes: HashMap::new(),
            dependent_changes: HashMap::new(),
            staged_changes: HashMap::new(),
            new_changes: HashMap::new(),
            solute_pct: 0.0,
            track_changes: false,
        }
    }

    /// Retrieves the current simulation time for the store.
    pub fn sim_time(&self) -> SimTime {
        self.sim_time
    }

    /// Retrieves the total percent volume of solutes in the solution
    pub fn solute_pct(&self) -> f64 {
        self.solute_pct
    }
    
    /// Retrieves the total percent volume of solvent in the solution
    pub fn solvent_pct(&self) -> f64 {
        1.0 - self.solute_pct
    }

    /// Retrieves the concentration of a given Substance in the store.
    ///
    /// ### Arguments
    /// * `substance` - Substance to retrieve
    ///
    /// Returns the current concentration of the substance
    pub fn concentration_of(&self, substance: &Substance) -> SubstanceConcentration {
        match self.composition.get(substance) {
            None => *Self::zero_concentration(),
            Some(amt) => *amt,
        }
    }

    /// sets the concentration of a given Substance in the store.
    ///
    /// ### Arguments
    /// * `substance` - Substance to set the concentration for
    /// * `concentration` - concentration to set for the Substance
    pub fn set_concentration(
        &mut self,
        substance: Substance,
        concentration: SubstanceConcentration,
    ) -> anyhow::Result<()> {
        if concentration < SubstanceConcentration::from_M(0.0) {
            return Err(anyhow!("Concentration must be a positive value."));
        }
        let concentration_change = concentration - self.concentration_of(&substance);
        let pct_change = concentration_change.molpm3*substance.molar_volume().m3_per_mol;
        if self.solute_pct + pct_change > 1.0 {
            return Err(anyhow!("Invalid concentration. Setting {} of {} increases the total solute concentration to greater than 100%.",
                concentration,
                substance,
            ))
        }
        self.solute_pct += pct_change;
        self.composition.insert(substance, concentration);
        Ok(())
    }

    /// Retrieves the current composition as a HashMap
    ///
    /// ### Arguments
    /// * `composition` - the Substance composition to merge
    pub(crate) fn get_composition(&self) -> &HashMap<Substance, simple_si_units::chemical::Concentration<f64>>{
        &self.composition
    }

    /// Merges the provided composition with this store's internal composition, updating
    /// any existing substances and adding any new concentrations
    ///
    /// ### Arguments
    /// * `composition` - the Substance composition to merge
    pub(crate) fn merge_composition(&mut self, composition: &HashMap<Substance, SubstanceConcentration>) {
        self.composition.extend(composition);
    }

    /// Merges the target store's composition with this store's internal composition, updating
    /// any existing substances and adding any new concentrations
    ///
    /// ### Arguments
    /// * `other` - the SubstanceStore to merge
    pub fn merge_from(&mut self, other: &SubstanceStore) {
        self.composition.extend(other.composition.clone());
    }

    /// Schedule a substance change on this store
    /// with a custom shape over the given duration.
    ///
    /// Panics if `start_time < sim_time` or `duration <= 0`
    ///
    /// ### Arguments
    /// * `substance`  - the substance to change
    /// * `start_time` - simulation time to start the change
    /// * `amount`     - total concentration change to take place
    /// * `duration`   - amount of time over which the change takes place
    /// * `bound_fn`   - the shape of the function
    ///
    /// Returns an id corresponding to this change
    pub fn schedule_change(
        &mut self,
        substance: Substance,
        amount: SubstanceConcentration,
        start_time: SimTime,
        duration: SimTimeSpan,
        bound_fn: BoundFn,
    ) -> IdType {
        // Constrain the start time to a minimum of the current sim time
        if start_time < self.sim_time {
            panic!("start_time cannot be less than the current sim time!");
        }

        let change_id = self.id_gen.get_id();
        let change = SubstanceChange::new(start_time, amount, duration, bound_fn);
        self.substance_changes
            .entry(substance)
            .or_default()
            .insert(change_id, change);
        
        if self.track_changes {
            self.staged_changes.insert(substance, change_id);
        }

        change_id
    }

    /// Schedule a dependent substance change on this store
    /// equal to a change on a different store with a given delay.
    ///
    /// Panics if `start_time < sim_time` or `start_time <= change.start_time()`
    ///
    /// ### Arguments
    /// * `substance`  - the substance to change
    /// * `start_time` - simulation time to start the change
    /// * `change`     - change to duplicate on this store
    ///
    /// Returns an id corresponding to this change
    pub fn schedule_dependent_change(
        &mut self,
        substance: Substance,
        start_time: SimTime,
        change: &SubstanceChange,
    ) {
        // Constrain the start time to a minimum of the current sim time
        if start_time < self.sim_time {
            panic!("start_time cannot be less than the current sim time!");
        }

        let dep_change = DependentSubstanceChange::new(start_time, change);
        self.dependent_changes
            .entry(substance)
            .or_default()
            .push(dep_change);

    }

    /// Get a reference to a previously added `SubstanceChange`
    ///
    /// ### Arguments
    /// * `change_id`  - change id returned previously
    ///
    /// Returns a reference to the `SubstanceChange`
    pub fn get_substance_change<'a>(
        &'a self,
        substance: &Substance,
        change_id: &IdType,
    ) -> Option<&'a SubstanceChange> {
        self.substance_changes.get(substance)?.get(change_id)
    }

    /// Get an iterator to all previously added `SubstanceChange`s
    /// Does not include any attached dependent changes
    ///
    /// Returns an iterator of all `SubstanceChange`s
    pub fn get_all_direct_changes<'a>(
        &'a self,
    ) -> impl Iterator<Item = (Substance, &SubstanceChange)> {
        self.substance_changes.iter().map(|(s, cm)| cm.values().map(move |c| (*s, c))).flatten()
    }

    /// Returns `true` if new changes have occurred since the last call to
    /// get_new_direct_changes(), `false` otherwise
    pub fn has_new_changes(&self) -> bool {
        !self.new_changes.is_empty()
    }


    /// Get an iterator to all newly added `SubstanceChange`s
    /// since the last time the method was called
    ///
    /// Returns an iterator of all new `SubstanceChange`s
    pub fn get_new_direct_changes(
        &mut self,
    ) -> impl Iterator<Item = (Substance, &SubstanceChange)> {

        // Create a reference of the changes that the iterator can own
        let new_changes = &self.new_changes;

        let results = self.substance_changes
            .iter()
            .filter(move |(s, cm)| new_changes.get(s).is_some_and(|id| cm.contains_key(id)))
            .map(|(s, cm)| cm.values().map(move |c| (*s, c))).flatten();

        results
    }

    /// Unschedule a substance change on this store
    ///
    /// ### Arguments
    /// * `substance` - the substance which was scheduled to be changed
    /// * `change_id` - the id returned from the call to schedule_change
    ///
    /// Returns a `SubstanceChange` if found and the change hasn't completed, None otherwise
    pub fn unschedule_change(
        &mut self,
        substance: &Substance,
        change_id: &IdType,
    ) -> Option<SubstanceChange> {
        self.substance_changes
            .entry(*substance)
            .or_default()
            .remove(change_id)
    }

    /// Track any new changes added to this store
    pub fn track_changes(&mut self) {
        self.track_changes = true;
    }

    /// Clear all scheduled changes on this store
    pub fn clear_all_changes(&mut self) {
        self.substance_changes.clear()
    }

    /// Advances time for this substance store, making any necessary changes
    ///
    /// ### Arguments
    /// * `sim_time` - the new simulation time
    pub(crate) fn advance(&mut self, sim_time: SimTime) {

        if self.track_changes {
            swap(&mut self.staged_changes, &mut self.new_changes);
            self.staged_changes.clear();
        }

        for (substance, change_map) in self.substance_changes.iter_mut() {
            let mut ids_to_remove = Vec::new();

            for (change_id, change) in change_map.iter_mut() {
                if change.start_time() < sim_time {
                    // Change we need to add is the function value at the current time
                    // minus the value recorded from the previous time point
                    let change_amt = change.next_amount(sim_time);
                    let prev_conc = self
                        .composition
                        .get(substance)
                        .unwrap_or(Self::zero_concentration());

                    // Check to make sure new concentration is non-negative
                    let mut new_conc = *prev_conc + change_amt;
                    if new_conc < SubstanceConcentration::from_M(0.0) {
                        log::warn!(
                            "Substance change attempted to set a negative solute concentration for {}: {}\n{}",
                            substance,
                            new_conc,
                            "Setting to zero.",
                        );
                        new_conc = SubstanceConcentration::from_M(0.0);
                    }

                    // Check to make sure new concentration doesn't exceed possible solute volume
                    let change_pct = change_amt.molpm3*substance.molar_volume().m3_per_mol;
                    if self.solute_pct + change_pct > 1.0 {
                        log::warn!(
                            "Substance change attempted to set an invalid solute concentration for {}: {}\n{}\n{}",
                            substance,
                            new_conc,
                            format!("Total solute percentage would be {:.1}%",change_pct*100.0),
                            format!("{} concentration will remain unchanged.", substance),
                        );
                        continue;
                    }
                    
                    // register the change in solute percent and the concentration change
                    self.solute_pct += change_pct;
                    self.composition.insert(*substance, new_conc);
                }

                if sim_time > change.start_time() + change.duration() {
                    ids_to_remove.push(*change_id);
                }
            }

            for change_id in ids_to_remove {
                change_map.remove(&change_id);
            }
        }

        self.sim_time = sim_time;
    }
}


mod tests {
    use super::{BoundFn, Substance, SubstanceStore, ZERO_CONCENTRATION};
    use crate::{
        substance::SubstanceConcentration,
        util::{mmol_per_L, secs}, SimTimeSpan,
    };
    use std::collections::HashMap;

    #[test]
    fn bad_concentration() {
        let mut store = SubstanceStore::new();
        let err = store.set_concentration(Substance::GLC, SubstanceConcentration::from_M(200.0));
        println!("{:?}", err);
        assert!(err.is_err());

        assert!(store.set_concentration(Substance::GLC, SubstanceConcentration::from_M(-1.0)).is_err());
    }

    #[test]
    fn has_empty() {
        let store = SubstanceStore::new();
        assert_eq!(
            store.concentration_of(&Substance::ADP),
            *ZERO_CONCENTRATION.get_or_init(|| { SubstanceConcentration::from_M(0.0) })
        );
    }

    #[test]
    fn has_value() {
        let mut store = SubstanceStore::new();
        let concentration = mmol_per_L!(1.0);
        store.set_concentration(Substance::ADP, concentration.clone()).unwrap();
        assert_eq!(store.concentration_of(&Substance::ADP), concentration);
    }

    #[test]
    fn merge_composition() {
        let mut store = SubstanceStore::new();
        let mut composition = HashMap::new();
        composition.insert(Substance::ADP, mmol_per_L!(1.0));
        composition.insert(Substance::ATP, mmol_per_L!(2.0));
        store.merge_composition(&composition);

        assert_eq!(store.concentration_of(&Substance::ADP), mmol_per_L!(1.0));
        assert_eq!(store.concentration_of(&Substance::ATP), mmol_per_L!(2.0));
    }

    #[test]
    fn merge_from() {
        let mut store = SubstanceStore::new();
        let mut other_store = SubstanceStore::new();
        other_store.set_concentration(Substance::ADP, mmol_per_L!(1.0)).unwrap();
        other_store.set_concentration(Substance::ATP, mmol_per_L!(2.0)).unwrap();
        store.merge_from(&other_store);

        assert_eq!(store.concentration_of(&Substance::ADP), mmol_per_L!(1.0));
        assert_eq!(store.concentration_of(&Substance::ATP), mmol_per_L!(2.0));
    }

    #[test]
    fn changes() {
        let mut store = SubstanceStore::new();
        let change_id = store.schedule_change(
            Substance::ADP,
            mmol_per_L!(1.0),
            secs!(0.0),
            SimTimeSpan::from_s(1.0),
            BoundFn::Sigmoid,
        );
        store.schedule_change(
            Substance::ATP,
            mmol_per_L!(1.0),
            secs!(1.0),
            SimTimeSpan::from_s(1.0),
            BoundFn::Sigmoid,
        );

        let time1 = secs!(0.5);
        store.advance(time1);

        // ADP should change, ATP should remaint at zero
        let adp_conc = store.concentration_of(&Substance::ADP);
        let expected_adp = mmol_per_L!(0.5);
        assert!(
            (adp_conc - expected_adp).molpm3.abs() < 0.0001,
            "Incorrect ADP concentration, found {:?}, expected {:?}",
            adp_conc,
            expected_adp
        );
        assert_eq!(
            store.concentration_of(&Substance::ATP),
            *ZERO_CONCENTRATION.get_or_init(|| { SubstanceConcentration::from_M(0.0) })
        );

        // Unschedule changes to ADP, so that should now not change anymore
        store.unschedule_change(&Substance::ADP, &change_id);

        let time2 = secs!(1.5);
        store.advance(time2);

        // ADP shouldn't have changed
        let adp_conc2 = store.concentration_of(&Substance::ADP);
        assert!(
            (adp_conc2 - expected_adp).molpm3.abs() < 0.01,
            "Incorrect ADP concentration, found {:?}, expected {:?}",
            adp_conc,
            expected_adp
        );

        // ATP should now be approximately at half
        let atp_conc1 = store.concentration_of(&Substance::ATP);
        let expected_atp1 = mmol_per_L!(0.5);
        assert!(
            (atp_conc1 - expected_atp1).molpm3.abs() < 0.01,
            "Incorrect ATP concentration, found {:?}, expected {:?}",
            adp_conc,
            expected_atp1
        );

        let time3 = secs!(2.0);
        store.advance(time3);

        // ATP should now be approximately at full value
        let atp_conc2 = store.concentration_of(&Substance::ATP);
        let expected_atp2 = mmol_per_L!(1.0);
        assert!(
            (atp_conc2 - expected_atp2).molpm3.abs() < 0.01,
            "Incorrect ATP concentration, found {:?}, expected {:?}",
            atp_conc2,
            expected_atp2
        );

        let time4 = secs!(5.0);
        store.advance(time4);

        // after much time they should still be about the same
        let adp_conc3 = store.concentration_of(&Substance::ADP);
        let atp_conc3 = store.concentration_of(&Substance::ATP);
        assert!(
            (adp_conc3 - expected_adp).molpm3.abs() < 0.01,
            "Incorrect ADP concentration, found {:?}, expected {:?}",
            adp_conc3,
            expected_adp
        );
        assert!(
            (atp_conc3 - expected_atp2).molpm3.abs() < 0.01,
            "Incorrect ATP concentration, found {:?}, expected {:?}",
            atp_conc3,
            expected_atp2
        );
    }
}
