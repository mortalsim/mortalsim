#[macro_use]
extern crate anyhow;

use std::collections::HashSet;
use std::hash::Hash;
use std::sync::OnceLock;
use mortalsim_core::{impl_sim, sim::layer::{nervous::{Nerve, NerveIter}, AnatomicalRegionIter}};
use strum_macros::{Display, IntoStaticStr};

use mortalsim_core::sim::{layer::circulation::{BloodVessel, BloodVesselType, VesselIter}, AnatomicalRegion, Organism};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum SampleAnatomicalRegion {
    Body,
}

impl AnatomicalRegion for SampleAnatomicalRegion {}

static SAMPLE_REGIONS: OnceLock<HashSet<SampleAnatomicalRegion>> = OnceLock::new();

fn get_region_set() -> AnatomicalRegionIter<'static, SampleAnatomicalRegion>{
    AnatomicalRegionIter(SAMPLE_REGIONS.get_or_init(|| {
        let mut x = HashSet::new();
        x.insert(SampleAnatomicalRegion::Body);
        x
    }).iter())
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr)]
pub enum SampleBloodVessel {
    Aorta,
    VenaCava,
}

static AORTA_SET: OnceLock<HashSet<SampleBloodVessel>> = OnceLock::new();
static VENA_CAVA_SET: OnceLock<HashSet<SampleBloodVessel>> = OnceLock::new();
static EMPTY_VESSEL_SET: OnceLock<HashSet<SampleBloodVessel>> = OnceLock::new();

fn get_aorta_set() -> VesselIter<'static, SampleBloodVessel>{
    VesselIter(AORTA_SET.get_or_init(|| {
        let mut x = HashSet::new();
        x.insert(SampleBloodVessel::Aorta);
        x
    }).iter())
}

fn get_vc_set() -> VesselIter<'static, SampleBloodVessel>{
    VesselIter(VENA_CAVA_SET.get_or_init(|| {
        let mut x = HashSet::new();
        x.insert(SampleBloodVessel::Aorta);
        x
    }).iter())
}

fn get_empty_vessel_set() -> VesselIter<'static, SampleBloodVessel>{
    VesselIter(EMPTY_VESSEL_SET.get_or_init(|| HashSet::new()).iter())
}

impl BloodVessel for SampleBloodVessel {
    type AnatomyType = SampleAnatomicalRegion;

    fn arteries<'a>() -> mortalsim_core::sim::layer::circulation::VesselIter<'a, Self> {
        get_aorta_set()
    }

    fn upstream<'a>(&self) -> VesselIter<'a, Self> {
        match self {
            SampleBloodVessel::Aorta => get_empty_vessel_set(),
            SampleBloodVessel::VenaCava => get_aorta_set(),
        }
    }

    fn downstream<'a>(&self) -> VesselIter<'a, Self> {
        match self {
            SampleBloodVessel::Aorta => get_vc_set(),
            SampleBloodVessel::VenaCava => get_empty_vessel_set(),
        }
    }

    fn max_arterial_depth() -> u32 {
        1
    }

    fn max_cycle() -> u32 {
        2
    }

    fn max_venous_depth() -> u32 {
        1
    }

    fn post_capillaries<'a>() -> VesselIter<'a, Self> {
        get_vc_set()
    }

    fn pre_capillaries<'a>() -> VesselIter<'a, Self> {
        get_aorta_set()
    }

    fn regions<'a>(&self) -> AnatomicalRegionIter<Self::AnatomyType> {
        match self {
            _ => get_region_set(),
        }
    }

    fn start_vessels<'a>() -> VesselIter<'a, Self> {
        get_aorta_set()
    }

    fn veins<'a>() -> VesselIter<'a, Self> {
        get_vc_set()
    }

    fn vessel_type(&self) -> mortalsim_core::sim::layer::circulation::BloodVesselType {
        match self {
            Self::Aorta => BloodVesselType::Artery,
            Self::VenaCava => BloodVesselType::Vein,
        }
    }
}

#[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq)]
pub enum SampleNerve {
    Brain,
    SpinalCord,
}

static BRAIN_SET: OnceLock<Vec<SampleNerve>> = OnceLock::new();
static SPINALCORD_SET: OnceLock<Vec<SampleNerve>> = OnceLock::new();
static EMPTY_NERVE_SET: OnceLock<Vec<SampleNerve>> = OnceLock::new();

fn get_empty_nerve_set() -> NerveIter<'static, SampleNerve>{
    NerveIter(EMPTY_NERVE_SET.get_or_init(|| Vec::new()).iter())
}

fn get_brain_set() -> NerveIter<'static, SampleNerve>{
    NerveIter(BRAIN_SET.get_or_init(|| {
        let mut x = Vec::new();
        x.push(SampleNerve::Brain);
        x
    }).iter())
}

fn get_sc_set() -> NerveIter<'static, SampleNerve>{
    NerveIter(SPINALCORD_SET.get_or_init(|| {
        let mut x = Vec::new();
        x.push(SampleNerve::SpinalCord);
        x
    }).iter())
}

impl Nerve for SampleNerve {
    type AnatomyType = SampleAnatomicalRegion;

    fn terminal_nerves<'a>() -> NerveIter<'a, Self> {
        get_sc_set()
    }

    fn uplink<'a>(&self) -> NerveIter<'a, Self> {
        match self {
            Self::Brain => get_empty_nerve_set(),
            Self::SpinalCord => get_brain_set(),
        }
    }

    fn downlink<'a>(&self) -> NerveIter<'a, Self> {
        match self {
            Self::Brain => get_sc_set(),
            Self::SpinalCord => get_empty_nerve_set(),
        }
    }

    fn regions<'a>(&self) -> AnatomicalRegionIter<Self::AnatomyType> {
        match self {
            _ => get_region_set(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SampleOrganism;

impl Organism for SampleOrganism {
    type VesselType = SampleBloodVessel;
    type NerveType = SampleNerve;
    type AnatomyType = SampleAnatomicalRegion;
}

impl_sim!(SampleSim, SampleOrganism);

#[test]
fn test_sample_sim() {
    SampleSim::new();
}
