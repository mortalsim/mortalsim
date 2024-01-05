
use crate::sim::layer::closed_circulation::DummyVessel;

use super::Organism;

#[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
pub enum GenericAnatomicalRegion {
    Body
}

pub struct GenericSim {}

impl Organism for GenericSim {
    type VesselType = DummyVessel;
    type AnatomyType = GenericAnatomicalRegion;
}