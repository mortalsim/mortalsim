use super::layer::circulation::BloodVessel;
use super::layer::nervous::Nerve;

mod macros;
pub use macros::*;

#[cfg(test)]
pub mod test;

pub mod human;

pub trait Organism {
    type VesselType: BloodVessel;
    type NerveType: Nerve;
    type AnatomyType;
}
