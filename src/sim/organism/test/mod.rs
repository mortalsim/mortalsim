use std::collections::HashSet;

pub use crate::sim::layer::circulation::vessel::test::TestBloodVessel;
pub use crate::sim::layer::nervous::nerve::test::TestNerve;
use crate::sim::layer::LayerManager;
use crate::sim::SimConnector;

use super::Organism;

pub struct TestSim {
    connector: SimConnector,
    layer_manager: LayerManager<Self>,
}

impl Organism for TestSim {
    type VesselType = TestBloodVessel;
    type NerveType = TestNerve;
    type AnatomyType = TestAnatomicalRegion;
}

#[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
pub enum TestAnatomicalRegion {
    Head,
    Torso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
}
