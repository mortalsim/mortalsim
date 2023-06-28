use super::super::HumanBloodVessel;
use crate::closed_circulation::ClosedCircInitializer;
use crate::core::sim::SimModuleInitializer;
use crate::event::Event;
use crate::substance::{MolarConcentration, Substance};
use std::collections::{HashMap, HashSet};

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
