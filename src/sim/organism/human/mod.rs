mod anatomy;
pub use anatomy::HumanAnatomicalRegion;

mod human_circulation;
pub use human_circulation::HumanBloodVessel;

mod human_nervous;
pub use human_nervous::HumanNerve;

use crate::sim::layer::LayerManager;

use super::Organism;

pub struct HumanSim {
    layer_manager: LayerManager<Self>,
}

impl Organism for HumanSim {
    type VesselType = HumanBloodVessel;
    type NerveType = HumanNerve;
    type AnatomyType = HumanAnatomicalRegion;
}

// #[cfg(test)]
// mod tests {
//     use super::circulation::{HumanCirculatorySystem, HUMAN_CIRCULATION_FILEPATH};
//     use super::HumanCirculationSim;

//     #[test]
//     fn test_human_manager() {
//         let _bm = HumanCirculationSim::new(HumanCirculatorySystem::new());
//     }
// }
