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

pub use vessel::{BloodVessel, BloodVesselType, BloodVesselId};

#[derive(Clone, Debug)]
pub struct BloodNode<T: BloodVessel> {
    pub vessel: T,
    pub vessel_type: BloodVesselType,
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
