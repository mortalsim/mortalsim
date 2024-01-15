use super::layer::closed_circulation::BloodVessel;
use super::layer::nervous::Nerve;

#[cfg(test)]
pub mod test;

pub mod human;

pub trait Organism {
    type VesselType: BloodVessel;
    type NerveType: Nerve;
    type AnatomyType;
}
