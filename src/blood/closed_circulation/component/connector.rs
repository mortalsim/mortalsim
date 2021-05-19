use std::collections::{HashMap, HashSet};
use anyhow::Result;
use crate::core::sim::SimConnector;
use crate::substance::{Substance, SubstanceStore, MolarConcentration};
use super::super::super::{BloodVessel, BloodVesselType, VesselIter};
use super::super::{BloodNode, ClosedCirculationManager, ClosedCircVesselIter, ClosedCircInitializer};

pub struct ClosedCircConnector<V: BloodVessel> {
    pub(crate) stores: HashMap<V, SubstanceStore>,
    pub(crate) vessel_connections: HashSet<V>,
    pub(crate) substance_notifies: HashMap<V, HashMap<Substance, MolarConcentration>>
}

impl<V: BloodVessel> ClosedCircConnector<V> {
    fn new(initializer: ClosedCircInitializer<V>) -> ClosedCircConnector<V> {
        ClosedCircConnector {
            stores: HashMap::new(),
            vessel_connections: initializer.vessel_connections,
            substance_notifies: initializer.substance_notifies,
        }
    }

    /// Retrieves the SubstanceStore for the given vessel
    fn blood_store(&self, vessel: &V) -> Option<&SubstanceStore> {
        self.stores.get(vessel)
    }
}

pub trait ClosedCircSimConnector<V: BloodVessel> {
    /// Retrieves the maximum depth of the circulation tree (from root to capillary)
    fn depth(&self) -> u32;

    /// Retrieves the SubstanceStore for the given vessel
    fn blood_store(&self, vessel: &V) -> Option<&SubstanceStore>;

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

    // Retrieves the SimConnector portion of this connector
    fn connector(&mut self) -> &mut SimConnector;
}
