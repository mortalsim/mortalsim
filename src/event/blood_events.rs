use super::Event;
use crate::closed_circulation::BloodVessel;
use crate::substance::{AmountOfSubstance, MolarConcentration, Substance, Volume};

#[derive(Debug)]
pub struct BloodCompositionChange<V: BloodVessel> {
    pub vessel: V,
    pub substance: Substance,
    pub change: MolarConcentration,
}

impl<V: BloodVessel + 'static> Event for BloodCompositionChange<V> {
    fn event_name(&self) -> &str {
        "BloodCompositionChange"
    }
}

#[derive(Debug)]
pub struct BloodVolumeChange<V: BloodVessel> {
    pub vessel: V,
    pub change: Volume,
}

impl<V: BloodVessel + 'static> Event for BloodVolumeChange<V> {
    fn event_name(&self) -> &str {
        "BloodVolumeChange"
    }
}
