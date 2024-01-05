mod anatomy;
pub use anatomy::HumanAnatomicalRegion;

mod human_circulation;
pub use human_circulation::HumanBloodVessel;

use super::Organism;

pub mod component;

pub struct HumanSim {}

impl Organism for HumanSim {
    type VesselType = HumanBloodVessel;
    type AnatomyType = HumanAnatomicalRegion;
}

// #[cfg(test)]
// mod tests {
//     use super::circulation::{HumanCirculatorySystem, HUMAN_CIRCULATION_FILEPATH};
//     use super::HumanClosedCirculationSim;

//     #[test]
//     fn test_human_manager() {
//         let _bm = HumanClosedCirculationSim::new(HumanCirculatorySystem::new());
//     }
// }
