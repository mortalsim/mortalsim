use std::sync::Arc;
use crate::closed_circulation::{ClosedCirculationSim, ClosedCirculatorySystem};
use crate::core::sim::CoreSim;
mod circulation;
mod sim;
mod component;

pub use circulation::{HumanBloodVessel, HumanCirculatorySystem, HUMAN_CIRCULATION_FILEPATH};
pub type HumanClosedCirculationSim = ClosedCirculationSim<HumanBloodVessel>;

#[cfg(test)]
mod tests {
    use super::circulation::{HumanCirculatorySystem, HUMAN_CIRCULATION_FILEPATH};
    use super::HumanClosedCirculationSim;

    #[test]
    fn test_human_manager() {
        let _bm = HumanClosedCirculationSim::new(HumanCirculatorySystem::new());
    }
}
