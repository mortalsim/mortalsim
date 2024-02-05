use super::layer::circulation::BloodVessel;
use super::layer::nervous::Nerve;

mod impl_sim;
pub use impl_sim::*;

#[cfg(test)]
pub mod test;

pub mod human;

pub trait Organism: Send + Sync {
    type VesselType: BloodVessel;
    type NerveType: Nerve;
    type AnatomyType;
}
