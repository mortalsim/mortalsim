use std::cell::Cell;
use std::collections::HashSet;
use std::path::Component;
use std::sync::{Mutex, MutexGuard, OnceLock};


use crate::sim::layer::circulation::vessel::test::TestBloodVessel;
use crate::sim::layer::core::test::TestComponentA;
use crate::sim::layer::nervous::nerve::test::TestNerve;

use super::{impl_sim, Organism};

impl_sim!(TestSim);

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

#[test]
fn test_default() {
    TestSim::set_default(|| {
        TestComponentA::new()
    });
}