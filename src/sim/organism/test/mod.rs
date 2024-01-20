use std::collections::HashSet;

use crate::sim::layer::closed_circulation::{BloodVessel, VesselIter, BloodVesselType};
use crate::sim::layer::nervous::{Nerve, NerveIter};
use crate::sim::layer::AnatomicalRegionIter;

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

#[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
pub enum TestNerve {
    Brain,
    SpinalCord,
}

lazy_static! {
    static ref TERMINAL_NERVES: Vec<TestNerve> = {
        let mut nerve_list = Vec::new();
        nerve_list.push(TestNerve::SpinalCord);
        nerve_list
    };

    static ref BRAIN_UPLINK: Vec<TestNerve> = {
        Vec::new()
    };

    static ref SPINALCORD_UPLINK: Vec<TestNerve> = {
        let mut nerve_list = Vec::new();
        nerve_list.push(TestNerve::Brain);
        nerve_list
    };

    static ref BRAIN_DOWNLINK: Vec<TestNerve> = {
        let mut nerve_list = Vec::new();
        nerve_list.push(TestNerve::SpinalCord);
        nerve_list
    };

    static ref SPINALCORD_DOWNLINK: Vec<TestNerve> = {
        Vec::new()
    };

    static ref BRAIN_REGIONS: HashSet<TestAnatomicalRegion> = {
        let mut region_list = HashSet::new();
        region_list.insert(TestAnatomicalRegion::Head);
        region_list
    };

    static ref SPINALCORD_REGIONS: HashSet<TestAnatomicalRegion> = {
        let mut region_list = HashSet::new();
        region_list.insert(TestAnatomicalRegion::Torso);
        region_list
    };
}

impl Nerve for TestNerve {
    type AnatomyType = TestAnatomicalRegion;

    fn terminal_nerves<'a>() -> NerveIter<'a, Self> {
        NerveIter(TERMINAL_NERVES.iter())
    }

    fn uplink<'a>(&self) -> NerveIter<'a, Self> {
        match self {
            TestNerve::Brain => NerveIter(BRAIN_UPLINK.iter()),
            TestNerve::SpinalCord => NerveIter(SPINALCORD_UPLINK.iter()),
        }
    }

    fn downlink<'a>(&self) -> NerveIter<'a, Self> {
        match self {
            TestNerve::Brain => NerveIter(BRAIN_DOWNLINK.iter()),
            TestNerve::SpinalCord => NerveIter(SPINALCORD_DOWNLINK.iter()),
        }
    }

    fn regions<'a>(&self) -> AnatomicalRegionIter<Self::AnatomyType> {
        match self {
            TestNerve::Brain => AnatomicalRegionIter(BRAIN_REGIONS.iter()),
            TestNerve::SpinalCord => AnatomicalRegionIter(SPINALCORD_REGIONS.iter()),
        }
    }

}
