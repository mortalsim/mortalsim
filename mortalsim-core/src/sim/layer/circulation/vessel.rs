use std::collections::hash_set;
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;

use crate::sim::layer::AnatomicalRegionIter;

pub trait BloodVessel:
    Hash + Clone + Copy + Eq + fmt::Debug + Send
{
    type AnatomyType: Clone;
    fn max_arterial_depth() -> u32;
    fn max_venous_depth() -> u32;
    fn max_cycle() -> u32;
    fn start_vessels<'a>() -> VesselIter<'a, Self>;
    fn arteries<'a>() -> VesselIter<'a, Self>;
    fn veins<'a>() -> VesselIter<'a, Self>;
    fn pre_capillaries<'a>() -> VesselIter<'a, Self>;
    fn post_capillaries<'a>() -> VesselIter<'a, Self>;
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

impl<'a, V: BloodVessel> ExactSizeIterator for VesselIter<'a, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
pub enum DummyVessel {}

impl BloodVessel for DummyVessel {
    type AnatomyType = i8;
    fn start_vessels<'a>() -> VesselIter<'a, Self> {
        panic!()
    }
    fn arteries<'a>() -> VesselIter<'a, Self> {
        panic!()
    }
    fn veins<'a>() -> VesselIter<'a, Self> {
        panic!()
    }
    fn pre_capillaries<'a>() -> VesselIter<'a, Self> {
        panic!()
    }
    fn post_capillaries<'a>() -> VesselIter<'a, Self> {
        panic!()
    }
    fn max_arterial_depth() -> u32 {
        panic!()
    }
    fn max_venous_depth() -> u32 {
        panic!()
    }
    fn max_cycle() -> u32 {
        panic!()
    }
    fn vessel_type(&self) -> BloodVesselType {
        panic!()
    }
    fn upstream<'a>(&self) -> VesselIter<'a, Self> {
        panic!()
    }
    fn downstream<'a>(&self) -> VesselIter<'a, Self> {
        panic!()
    }
    fn regions<'a>(&self) -> AnatomicalRegionIter<Self::AnatomyType> {
        panic!()
    }
}
