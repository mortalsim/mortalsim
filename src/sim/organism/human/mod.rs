mod human_anatomy;
mod human_circulation;
mod human_nervous;

pub use human_circulation::HumanBloodVessel;
pub use human_anatomy::HumanAnatomicalRegion;
pub use human_nervous::HumanNerve;

use super::{impl_sim, Organism};

#[derive(Debug, Clone, Copy)]
pub struct HumanOrganism;

impl Organism for HumanOrganism {
    type VesselType = HumanBloodVessel;
    type NerveType = HumanNerve;
    type AnatomyType = HumanAnatomicalRegion;
}

impl_sim!(HumanSim, HumanOrganism);
