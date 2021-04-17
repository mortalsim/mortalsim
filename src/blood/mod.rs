use std::rc::Rc;
use std::hash::{Hash, Hasher};
use std::fmt;

// pub mod blood_manager;
mod util;

/// Type of a blood vessel
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum BloodVesselType {
    Vein,
    Artery,
}

#[derive(Clone, Debug)]
pub struct BloodNode {
    pub vessel_id: Rc<String>,
    pub vessel_type: BloodVesselType,
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

/// A blood vessel of the circulatory system
trait BloodVessel {
    /// Retrieves a list of BloodVessel objects which are immediately upstream
    fn upstream(&self) -> Vec<&'static dyn BloodVessel>;
    
    /// Retrieves a list of mutable BloodVessel objects which are immediately downstream
    fn upstream_mut(&mut self) -> Vec<&'static mut dyn BloodVessel>;
    
    /// Retrieves a list of BloodVessel objects which are immediately downstream
    fn downstream(&self) -> Vec<&'static dyn BloodVessel>;
    
    /// Retrieves a list of mutable BloodVessel objects which are immediately downstream
    fn downstream_mut(&mut self) -> Vec<&'static mut dyn BloodVessel>;
}
