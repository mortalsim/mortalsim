use std::collections::HashSet;

use crate::sim::layer::closed_circulation::vessel::test::TestBloodVessel;
use crate::sim::layer::nervous::nerve::test::TestNerve;

use super::Organism;

pub struct TestSim;

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

lazy_static! {
    static ref START_VESSELS: HashSet<TestBloodVessel> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestBloodVessel::Aorta);
        vessel_list
    };

    static ref ARTERIES: HashSet<TestBloodVessel> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestBloodVessel::Aorta);
        vessel_list
    };

    static ref VEINS: HashSet<TestBloodVessel> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestBloodVessel::VenaCava);
        vessel_list
    };

    static ref PRE_CAPILLARIES: HashSet<TestBloodVessel> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestBloodVessel::Aorta);
        vessel_list
    };

    static ref POST_CAPILLARIES: HashSet<TestBloodVessel> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestBloodVessel::VenaCava);
        vessel_list
    };

    static ref AORTA_UPSTREAM: HashSet<TestBloodVessel> = {
        HashSet::new()
    };

    static ref VENACAVA_UPSTREAM: HashSet<TestBloodVessel> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestBloodVessel::Aorta);
        vessel_list
    };

    static ref AORTA_DOWNSTREAM: HashSet<TestBloodVessel> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestBloodVessel::VenaCava);
        vessel_list
    };

    static ref VENACAVA_DOWNSTREAM: HashSet<TestBloodVessel> = {
        HashSet::new()
    };

    static ref AORTA_REGIONS: HashSet<TestAnatomicalRegion> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestAnatomicalRegion::Torso);
        vessel_list
    };

    static ref VENACAVA_REGIONS: HashSet<TestAnatomicalRegion> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestAnatomicalRegion::Torso);
        vessel_list
    };
}

