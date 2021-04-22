use crate::blood::manager::BloodManager;
pub(crate) mod circulation;

pub use circulation::HumanBloodVessel;
pub type HumanBloodManager = BloodManager<HumanBloodVessel>;

#[cfg(test)]
mod tests {
    use super::circulation::{HumanCirculationDef, HUMAN_CIRCULATION_FILEPATH};
    use super::HumanBloodManager;

    #[test]
    fn test_human_manager() {
        let _bm = HumanBloodManager::new(HumanCirculationDef::from_json_file(HUMAN_CIRCULATION_FILEPATH).unwrap());
    }
}
