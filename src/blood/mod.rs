use std::rc::Rc;
use std::hash::{Hash, Hasher};
use std::fmt;
use std::collections::HashMap;
use std::borrow::Borrow;
use crate::substance::{SubstanceStore, Volume};
use uom::si::volume::liter;

mod manager;
mod circulation;
mod vessel;

pub use vessel::BloodVesselType;
pub use vessel::BloodVesselId;

#[derive(Clone, Debug)]
pub struct BloodNode {
    pub vessel_id: BloodVesselId,
    pub vessel_type: BloodVesselType,
    pub composition: SubstanceStore,
}

impl BloodNode {
    pub fn new(vessel_id: BloodVesselId, vessel_type: BloodVesselType) -> BloodNode {
        BloodNode {
            vessel_id: vessel_id,
            vessel_type: vessel_type,
            composition: SubstanceStore::new(Volume::new::<liter>(1.0)),
        }
    }
}

// Hash only by the vessel_id
impl Hash for BloodNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vessel_id.hash(state);
    }
}

impl fmt::Display for BloodNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.vessel_id)
    }
}

#[derive(Clone, Debug)]
pub struct BloodEdge {
    pub incoming_pct: f32,
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
