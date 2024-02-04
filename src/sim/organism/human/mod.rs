mod anatomy;
pub use anatomy::HumanAnatomicalRegion;

mod human_circulation;
pub use human_circulation::HumanBloodVessel;

mod human_nervous;
pub use human_nervous::HumanNerve;

use crate::sim::layer::LayerManager;

use super::{impl_sim, Organism};

impl_sim!(HumanSim);

impl Organism for HumanSim {
    type VesselType = HumanBloodVessel;
    type NerveType = HumanNerve;
    type AnatomyType = HumanAnatomicalRegion;
}
