use std::sync::Arc;
use crate::blood::{ClosedCirculationManager, ClosedCirculatorySystem};
use crate::core::sim::Organism;
mod circulation;

pub use circulation::{HumanBloodVessel, HumanCirculatorySystem, HUMAN_CIRCULATION_FILEPATH};
pub type HumanClosedCirculationManager = ClosedCirculationManager<HumanBloodVessel>;

lazy_static! {
    static ref CIRC_SYSTEM: HumanCirculatorySystem = HumanCirculatorySystem::from_json_file(HUMAN_CIRCULATION_FILEPATH).unwrap();
}

#[cfg(test)]
mod tests {
    use super::circulation::{HumanCirculatorySystem, HUMAN_CIRCULATION_FILEPATH};
    use super::HumanClosedCirculationManager;

    #[test]
    fn test_human_manager() {
        let _bm = HumanClosedCirculationManager::new(HumanCirculatorySystem::from_json_file(HUMAN_CIRCULATION_FILEPATH).unwrap());
    }
}
