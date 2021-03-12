use std::collections::HashMap;
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

lazy_static! {
    static ref ZERO_MOLE: AmountOfSubstance = AmountOfSubstance::new::<mole>(0.0);
    static ref ZERO_MOLAR: MolarConcentration = MolarConcentration::new::<mole_per_liter>(0.0);
}

/// A storage construct for Substance concentrations in a volume
pub struct SubstanceStore<'a> {
    /// Id for this SubstanceStore
    store_id: Uuid,
    /// Substance volume
    volume: Volume,
    composition: HashMap<Substance, MolarConcentration>,
    id_gen: IdGenerator,
    composition_listeners: HashMap<Substance, Vec<(IdType, Box<dyn FnMut(&MolarConcentration) + 'a>)>>,
    any_composition_listeners: Vec<(IdType, Box<dyn FnMut(&Substance, &MolarConcentration) + 'a>)>,
    volume_listeners: Vec<(IdType, Box<dyn FnMut() + 'a>)>
}

impl<'a> fmt::Debug for SubstanceStore<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SubstanceStore<{:?}> {{ volume = {:?}, composition = {:?} }}", self.store_id, self.volume, self.composition)?;
        Ok(())
    }
}

impl<'a> SubstanceStore<'a> {
    /// Constructs a new Substance store with the given identifier and initial volume
    /// 
    /// # Arguments
    /// * `volume` - initial volume
    pub fn new(volume: Volume) -> SubstanceStore<'a> {
        SubstanceStore {
            store_id: Uuid::new_v4(),
            volume: volume,
            composition: HashMap::new(),
            id_gen: IdGenerator::new(),
            composition_listeners: HashMap::new(),
            any_composition_listeners: Vec::new(),
            volume_listeners: Vec::new(),
        }
    }

    /// Determines whether the store has any of the indicated Substance
    /// 
    /// # Arguments
    /// * `substance` - Substance to check
    /// 
    /// Returns true if any amount of that Substance is present, false otherwise
    pub fn has(&self, substance: &Substance) -> bool {
        self.composition.contains_key(substance)
    }
    
    /// Retrieves the concentration of a given Substance in the store.
    /// 
    /// # Arguments
    /// * `substance` - Substance to retrieve
    /// 
    /// Returns the amount of that substance, or 0.0 mol/L if it is not present
    pub fn concentration_of(&self, substance: &Substance) -> MolarConcentration {
        match self.composition.get(substance) {
            None => MolarConcentration::new::<mole_per_liter>(0.0),
            Some(amt) => amt.clone()
        }
    }
    
    /// Retrieves the amount of a given Substance in the store.
    /// 
    /// # Arguments
    /// * `substance` - Substance to retrieve
    /// 
    /// Returns the amount of that substance, or 0.0 mol if it is not present
    pub fn amount_of(&self, substance: &Substance) -> AmountOfSubstance {
        match self.composition.get(substance) {
            None => AmountOfSubstance::new::<mole>(0.0),
            Some(amt) => amt.clone() * self.volume
        }
    }
    
    /// sets the concentration of a given Substance in the store.
    /// 
    /// # Arguments
    /// * `substance` - Substance to set the concentration for
    /// * `concentration` - concentration to set for the Substance
    pub fn set_concentration(&mut self, substance: &Substance, concentration: MolarConcentration) {
        self.composition.insert(substance.clone(), concentration.clone());
        match self.composition_listeners.get_mut(&substance) {
            None => {},
            Some(listeners) => {
                for (_, listener) in listeners.iter_mut() {
                    listener(&concentration);
                }
            }
        }
    }
    
    /// sets the amount of a given Substance in the store.
    /// 
    /// # Arguments
    /// * `substance` - Substance to set the concentration for
    /// * `amount` - amount to set for the Substance
    pub fn set_amount(&mut self, substance: &Substance, amount: AmountOfSubstance) {
        self.set_concentration(substance, (amount / self.volume).into());
    }
    
    /// Modifies the volume of this store
    /// 
    /// # Arguments
    /// * `volume` - new volume to set
    pub fn set_volume(&mut self, volume: Volume) {
        self.volume = volume;
        for (_, listener) in self.volume_listeners.iter_mut() {
            listener();
        }
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
    /// # Arguments
    /// * `composition` - the Substance composition to merge
    pub fn merge_composition(&mut self, composition: HashMap<Substance, MolarConcentration>) {
        self.composition.extend(composition);
    }
    
    /// Registers a listener for composition changes on a particular substance
    /// 
    /// # Arguments
    /// * `substance` - the Substance to register the listener for
    /// * `listener` - substance change listener function
    /// 
    /// Returns a unique id for this listener
    pub fn on_composition_change(&mut self, substance: &Substance, listener: impl FnMut(&MolarConcentration) + 'a) -> IdType {
        let listener_id = self.id_gen.get_id();
        match self.composition_listeners.get_mut(substance) {
            None => {
                self.composition_listeners.insert(substance.clone(), vec!((listener_id, Box::new(listener))));
            },
            Some(listeners) => {
                listeners.push((listener_id, Box::new(listener)));
            }
        }
        listener_id
    }
    
    /// Removes a listener for composition changes on a particular substance
    /// 
    /// # Arguments
    /// * `substance` - the Substance to the listener is registered for
    /// * `listener_id` - the listener id returned from the call to `on_composition_change`
    /// 
    /// Returns Ok if the id was removed successfully and an InvalidIdError otherwise
    pub fn off_composition_change(&mut self, substance: &Substance, listener_id: IdType) -> Result<()> {
        match self.composition_listeners.get_mut(substance)  {
            Some(listeners) => {
                match listeners.iter().position(|(id,_)| id == &listener_id) {
                    Some(idx) => {
                        let _ = listeners.remove(idx);
                        Ok(())
                    },
                    None => Err(anyhow::Error::new(InvalidIdError::new(format!("{:?}", self), listener_id)))
                }
            },
            None => Err(anyhow::Error::new(InvalidIdError::new(format!("{:?}", self), listener_id)))
        }
    }
    
    /// Registers a listener for any composition changes on this SubstanceStore
    /// 
    /// # Arguments
    /// * `listener` - substance change listener function
    /// 
    /// Returns a unique id for this listener
    pub fn on_any_composition_change(&mut self, listener: impl FnMut(&Substance, &MolarConcentration) + 'a) -> IdType {
        let listener_id = self.id_gen.get_id();
        self.any_composition_listeners.push((listener_id, Box::new(listener)));
        listener_id
    }
    
    /// Removes a listener for any composition changes on this SubstanceStore
    /// 
    /// # Arguments
    /// * `listener_id` - the listener id returned from the call to `on_any_composition_change`
    /// 
    /// Returns Ok if the id was removed successfully and an InvalidIdError otherwise
    pub fn off_any_composition_change(&mut self, listener_id: IdType) -> Result<()> {
        match self.any_composition_listeners.iter().position(|(id,_)| id == &listener_id) {
            Some(idx) => {
                let _ = self.any_composition_listeners.remove(idx);
                Ok(())
            },
            None => Err(anyhow::Error::new(InvalidIdError::new(format!("{:?}", self), listener_id)))
        }
    }
    
    /// Registers a listener for volume changes on this SubstanceStore
    /// 
    /// # Arguments
    /// * `listener` - volume change listener function
    /// 
    /// Returns a unique id for this listener
    pub fn on_volume_change(&mut self, listener: impl FnMut() + 'a) -> IdType {
        let listener_id = self.id_gen.get_id();
        self.volume_listeners.push((listener_id, Box::new(listener)));
        listener_id
    }
    
    /// Removes a listener for volume changes on this SubstanceStore
    /// 
    /// # Arguments
    /// * `listener_id` - the listener id returned from the call to `on_any_composition_change`
    /// 
    /// Returns Ok if the id was removed successfully and an InvalidIdError otherwise
    pub fn off_volume_change(&mut self, listener_id: IdType) -> Result<()> {
        match self.volume_listeners.iter().position(|(id,_)| id == &listener_id) {
            Some(idx) => {
                let _ = self.volume_listeners.remove(idx);
                Ok(())
            },
            None => Err(anyhow::Error::new(InvalidIdError::new(format!("{:?}", self), listener_id)))
        }
    }
}
