use std::collections::{HashMap, HashSet};
use std::fmt;
use core::any::TypeId;
use std::ops::Bound;
use uuid::Uuid;
use anyhow::Result;
use uom::si::molar_concentration::millimole_per_liter;
use crate::util::id_gen::{IdType, IdGenerator, InvalidIdError};
use crate::substance::Substance;
use crate::core::sim::{Time, second};
use crate::util::BoundFn;
use super::MolarConcentration;
use super::SubstanceChange;

lazy_static! {
    static ref ZERO_CONCENTRATION: MolarConcentration = MolarConcentration::new::<millimole_per_liter>(0.0);
}

/// A storage construct for Substance concentrations in a volume
pub struct SubstanceStore {
    /// Id for this SubstanceStore
    store_id: Uuid,
    /// Id generator for substance changes
    id_gen: IdGenerator,
    /// Local cache of simulation time
    sim_time: Time,
    /// Data structure containing the internal substance concentration
    composition: HashMap<Substance, MolarConcentration>,
    /// Keep track of any Substances which are changing
    substance_changes: HashMap<Substance, HashMap<IdType, SubstanceChange>>,
}

impl fmt::Debug for SubstanceStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SubstanceStore<{:?}> {{composition = {:?}, substance_changes = {:?}}}", self.store_id, self.composition, self.substance_changes)?;
        Ok(())
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
            sim_time: Time::new::<second>(0.0),
            composition: HashMap::new(),
            substance_changes: HashMap::new(),
        }
    }

    /// Determines whether the store has any of the indicated Substance
    /// 
    /// ### Arguments
    /// * `substance` - Substance to check
    /// 
    /// Returns true if any amount of that Substance is present, false otherwise
    pub fn has(&self, substance: &Substance) -> bool {
        self.composition.contains_key(substance)
    }
    
    /// Retrieves the concentration of a given Substance in the store.
    /// 
    /// ### Arguments
    /// * `substance` - Substance to retrieve
    /// 
    /// Returns the amount of that substance, or None if it is not present
    pub fn concentration_of(&self, substance: &Substance) -> Option<MolarConcentration> {
        match self.composition.get(substance) {
            None => None,
            Some(amt) => Some(amt.clone())
        }
    }
    
    /// sets the concentration of a given Substance in the store.
    /// 
    /// ### Arguments
    /// * `substance` - Substance to set the concentration for
    /// * `concentration` - concentration to set for the Substance
    pub fn set_concentration(&mut self, substance: Substance, concentration: MolarConcentration) {
        self.composition.insert(substance, concentration.clone());
    }
    
    /// Merges the provided composition with this store's internal composition, updating
    /// any existing substances and adding any new concentrations
    /// 
    /// ### Arguments
    /// * `composition` - the Substance composition to merge
    pub fn merge_composition(&mut self, composition: HashMap<Substance, MolarConcentration>) {
        self.composition.extend(composition);
    }
    
    /// Merges the target store's composition with this store's internal composition, updating
    /// any existing substances and adding any new concentrations
    /// 
    /// ### Arguments
    /// * `other` - the SubstanceStore to merge
    pub fn merge_all(&mut self, other: &SubstanceStore) {
        self.composition.extend(other.composition.clone());
    }

    /// Schedule a substance change on this store with a sigmoid
    /// shape over the given duration.
    /// 
    /// Panics if `duration <= 0`
    /// 
    /// ### Arguments
    /// * `substance` - the substance to change
    /// * `amount`    - total concentration change to take place
    /// * `duration`  - amount of time over which the change takes place
    /// 
    /// Returns an id corresponding to this change
    pub fn schedule_change(&mut self, substance: Substance, amount: MolarConcentration, duration: Time) -> IdType {
        let change_id = self.id_gen.get_id();
        let change = SubstanceChange::new(self.sim_time, amount, duration, BoundFn::Sigmoid);
        self.substance_changes.entry(substance).or_default().insert(change_id, change);

        change_id
    }
    
    /// Schedule a substance change on this store with a custom
    /// shape over the given duration.
    /// 
    /// Panics if `duration <= 0`
    /// 
    /// ### Arguments
    /// * `substance` - the substance to change
    /// * `amount`    - total concentration change to take place
    /// * `duration`  - amount of time over which the change takes place
    /// * `bound_fn`  - the shape of the function 
    /// 
    /// Returns an id corresponding to this change
    pub fn schedule_change_custom(&mut self, substance: Substance, amount: MolarConcentration, duration: Time, bound_fn: BoundFn) -> IdType {
        let change_id = self.id_gen.get_id();
        let change = SubstanceChange::new(self.sim_time, amount, duration, bound_fn);
        self.substance_changes.entry(substance).or_default().insert(change_id, change);

        change_id
    }
    
    /// Schedule a substance change on this store for a future time
    /// with a sigmoid shape over the given duration.
    /// If the `start_time` is less than the current simulation time,
    /// it will be set to the current simulation time.
    /// 
    /// Panics if `duration <= 0`
    /// 
    /// ### Arguments
    /// * `substance` - the substance to change
    /// * `amount`    - total concentration change to take place
    /// * `duration`  - amount of time over which the change takes place
    /// * `bound_fn`  - the shape of the function 
    /// 
    /// Returns an id corresponding to this change
    pub fn schedule_future_change(&mut self, substance: Substance, start_time: Time, amount: MolarConcentration, duration: Time) -> IdType {
        // Constrain the start time to a minimum of the current sim time
        let x_start_time = {
            if start_time < self.sim_time {
                self.sim_time;
            }
            start_time
        };

        let change_id = self.id_gen.get_id();
        let change = SubstanceChange::new(x_start_time, amount, duration, BoundFn::Sigmoid);
        self.substance_changes.entry(substance).or_default().insert(change_id, change);

        change_id
    }
    
    /// Schedule a substance change on this store for a future time
    /// with a custom shape over the given duration.
    /// If the `start_time` is less than the current simulation time,
    /// it will be set to the current simulation time.
    /// 
    /// Panics if `duration <= 0`
    /// 
    /// ### Arguments
    /// * `substance` - the substance to change
    /// * `amount`    - total concentration change to take place
    /// * `duration`  - amount of time over which the change takes place
    /// * `bound_fn`  - the shape of the function 
    /// 
    /// Returns an id corresponding to this change
    pub fn schedule_future_change_custom(&mut self, substance: Substance, start_time: Time, amount: MolarConcentration, duration: Time, bound_fn: BoundFn) -> IdType {
        // Constrain the start time to a minimum of the current sim time
        let x_start_time = {
            if start_time < self.sim_time {
                self.sim_time;
            }
            start_time
        };

        let change_id = self.id_gen.get_id();
        let change = SubstanceChange::new(x_start_time, amount, duration, bound_fn);
        self.substance_changes.entry(substance).or_default().insert(change_id, change);

        change_id
    }
    
    /// Unschedule a substance change on this store
    /// 
    /// ### Arguments
    /// * `substance` - the substance which was scheduled to be changed
    /// * `change_id` - the id returned from the call to schedule_change
    /// 
    /// Returns the provided BoundFn if found and the change hasn't completed, None otherwise
    pub fn unschedule_change(&mut self, substance: Substance, change_id: IdType) -> Option<SubstanceChange> {
        self.substance_changes.entry(substance).or_default().remove(&change_id)
    }
    
    /// Advances time for this substance store, making any necessary changes 
    /// 
    /// ### Arguments
    /// * `sim_time` - the new simulation time
    pub fn advance(&mut self, sim_time: Time) {
        for (substance, change_map) in self.substance_changes.iter_mut() {

            let mut ids_to_remove = Vec::new();

            for (change_id, change) in change_map.iter_mut() {
                if change.start_time < sim_time {
                    // Change we need to add is the function value at the current time
                    // minus the value recorded from the previous time point
                    let change_val = change.call(sim_time - change.start_time) - change.previous_val;
                    let prev_conc = self.composition.get(substance).unwrap_or(&ZERO_CONCENTRATION);
                    let new_conc = *prev_conc + change_val;
                    self.composition.insert(*substance, new_conc);
                }

                if sim_time > change.start_time + change.duration {
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
