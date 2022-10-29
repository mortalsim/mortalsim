use std::sync::Arc;
use crate::blood::{ClosedCirculationManager, ClosedCirculatorySystem};
use crate::core::sim::CoreSim;
mod circulation;
mod sim;
mod component;

pub use circulation::{HumanBloodVessel, HumanCirculatorySystem, HUMAN_CIRCULATION_FILEPATH};
pub type HumanClosedCirculationManager = ClosedCirculationManager<HumanBloodVessel>;

#[cfg(test)]
mod tests {
    use super::circulation::{HumanCirculatorySystem, HUMAN_CIRCULATION_FILEPATH};
    use super::HumanClosedCirculationManager;

    #[test]
    fn test_human_manager() {
        let _bm = HumanClosedCirculationManager::new(HumanCirculatorySystem::new());
    }
}
