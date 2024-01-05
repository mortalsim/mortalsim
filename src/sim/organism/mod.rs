use super::layer::closed_circulation::BloodVessel;

#[cfg(test)]
pub mod test;

pub mod generic;
pub mod human;

pub trait Organism {
    type VesselType: BloodVessel;
    type AnatomyType;
}
