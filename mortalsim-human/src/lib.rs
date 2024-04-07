#[macro_use]
extern crate strum_macros;

#[macro_use]
extern crate anyhow;

mod human_anatomy;
mod human_circulation;
mod human_nervous;

pub use human_circulation::HumanBloodVessel;
pub use human_anatomy::HumanAnatomicalRegion;
pub use human_nervous::HumanNerve;

use mortalsim_core::sim::{Organism, impl_sim};

#[derive(Debug, Clone, Copy)]
pub struct HumanOrganism;

impl Organism for HumanOrganism {
    type VesselType = HumanBloodVessel;
    type NerveType = HumanNerve;
    type AnatomyType = HumanAnatomicalRegion;
}

impl_sim!(HumanSim, HumanOrganism);
