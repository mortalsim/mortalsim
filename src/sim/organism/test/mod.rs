use std::any::Any;
use std::cell::Cell;
use std::collections::HashSet;
use std::path::Component;
use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::units::base::Distance;

use crate::event::test::TestEventA;
use crate::sim::layer::circulation::vessel::test::TestBloodVessel;
use crate::sim::layer::core::component::test::{TestComponentA, TestComponentB};
use crate::sim::layer::nervous::nerve::test::TestNerve;
use crate::sim::Sim;
use crate::util::secs;

use super::{impl_sim, Organism};

pub struct TestOrganism;

impl Organism for TestOrganism {
    type VesselType = TestBloodVessel;
    type NerveType = TestNerve;
    type AnatomyType = TestAnatomicalRegion;
}

impl_sim!(TestSim, TestOrganism);

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
    TestSim::set_default(|| TestComponentA::new());

    let mut tsim = TestSim::new();
    assert!(tsim.add_component(TestComponentB::new()).is_ok());

    let mut sim: Box<dyn Sim> = Box::new(tsim);

    sim.advance();
    sim.advance_by(secs!(1.0));
    assert_eq!(sim.active_components().len(), 2);
    assert!(sim.has_component("TestComponentA"));
    assert!(sim.has_component("TestComponentB"));
    assert!(!sim.has_component("not there"));
    assert!(sim.remove_component("test").is_err());
    sim.schedule_event(secs!(0.0), Box::new(TestEventA::new(Distance::from_m(1.0))));
    assert!(sim.unschedule_event(&1234).is_err());
    assert_eq!(sim.time(), secs!(1.0));
}
