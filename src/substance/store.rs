use super::{SubstanceConcentration, SubstanceChange, ZERO_CONCENTRATION};
use crate::sim::SimTime;
use crate::substance::Substance;
use crate::util::id_gen::{IdGenerator, IdType, InvalidIdError};
use crate::util::{mmol_per_L, secs, BoundFn};
use anyhow::Result;
use core::any::TypeId;
use core::panic;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::Bound;
use uuid::Uuid;

/// A storage construct for Substance concentrations in a volume
pub struct SubstanceStore {
    /// Id for this SubstanceStore
    store_id: Uuid,
    /// Id generator for substance changes
    id_gen: IdGenerator,
    /// Local cache of simulation time
    sim_time: SimTime,
    /// Data structure containing the internal substance concentration
    composition: HashMap<Substance, SubstanceConcentration>,
    /// Keep track of any Substances which are changing
    substance_changes: HashMap<Substance, HashMap<IdType, SubstanceChange>>,
}

impl fmt::Debug for SubstanceStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SubstanceStore<{:?}> {{composition = {:?}, substance_changes = {:?}}}",
            self.store_id, self.composition, self.substance_changes
        )?;
        Ok(())
    }
}

impl Default for SubstanceStore {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static! {
    static ref DEFAULT_STORE: SubstanceStore = SubstanceStore::new();
}

impl<'a> Default for &'a SubstanceStore {
    fn default() -> Self {
        &DEFAULT_STORE
    }
}

impl SubstanceStore {
    /// Constructs a new Substance store with the given identifier and initial volume
    ///
    /// ### Arguments
    /// * `volume` - initial volume
    pub fn new() -> SubstanceStore {
        SubstanceStore {
            store_id: Uuid::new_v4(),
            id_gen: IdGenerator::new(),
            sim_time: secs!(0.0),
            composition: HashMap::new(),
            substance_changes: HashMap::new(),
        }
    }

    /// Retrieves the current simulation time for the store.
    pub fn sim_time(&self) -> SimTime {
        self.sim_time
    }

    /// Retrieves the concentration of a given Substance in the store.
    ///
    /// ### Arguments
    /// * `substance` - Substance to retrieve
    ///
    /// Returns the current concentration of the substance
    pub fn concentration_of(&self, substance: &Substance) -> SubstanceConcentration {
        match self.composition.get(substance) {
            None => *ZERO_CONCENTRATION,
            Some(amt) => *amt,
        }
    }

    /// sets the concentration of a given Substance in the store.
    ///
    /// ### Arguments
    /// * `substance` - Substance to set the concentration for
    /// * `concentration` - concentration to set for the Substance
    pub fn set_concentration(&mut self, substance: Substance, concentration: SubstanceConcentration) {
        self.composition.insert(substance, concentration);
    }

    /// Merges the provided composition with this store's internal composition, updating
    /// any existing substances and adding any new concentrations
    ///
    /// ### Arguments
    /// * `composition` - the Substance composition to merge
    pub fn merge_composition(&mut self, composition: &HashMap<Substance, SubstanceConcentration>) {
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
    /// Panics if `delay < 0` or `duration <= 0`
    ///
    /// ### Arguments
    /// * `substance`  - the substance to change
    /// * `delay`      - future simulation time to start the change
    /// * `substance`  - the substance to change
    /// * `amount`     - total concentration change to take place
    /// * `duration`   - amount of time over which the change takes place
    /// * `bound_fn`   - the shape of the function
    ///
    /// Returns an id corresponding to this change
    pub fn schedule_change(
        &mut self,
        substance: Substance,
        amount: SubstanceConcentration,
        delay: SimTime,
        duration: SimTime,
        bound_fn: BoundFn,
    ) -> IdType {
        // Constrain the start time to a minimum of the current sim time
        let x_start_time = {
            if delay.s < 0.0 {
                panic!("Delay cannot be less than zero!");
            }
            self.sim_time + delay
        };

        let change_id = self.id_gen.get_id();
        let change = SubstanceChange::new(x_start_time, amount, duration, bound_fn);
        self.substance_changes
            .entry(substance)
            .or_default()
            .insert(change_id, change);

        change_id
    }

    /// Schedule a substance change on this store
    /// defined by the given `SubstanceChange`
    ///
    /// ### Arguments
    /// * `substance`  - the substance to change
    /// * `change`     - the change to exert on the substance
    ///
    /// Returns an id corresponding to this change
    pub(crate) fn schedule_substance_change(
        &mut self,
        substance: Substance,
        change: SubstanceChange
    ) -> IdType {
        let change_id = self.id_gen.get_id();
        self.substance_changes
            .entry(substance)
            .or_default()
            .insert(change_id, change);

        change_id
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

    /// Advances time for this substance store, making any necessary changes
    ///
    /// ### Arguments
    /// * `sim_time` - the new simulation time
    pub fn advance(&mut self, sim_time: SimTime) {
        for (substance, change_map) in self.substance_changes.iter_mut() {
            let mut ids_to_remove = Vec::new();

            for (change_id, change) in change_map.iter_mut() {
                if change.start_time < sim_time {
                    // Change we need to add is the function value at the current time
                    // minus the value recorded from the previous time point
                    let change_amt = change.next_amount(sim_time);
                    let prev_conc = self
                        .composition
                        .get(substance)
                        .unwrap_or(&ZERO_CONCENTRATION);
                    let new_conc = *prev_conc + change_amt;
                    self.composition.insert(*substance, new_conc);
                }

                if sim_time.s > change.start_time.s + change.duration.s {
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

#[cfg(test)]
mod tests {
    use super::{
        BoundFn, SubstanceConcentration, Substance, SubstanceStore, SimTime,
        ZERO_CONCENTRATION,
    };
    use crate::util::{mmol_per_L, secs};
    use std::collections::HashMap;

    #[test]
    fn has_empty() {
        let store = SubstanceStore::new();
        assert_eq!(store.concentration_of(&Substance::ADP), *ZERO_CONCENTRATION);
    }

    #[test]
    fn has_value() {
        let mut store = SubstanceStore::new();
        let concentration = mmol_per_L!(1.0);
        store.set_concentration(Substance::ADP, concentration.clone());
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
        other_store.set_concentration(Substance::ADP, mmol_per_L!(1.0));
        other_store.set_concentration(Substance::ATP, mmol_per_L!(2.0));
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
            secs!(1.0),
            BoundFn::Sigmoid,
        );
        store.schedule_change(
            Substance::ATP,
            mmol_per_L!(1.0),
            secs!(1.0),
            secs!(1.0),
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
        assert_eq!(store.concentration_of(&Substance::ATP), *ZERO_CONCENTRATION);

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
