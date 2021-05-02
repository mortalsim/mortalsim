use std::collections::HashMap;
use anyhow::Result;
use crate::core::sim::SimConnector;
use crate::substance::SubstanceStore;
use super::super::super::{BloodVessel, BloodVesselType, VesselIter};
use super::super::{BloodNode, ClosedCirculationManager, ClosedCircVesselIter};

pub trait ClosedCircConnector<V: BloodVessel> {
    /// Retrieves the SubstanceStore for the given vessel
    fn composition(&self, vessel: V) -> &SubstanceStore;

    /// Retrieves a mutable SubstanceStore for the given vessel
    fn composition_mut(&mut self, vessel: V) -> &mut SubstanceStore;
}

pub trait ClosedCircSimConnector<V: BloodVessel>: ClosedCircConnector<V> {
    /// Retrieves the maximum depth of the circulation tree (from root to capillary)
    fn depth(&self) -> u32;

    /// Returns the BloodVesselType for the given vessel. Panics if the vessel is invalid
    fn vessel_type(&self, vessel: V) -> BloodVesselType;
    
    /// Determines whether the given vessel is a pre-capillary vessel
    /// (Artery with no more downstream arteries, only veins)
    fn is_pre_capillary(&self, vessel: &V) -> bool;
    
    /// Determines whether the given vessel is a post-capillary vessel
    /// (Vein with no more upstream veins, only arteries)
    fn is_post_capillary(&self, vessel: &V) -> bool;

    /// Retrieves an iterator of pre-capillary vessels
    /// (Arteries with no more downstream arteries, only veins)
    fn pre_capillaries(&self) -> VesselIter<V>;
    
    /// Retrieves an iterator of post-capillary vessels
    /// (Veins with no more upstream veins, only arteries)
    fn post_capillaries(&self) -> VesselIter<V>;

    /// Retrieves an iterator over all downstream vessels from
    /// the provided vessel
    fn downstream(&self, vessel: V) -> ClosedCircVesselIter<V>;

    /// Retrieves an iterator over all upstream vessels from
    /// the provided vessel
    fn upstream(&self, vessel: V) -> ClosedCircVesselIter<V>;
}
