use std::collections::{HashSet, HashMap};
use crate::substance::{Substance, MolarConcentration};
use crate::core::sim::SimModuleInitializer;
use crate::closed_circulation::ClosedCircInitializer;
use crate::event::Event;
use super::super::HumanBloodVessel;

pub struct HumanModuleInitializer {
    pub(crate) initializer: SimModuleInitializer,
    pub(crate) cc_initializer: ClosedCircInitializer<HumanBloodVessel>,
}

impl HumanModuleInitializer {
    pub fn new() -> HumanModuleInitializer {
        HumanModuleInitializer {
            initializer: SimModuleInitializer::new(),
            cc_initializer: ClosedCircInitializer::new(),
        }
    }
}
