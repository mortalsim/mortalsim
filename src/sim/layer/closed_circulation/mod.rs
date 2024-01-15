use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Mutex;

mod closed_circulation_layer;
mod component;
pub mod vessel;

// pub use graph::{BloodEdge, BloodNode};
pub use component::{ClosedCircConnector, ClosedCircInitializer, ClosedCircComponent, BloodStore};
// pub use closed_circulation_layer::ClosedCirculationLayer;
// pub use system::{ClosedCircVesselIter, ClosedCirculatorySystem};
pub use vessel::{BloodVessel, VesselIter, BloodVesselType, DummyVessel};

