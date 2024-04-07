use std::fmt::Debug;

use super::layer::circulation::BloodVessel;
use super::layer::nervous::Nerve;

pub trait AnatomicalRegion: Debug + Copy + PartialEq + Eq + Send + Sync {}

pub trait Organism: Debug + Send + Clone + Copy + 'static {
    type VesselType: BloodVessel;
    type NerveType: Nerve;
    type AnatomyType: AnatomicalRegion;
}

#[cfg(test)]
pub mod test;
