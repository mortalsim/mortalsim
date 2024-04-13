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
use crate::{secs, SimTimeSpan};

use crate::sim::impl_sim;
use super::{AnatomicalRegion, Organism};

#[derive(Debug, Clone, Copy)]
pub struct TestOrganism;

impl Organism for TestOrganism {
    type VesselType = TestBloodVessel;
    type NerveType = TestNerve;
    type AnatomyType = TestAnatomicalRegion;
}

impl_sim!(TestSim, TestOrganism);

#[test]
fn test_organism() {
    // Since we're setting "global" properties here,
    // we need to make sure the tests execute in sequence,
    // not parallel
    test_default();
    test_layers_init_run();
}

fn test_default() {
    let fid = TestSim::set_default(TestComponentA::new);

    let mut tsim = TestSim::new();
    assert!(tsim.add_component(TestComponentB::new()).is_ok());

    let mut sim: Box<dyn Sim> = Box::new(tsim);

    sim.advance();
    sim.advance_by(SimTimeSpan::from_s(1.0));
    assert_eq!(sim.active_components().len(), 2);
    assert!(sim.has_component("TestComponentA"));
    assert!(sim.has_component("TestComponentB"));
    assert!(!sim.has_component("not there"));
    assert!(sim.remove_component("test").is_err());
    sim.schedule_event(SimTimeSpan::from_s(0.0), Box::new(TestEventA::new(Distance::from_m(1.0))));
    assert!(sim.unschedule_event(&1234).is_err());
    assert_eq!(sim.time(), secs!(1.0));

    TestSim::remove_default(&fid).unwrap();
}

fn test_layers_init_run() {
    let fids = vec![
        TestSim::set_default(TestComponentA::new),
        TestSim::set_default(TestComponentB::new),
        TestSim::set_default(TestCircComponentA::new),
        TestSim::set_default(TestDigestionComponent::new),
        TestSim::set_default(TestPainReflexComponent::new),
        TestSim::set_default(TestMovementComponent::new),
    ];

    // Test the sequential version
    println!("creating test sim");
    let mut tsim = TestSim::new();

    println!("running test sim");
    for i in 1..10 {
        tsim.advance_by(SimTimeSpan::from_s(i.into()));
    }

    // test the threaded version
    println!("creating threaded test sim");
    let mut tsim = TestSim::new_threaded();

    println!("running threaded test sim");
    for i in 1..10 {
        tsim.advance_by(SimTimeSpan::from_s(i.into()));
    }

    for fid in fids {
        TestSim::remove_default(&fid).unwrap();
    }
}
