mod human_anatomy;
pub use human_anatomy::HumanAnatomicalRegion;

mod human_circulation;
pub use human_circulation::HumanBloodVessel;

mod human_nervous;
pub use human_nervous::HumanNerve;

use crate::sim::layer::LayerManager;

use super::{impl_sim, Organism};

struct HumanOrganism;

impl Organism for HumanOrganism {
    type VesselType = HumanBloodVessel;
    type NerveType = HumanNerve;
    type AnatomyType = HumanAnatomicalRegion;
}

impl_sim!(HumanSim, HumanOrganism);
