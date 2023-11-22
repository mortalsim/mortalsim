use super::super::HumanBloodVessel;
use crate::sim::layer::core::CoreComponentInitializer;
use crate::sim::layer::closed_circulation::ClosedCircInitializer;

pub type HumanCircInitializer = ClosedCircInitializer<HumanBloodVessel>;

pub struct HumanModuleInitializer {
    pub(crate) core: CoreComponentInitializer,
    pub(crate) circ: HumanCircInitializer,
}

impl HumanModuleInitializer {
    pub fn new() -> HumanModuleInitializer {
        HumanModuleInitializer {
            core: CoreComponentInitializer::new(),
            circ: HumanCircInitializer::new(),
        }
    }
}
