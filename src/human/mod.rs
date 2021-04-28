use std::sync::Arc;
use crate::blood::{BloodManager, ClosedCirculatorySystem};
use crate::core::sim::Organism;
mod circulation;

pub use circulation::{HumanBloodVessel, HumanCirculatorySystem, HUMAN_CIRCULATION_FILEPATH};
pub type HumanBloodManager = BloodManager<HumanBloodVessel>;

lazy_static! {
    static ref CIRC_SYSTEM: HumanCirculatorySystem = HumanCirculatorySystem::from_json_file(HUMAN_CIRCULATION_FILEPATH).unwrap();
}

#[cfg(test)]
mod tests {
    use super::circulation::{HumanCirculatorySystem, HUMAN_CIRCULATION_FILEPATH};
    use super::HumanBloodManager;

    #[test]
    fn test_human_manager() {
        let _bm = HumanBloodManager::new(HumanCirculatorySystem::from_json_file(HUMAN_CIRCULATION_FILEPATH).unwrap());
    }
}
