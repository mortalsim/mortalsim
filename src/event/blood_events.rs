use crate::blood::BloodVessel;
use crate::substance::{Substance, MolarConcentration, Volume, Ratio};
use super::Event;

#[derive(Debug)]
pub struct BloodCompositionChange {
    pub vessel_id: &'static str,
    pub substance: Substance,
    pub previous_value: MolarConcentration,
    pub new_value: MolarConcentration,
}

impl Event for BloodCompositionChange {
    fn event_name(&self) -> &str {
        "BloodCompositionChange"
    }
}

#[derive(Debug)]
pub struct BloodVolumeRatioChange {
    pub vessel_id: &'static str,
    pub previous_value: Ratio,
    pub new_value: Ratio,
}

impl Event for BloodVolumeRatioChange {
    fn event_name(&self) -> &str {
        "BloodVolumeRatioChange"
    }
}