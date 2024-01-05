use std::collections::HashSet;

use crate::sim::layer::closed_circulation::{BloodVessel, VesselIter, BloodVesselType, AnatomicalRegionIter};

use super::Organism;

pub struct TestSim;

impl Organism for TestSim {
    type VesselType = TestBloodVessel;
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
}

lazy_static! {
    static ref ARTERIES: HashSet<TestBloodVessel> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestBloodVessel::Aorta);
        vessel_list
    };
}

lazy_static! {
    static ref VEINS: HashSet<TestBloodVessel> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestBloodVessel::VenaCava);
        vessel_list
    };
}

lazy_static! {
    static ref PRE_CAPILLARIES: HashSet<TestBloodVessel> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestBloodVessel::Aorta);
        vessel_list
    };
}

lazy_static! {
    static ref POST_CAPILLARIES: HashSet<TestBloodVessel> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestBloodVessel::VenaCava);
        vessel_list
    };
}

lazy_static! {
    static ref AORTA_UPSTREAM: HashSet<TestBloodVessel> = {
        HashSet::new()
    };
}

lazy_static! {
    static ref VENACAVA_UPSTREAM: HashSet<TestBloodVessel> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestBloodVessel::Aorta);
        vessel_list
    };
}

lazy_static! {
    static ref AORTA_DOWNSTREAM: HashSet<TestBloodVessel> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestBloodVessel::VenaCava);
        vessel_list
    };
}

lazy_static! {
    static ref VENACAVA_DOWNSTREAM: HashSet<TestBloodVessel> = {
        HashSet::new()
    };
}

lazy_static! {
    static ref AORTA_REGIONS: HashSet<TestAnatomicalRegion> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestAnatomicalRegion::Torso);
        vessel_list
    };
}

lazy_static! {
    static ref VENACAVA_REGIONS: HashSet<TestAnatomicalRegion> = {
        let mut vessel_list = HashSet::new();
        vessel_list.insert(TestAnatomicalRegion::Torso);
        vessel_list
    };
}

#[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
pub enum TestBloodVessel {
    Aorta,
    VenaCava,
}

impl BloodVessel for TestBloodVessel {
    type AnatomyType = TestAnatomicalRegion;

    fn max_arterial_depth() -> u32 { 1 }
    fn max_venous_depth() -> u32 { 1 }
    fn max_cycle() -> u32 { 2 }
    fn start_vessels<'a>() -> VesselIter<'a, Self> {
        VesselIter(START_VESSELS.iter())
    }
    fn arteries<'a>() -> VesselIter<'a, Self> {
        VesselIter(ARTERIES.iter())
    }
    fn veins<'a>() -> VesselIter<'a, Self> {
        VesselIter(VEINS.iter())
    }
    fn pre_capillaries<'a>() -> VesselIter<'a, Self> {
        VesselIter(PRE_CAPILLARIES.iter())
    }
    fn post_capillaries<'a>() -> VesselIter<'a, Self> {
        VesselIter(POST_CAPILLARIES.iter())
    }
    fn vessel_type(&self) -> BloodVesselType {
        match self {
            TestBloodVessel::Aorta => BloodVesselType::Artery,
            TestBloodVessel::VenaCava => BloodVesselType::Vein,
        }
    }
    fn upstream<'a>(&self) -> VesselIter<'a, Self> {
        match self {
            TestBloodVessel::Aorta => VesselIter(AORTA_UPSTREAM.iter()),
            TestBloodVessel::VenaCava => VesselIter(VENACAVA_UPSTREAM.iter()),
        }
    }
    fn downstream<'a>(&self) -> VesselIter<'a, Self> {
        match self {
            TestBloodVessel::Aorta => VesselIter(AORTA_DOWNSTREAM.iter()),
            TestBloodVessel::VenaCava => VesselIter(VENACAVA_DOWNSTREAM.iter()),
        }
    }
    fn regions<'a>(&self) -> AnatomicalRegionIter<Self::AnatomyType> {
        match self {
            TestBloodVessel::Aorta => AnatomicalRegionIter(AORTA_REGIONS.iter()),
            TestBloodVessel::VenaCava => AnatomicalRegionIter(VENACAVA_REGIONS.iter()),
        }
    }
}
