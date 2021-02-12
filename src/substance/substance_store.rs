use std::collections::HashMap;
use std::fmt;
use core::any::TypeId;
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

/// An abstract storage construct for Substance concentrations in a volume
pub struct SubstanceStore<'a> {
    identifier: String,
    /// Substance volume
    pub volume: Volume,
    pub composition: HashMap<Substance, MolarConcentration>,
    id_gen: IdGenerator,
    composition_listeners: HashMap<Substance, Vec<(IdType, Box<dyn FnMut(&MolarConcentration) + 'a>)>>,
    any_composition_listeners: Vec<(IdType, Box<dyn FnMut(&Substance, &MolarConcentration) + 'a>)>,
    volume_listeners: Vec<(IdType, Box<dyn FnMut() + 'a>)>
}

impl<'a> fmt::Display for SubstanceStore<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.identifier)?;
        Ok(())
    }
}

impl<'a> SubstanceStore<'a> {
    pub fn new(identifier: String, volume: Volume) -> SubstanceStore<'a> {
        SubstanceStore {
            identifier,
            volume: volume,
            composition: HashMap::new(),
            id_gen: IdGenerator::new(),
            composition_listeners: HashMap::new(),
            any_composition_listeners: Vec::new(),
            volume_listeners: Vec::new(),
        }
    }

    pub fn has(&self, substance: &Substance) -> bool {
        self.composition.contains_key(substance)
    }
    pub fn concentration_of(&self, substance: &Substance) -> MolarConcentration {
        match self.composition.get(substance) {
            None => MolarConcentration::new::<mole_per_liter>(0.0),
            Some(amt) => amt.clone()
        }
    }
    pub fn amount_of(&self, substance: &Substance) -> AmountOfSubstance {
        match self.composition.get(substance) {
            None => AmountOfSubstance::new::<mole>(0.0),
            Some(amt) => amt.clone() * self.volume
        }
    }
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
    pub fn set_amount(&mut self, substance: &Substance, amount: AmountOfSubstance) {
        self.set_concentration(substance, (amount / self.volume).into());
    }
    pub fn set_volume(&mut self, volume: Volume) {
        self.volume = volume;
        for (_, listener) in self.volume_listeners.iter_mut() {
            listener();
        }
    }
    pub fn merge_composition(&mut self, composition: HashMap<Substance, MolarConcentration>) {
        self.composition.extend(composition);
    }
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
    pub fn off_composition_change(&mut self, substance: &Substance, listener_id: IdType) -> Result<()> {
        match self.composition_listeners.get_mut(substance)  {
            Some(listeners) => {
                match listeners.iter().position(|(id,_)| id == &listener_id) {
                    Some(idx) => {
                        let _ = listeners.remove(idx);
                        Ok(())
                    },
                    None => Err(anyhow::Error::new(InvalidIdError::new(self.to_string(), listener_id)))
                }
            },
            None => Err(anyhow::Error::new(InvalidIdError::new(self.to_string(), listener_id)))
        }
    }
    pub fn on_any_composition_change(&mut self, listener: impl FnMut(&Substance, &MolarConcentration) + 'a) -> IdType {
        let listener_id = self.id_gen.get_id();
        self.any_composition_listeners.push((listener_id, Box::new(listener)));
        listener_id
    }
    pub fn off_any_composition_change(&mut self, listener_id: IdType) -> Result<()> {
        match self.any_composition_listeners.iter().position(|(id,_)| id == &listener_id) {
            Some(idx) => {
                let _ = self.any_composition_listeners.remove(idx);
                Ok(())
            },
            None => Err(anyhow::Error::new(InvalidIdError::new(self.to_string(), listener_id)))
        }
    }
    pub fn on_volume_change(&mut self, listener: impl FnMut() + 'a) -> IdType {
        let listener_id = self.id_gen.get_id();
        self.volume_listeners.push((listener_id, Box::new(listener)));
        listener_id
    }
    pub fn off_volume_change(&mut self, listener_id: IdType) -> Result<()> {
        match self.volume_listeners.iter().position(|(id,_)| id == &listener_id) {
            Some(idx) => {
                let _ = self.volume_listeners.remove(idx);
                Ok(())
            },
            None => Err(anyhow::Error::new(InvalidIdError::new(self.to_string(), listener_id)))
        }
    }
}
