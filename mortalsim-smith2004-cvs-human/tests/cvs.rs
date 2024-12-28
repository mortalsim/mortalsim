use mortalsim_core::event::{AorticBloodPressure, HeartRate, PulmonaryBloodPressure};
use mortalsim_core::sim::Sim;
use mortalsim_core::sim::component::SimComponent;
use mortalsim_core::units::mechanical::{Frequency, Pressure};
use mortalsim_core::SimTimeSpan;
use mortalsim_human::HumanSim;

use mortalsim_smith2004_cvs_human::params::Smith2004CvsConstantParam;
use mortalsim_smith2004_cvs_human::{Smith2004CvsComponent, Smith2004CvsParamChanges};

#[test_log::test]
fn component_run() {
    let mut comp = Smith2004CvsComponent::new();
    comp.run();
}

#[test_log::test]
fn with_test_sim() {
    let mut sim = HumanSim::new();
    let comp = Smith2004CvsComponent::new();
    sim.add_component(comp).expect("Should add successfully");

    assert!(sim.has_state::<AorticBloodPressure>());
    assert!(sim.has_state::<PulmonaryBloodPressure>());

    // Check that defaults are set
    let ao_bp = sim.get_state::<AorticBloodPressure>().unwrap().clone();
    assert!(ao_bp.systolic > Pressure::from_mmHg(0.0));
    assert!(ao_bp.diastolic > Pressure::from_mmHg(0.0));

    let p_bp = sim.get_state::<PulmonaryBloodPressure>().unwrap().clone();
    assert!(p_bp.systolic > Pressure::from_mmHg(0.0));
    assert!(p_bp.diastolic > Pressure::from_mmHg(0.0));

    // Check advance
    sim.advance_by(SimTimeSpan::from_s(10.0));

    // Check that new values are different
    let ao_bp_2 = sim.get_state::<AorticBloodPressure>().unwrap();
    assert!(ao_bp.systolic != ao_bp_2.systolic, "{} == {}", ao_bp.systolic, ao_bp_2.systolic);
    assert!(ao_bp.diastolic != ao_bp_2.diastolic);

    let p_bp_2 = sim.get_state::<PulmonaryBloodPressure>().unwrap();
    assert!(p_bp.systolic != p_bp_2.systolic);
    assert!(p_bp.diastolic != p_bp_2.diastolic);
}

#[test_log::test]
fn change_inputs() {
    let mut sim = HumanSim::new();
    let comp = Smith2004CvsComponent::new();
    sim.add_component(comp).expect("Should add successfully");

    sim.schedule_event(SimTimeSpan::from_s(0.0), Box::new(HeartRate(Frequency::from_Hz(65.0))));

    // Check advance
    sim.advance();

    // Check that new values are different
    let ao_bp = sim.get_state::<AorticBloodPressure>().unwrap().clone();
    assert!(ao_bp.systolic > Pressure::from_mmHg(0.0));
    assert!(ao_bp.diastolic > Pressure::from_mmHg(0.0));

    let mut param_changes = Smith2004CvsParamChanges::new();

    param_changes.set_param(Smith2004CvsConstantParam::R_sys, 2.0);

    sim.schedule_event(SimTimeSpan::from_s(0.0), Box::new(param_changes));

    // Advance to run the new sim
    sim.advance_by(SimTimeSpan::from_s(1.0));

    // Advance again to get the new results
    sim.advance_by(SimTimeSpan::from_s(10.0));

    // Increased systolic resistance should raise the systolic and diastolic pressure
    let ao_bp_2 = sim.get_state::<AorticBloodPressure>().unwrap().clone();
    assert!(ao_bp_2.systolic > ao_bp.systolic, "{} <= {}", ao_bp_2.systolic.to_mmHg(), ao_bp.systolic.to_mmHg());
    assert!(ao_bp_2.diastolic > ao_bp.diastolic, "{} <= {}", ao_bp_2.diastolic.to_mmHg(), ao_bp.diastolic.to_mmHg());
}