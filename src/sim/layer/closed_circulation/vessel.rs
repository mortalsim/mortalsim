use std::collections::hash_set;
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;

pub trait BloodVessel:
    FromStr + Hash + Clone + Copy + Eq + fmt::Debug + fmt::Display + Send + Sync + Into<&'static str>
{
    type AnatomyType: Clone;
    fn start_vessels<'a>() -> VesselIter<'a, Self>;
    fn arteries<'a>() -> VesselIter<'a, Self>;
    fn veins<'a>() -> VesselIter<'a, Self>;
    fn pre_capillaries<'a>() -> VesselIter<'a, Self>;
    fn post_capillaries<'a>() -> VesselIter<'a, Self>;
    fn max_arterial_depth() -> u32;
    fn max_venous_depth() -> u32;
    fn max_cycle() -> u32;
    fn vessel_type(&self) -> BloodVesselType;
    fn upstream<'a>(&self) -> VesselIter<'a, Self>;
    fn downstream<'a>(&self) -> VesselIter<'a, Self>;
    fn regions<'a>(&self) -> AnatomicalRegionIter<Self::AnatomyType>;
}

/// Type of a blood vessel
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum BloodVesselType {
    Vein,
    Artery,
}

pub struct VesselIter<'a, V: BloodVessel>(pub hash_set::Iter<'a, V>);

impl<'a, V: BloodVessel> Iterator for VesselIter<'a, V> {
    type Item = V;
    fn next(&mut self) -> Option<V> {
        Some(self.0.next()?.clone())
    }
}

pub struct AnatomicalRegionIter<'a, T: Clone>(pub hash_set::Iter<'a, T>);

impl<'a, T: Clone> Iterator for AnatomicalRegionIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        Some(self.0.next()?.clone())
    }
}
