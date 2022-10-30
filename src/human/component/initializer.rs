use std::collections::{HashSet, HashMap};
use crate::substance::{Substance, MolarConcentration};
use crate::core::sim::SimComponentInitializer;
use crate::blood::ClosedCircInitializer;
use crate::event::Event;
use super::super::HumanBloodVessel;

pub struct HumanComponentInitializer {
    pub(crate) initializer: SimComponentInitializer,
    pub(crate) cc_initializer: ClosedCircInitializer<HumanBloodVessel>,
}

impl HumanComponentInitializer {
    pub fn new() -> HumanComponentInitializer {
        HumanComponentInitializer {
            initializer: SimComponentInitializer::new(),
            cc_initializer: ClosedCircInitializer::new(),
        }
    }
}
