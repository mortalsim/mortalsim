use crate::{sim::Organism, units::base::Distance};

use super::{Event, Infection, NumType};

/// Properties of an acute wound
#[derive(Debug, Clone, PartialEq)]
pub struct WoundProperties<O: Organism> {
    location: O::AnatomyType,
    length: Distance<NumType>,
    width: Distance<NumType>,
    depth: Distance<NumType>,
    infections: Vec<Infection<O>>,
}

/// Event indicating a wound to a body location
/// See https://www.ncbi.nlm.nih.gov/books/NBK380/
#[derive(Debug, Clone, PartialEq, EnumCount, EnumIs)]
pub enum AcuteWound<O: Organism> {
    Incision(WoundProperties<O>),
    Burn(WoundProperties<O>),
    Cut(WoundProperties<O>),
    Laceration(WoundProperties<O>),
    PressureUlcer(WoundProperties<O>),
    Puncture(WoundProperties<O>),
    Abrasion(WoundProperties<O>),
    Avulsion(WoundProperties<O>),
    Bruise(WoundProperties<O>),
}

impl<O: Organism> AcuteWound<O> {
    pub fn location(&self) -> O::AnatomyType {
        match self {
            Self::Incision(props) => props.location,
            Self::Burn(props) => props.location,
            Self::Cut(props) => props.location,
            Self::Laceration(props) => props.location,
            Self::PressureUlcer(props) => props.location,
            Self::Puncture(props) => props.location,
            Self::Abrasion(props) => props.location,
            Self::Avulsion(props) => props.location,
            Self::Bruise(props) => props.location,
        }
    }
    pub fn length(&self) -> Distance<NumType> {
        match self {
            Self::Incision(props) => props.length,
            Self::Burn(props) => props.length,
            Self::Cut(props) => props.length,
            Self::Laceration(props) => props.length,
            Self::PressureUlcer(props) => props.length,
            Self::Puncture(props) => props.length,
            Self::Abrasion(props) => props.length,
            Self::Avulsion(props) => props.length,
            Self::Bruise(props) => props.length,
        }
    }
    pub fn width(&self) -> Distance<NumType> {
        match self {
            Self::Incision(props) => props.width,
            Self::Burn(props) => props.width,
            Self::Cut(props) => props.width,
            Self::Laceration(props) => props.width,
            Self::PressureUlcer(props) => props.width,
            Self::Puncture(props) => props.width,
            Self::Abrasion(props) => props.width,
            Self::Avulsion(props) => props.width,
            Self::Bruise(props) => props.width,
        }
    }
    pub fn depth(&self) -> Distance<NumType> {
        match self {
            Self::Incision(props) => props.depth,
            Self::Burn(props) => props.depth,
            Self::Cut(props) => props.depth,
            Self::Laceration(props) => props.depth,
            Self::PressureUlcer(props) => props.depth,
            Self::Puncture(props) => props.depth,
            Self::Abrasion(props) => props.depth,
            Self::Avulsion(props) => props.depth,
            Self::Bruise(props) => props.depth,
        }
    }
    
    pub fn infections(&self) -> impl Iterator<Item = &Infection<O>> {
        match self {
            Self::Incision(props) => props.infections.iter(),
            Self::Burn(props) => props.infections.iter(),
            Self::Cut(props) => props.infections.iter(),
            Self::Laceration(props) => props.infections.iter(),
            Self::PressureUlcer(props) => props.infections.iter(),
            Self::Puncture(props) => props.infections.iter(),
            Self::Abrasion(props) => props.infections.iter(),
            Self::Avulsion(props) => props.infections.iter(),
            Self::Bruise(props) => props.infections.iter(),
        }
    }
}

impl<O: Organism> Event for AcuteWound<O> {}
