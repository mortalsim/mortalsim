use std::fmt;
use std::hash::{Hash, Hasher};
use uom::si::volume::liter;
use crate::substance::{SubstanceStore, Volume};
use super::super::{BloodVessel, BloodVesselType};

/// Blood Vessel node within the circulation graph
#[derive(Clone, Debug)]
pub struct BloodNode<T: BloodVessel> {
    /// BloodVessel associated with this Node
    pub vessel: T,
    /// Whether it's an Artery or a Vein
    pub vessel_type: BloodVesselType,
    /// Current substance composition of the blood in the vessel
    pub composition: SubstanceStore,
}

impl<T: BloodVessel> BloodNode<T> {
    pub fn new(vessel: T, vessel_type: BloodVesselType) -> BloodNode<T> {
        BloodNode {
            vessel: vessel,
            vessel_type: vessel_type,
            composition: SubstanceStore::new(Volume::new::<liter>(1.0)),
        }
    }
}

// Hash only by the vessel_id
impl<T: BloodVessel> Hash for BloodNode<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vessel.hash(state);
    }
}

impl<T: BloodVessel> fmt::Display for BloodNode<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.vessel)
    }
}

/// Blood Vessel edge within the circulation graph
#[derive(Clone, Debug)]
pub struct BloodEdge {
    /// Percentage of source node blood flowing into this edge
    pub incoming_pct: f32,
    /// Percentage of target node blood flowing out from this edge
    pub outgoing_pct: f32,
}

impl BloodEdge {
    pub fn new() -> BloodEdge {
        BloodEdge {
            incoming_pct: 0.0,
            outgoing_pct: 0.0,
        }
    }
}

impl fmt::Display for BloodEdge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(out: {}, in: {})", self.outgoing_pct, self.incoming_pct)
    }
}