use std::hash::Hash;
use std::collections::hash_set;
use std::fmt;
use std::str::FromStr;

mod closed_circulation;
pub use closed_circulation::{ClosedCirculatorySystem, ClosedCirculationManager, ClosedCircInitializer, ClosedCircConnector, ClosedCircSimComponent};

/// Blood vessel identifier trait. Intended to be implemented by enums for various types of
/// simulated blood circulation systems (human, dog, cat, etc.)
pub trait BloodVessel: FromStr + Hash + Clone + Copy + Eq + fmt::Debug + fmt::Display + Send + Sync + Into<&'static str> {
    fn source() -> Self;
    fn sink() -> Self;
}

/// Type of a blood vessel
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum BloodVesselType {
    Vein,
    Artery,
}

pub struct VesselIter<'a, V: BloodVessel> {
    iter: hash_set::Iter<'a, V>
}

impl<'a, V: BloodVessel> Iterator for VesselIter<'a, V> {
    type Item = V;
    fn next(&mut self) -> Option<V> {
        Some(self.iter.next()?.clone())
    }
}

impl<'a, V: BloodVessel> From<hash_set::Iter<'a, V>> for VesselIter<'a, V> {
    fn from(set_iter: hash_set::Iter<'a, V>) -> VesselIter<'a, V> {
        VesselIter {
            iter: set_iter
        }
    }
}
