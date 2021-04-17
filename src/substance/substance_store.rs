use std::collections::{HashMap, HashSet};
use std::fmt;
use core::any::TypeId;
use uuid::Uuid;
use uom::si::f64::*;
use uom::si::length::kilometer;
use uom::si::time::second;
use uom::si::volume::liter;
use uom::si::molar_concentration::mole_per_liter;
use uom::si::amount_of_substance::mole;
use anyhow::Result;
use crate::util::id_gen::{IdType, IdGenerator, InvalidIdError};
use crate::substance::Substance;

/// A storage construct for Substance concentrations in a volume
pub struct SubstanceStore {
    /// Id for this SubstanceStore
    store_id: Uuid,
    /// Substance volume
    volume: Volume,
    composition: HashMap<Substance, MolarConcentration>,
    id_gen: IdGenerator,
    /// Keep track of any Events which have been tainted
    tainted_substances: HashSet<Substance>,
}

impl fmt::Debug for SubstanceStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SubstanceStore<{:?}> {{ volume = {:?}, composition = {:?} }}", self.store_id, self.volume, self.composition)?;
        Ok(())
    }
}

impl SubstanceStore {
    /// Constructs a new Substance store with the given identifier and initial volume
    /// 
    /// ### Arguments
    /// * `volume` - initial volume
    pub fn new(volume: Volume) -> SubstanceStore {
        SubstanceStore {
            store_id: Uuid::new_v4(),
            volume: volume,
            composition: HashMap::new(),
            id_gen: IdGenerator::new(),
            tainted_substances: HashSet::new(),
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
    /// Returns the amount of that substance, or 0.0 mol/L if it is not present
    pub fn concentration_of(&self, substance: &Substance) -> Option<MolarConcentration> {
        match self.composition.get(substance) {
            None => None,
            Some(amt) => Some(amt.clone())
        }
    }
    
    /// Retrieves the amount of a given Substance in the store.
    /// 
    /// ### Arguments
    /// * `substance` - Substance to retrieve
    /// 
    /// Returns the amount of that substance, or 0.0 mol if it is not present
    pub fn amount_of(&self, substance: &Substance) -> Option<AmountOfSubstance> {
        match self.composition.get(substance) {
            None => None,
            Some(amt) => Some(amt.clone() * self.volume)
        }
    }
    
    /// sets the concentration of a given Substance in the store.
    /// 
    /// ### Arguments
    /// * `substance` - Substance to set the concentration for
    /// * `concentration` - concentration to set for the Substance
    pub fn set_concentration(&mut self, substance: Substance, concentration: MolarConcentration) {
        self.composition.insert(substance, concentration.clone());
        self.tainted_substances.insert(substance);
    }
    
    /// sets the amount of a given Substance in the store.
    /// 
    /// ### Arguments
    /// * `substance` - Substance to set the concentration for
    /// * `amount` - amount to set for the Substance
    pub fn set_amount(&mut self, substance: Substance, amount: AmountOfSubstance) {
        self.set_concentration(substance, (amount / self.volume).into());
    }
    
    /// Modifies the volume of this store
    /// 
    /// ### Arguments
    /// * `volume` - new volume to set
    pub fn set_volume(&mut self, volume: Volume) {
        self.volume = volume;
    }
    
    /// Retrieves the volume of this store
    /// 
    /// Returns the volume of this SubstanceStore
    pub fn get_volume(&mut self) -> &Volume {
        &self.volume
    }
    
    /// Merges the provided composition with this store's internal composition, updating
    /// any existing substances and adding any new concentrations
    /// 
    /// ### Arguments
    /// * `composition` - the Substance composition to merge
    pub fn merge_composition(&mut self, composition: HashMap<Substance, MolarConcentration>) {
        self.composition.extend(composition);
    }
    
}
