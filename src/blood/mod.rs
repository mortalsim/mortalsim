use std::hash::Hash;
use std::fmt;
use std::str::FromStr;

mod manager;
mod graph;
mod closed_circulation;

use graph::{BloodEdge, BloodNode};
pub use manager::BloodManager;
pub use closed_circulation::ClosedCirculatorySystem;

/// Blood vessel identifier trait. Intended to be implemented by enums for various types of
/// simulated blood circulation systems (human, dog, cat, etc.)
pub trait BloodVessel: FromStr + Hash + Clone + Copy + PartialEq + Eq + fmt::Debug + fmt::Display {}

/// Type of a blood vessel
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum BloodVesselType {
    Vein,
    Artery,
}
