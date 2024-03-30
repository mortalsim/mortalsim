mod test_anatomy;
mod test_circulation;
mod test_nervous;

pub use test_circulation::TestBloodVessel;
pub use test_anatomy::TestAnatomicalRegion;
pub use test_nervous::TestNerve;

use std::any::Any;
use std::cell::Cell;
use std::collections::HashSet;
use std::path::Component;
use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::sim::layer::circulation::component::test::TestCircComponentA;
use crate::sim::layer::digestion::component::test::TestDigestionComponent;
use crate::sim::layer::nervous::component::test::{TestMovementComponent, TestPainReflexComponent};
use crate::units::base::Distance;

use crate::event::test::TestEventA;
use crate::sim::layer::core::component::test::{TestComponentA, TestComponentB};
use crate::sim::{Sim, SimTime};
use crate::util::secs;

use super::{impl_sim, AnatomicalRegion, Organism};

#[derive(Debug, Clone, Copy)]
pub struct TestOrganism;

impl Organism for TestOrganism {
    type VesselType = TestBloodVessel;
    type NerveType = TestNerve;
    type AnatomyType = TestAnatomicalRegion;
}

impl_sim!(TestSim, TestOrganism);

#[test]
fn test_default() {
    TestSim::set_default(TestComponentA::new);

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

#[test]
fn test_layers_init_run() {
    TestSim::set_default(TestComponentA::new);
    TestSim::set_default(TestComponentB::new);
    TestSim::set_default(TestCircComponentA::new);
    TestSim::set_default(TestDigestionComponent::new);
    TestSim::set_default(TestPainReflexComponent::new);
    TestSim::set_default(TestMovementComponent::new);

    let mut tsim = TestSim::new();

    for i in 1..10 {
        tsim.advance_by(SimTime::from_s(i.into()));
    }
}


#[test]
fn test_layers_init_run_sync() {
    TestSim::set_default(TestComponentA::new);
    TestSim::set_default(TestComponentB::new);
    TestSim::set_default(TestCircComponentA::new);
    TestSim::set_default(TestDigestionComponent::new);
    TestSim::set_default(TestPainReflexComponent::new);
    TestSim::set_default(TestMovementComponent::new);

    let mut tsim = TestSim::new_threaded();

    for i in 1..10 {
        tsim.advance_by(SimTime::from_s(i.into()));
    }
}
