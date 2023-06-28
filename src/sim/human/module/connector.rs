use super::super::HumanBloodVessel;
use super::initializer::HumanModuleInitializer;
use crate::closed_circulation::ClosedCircConnector;
use crate::core::sim::SimConnector;
use crate::substance::{MolarConcentration, Substance, SubstanceStore};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

pub struct HumanSimConnector {
    connector: SimConnector,
    blood_connector: ClosedCircConnector<HumanBloodVessel>,
}

impl HumanSimConnector {
    pub fn new(
        connector: SimConnector,
        cc_connector: ClosedCircConnector<HumanBloodVessel>,
    ) -> HumanSimConnector {
        HumanSimConnector {
            connector: connector,
            blood_connector: cc_connector,
        }
    }
}
