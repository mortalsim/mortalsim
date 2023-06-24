use std::collections::{HashMap, HashSet};
use anyhow::Result;
use crate::core::sim::SimConnector;
use crate::closed_circulation::ClosedCircConnector;
use crate::substance::{Substance, SubstanceStore, MolarConcentration};
use super::initializer::HumanModuleInitializer;
use super::super::HumanBloodVessel;

pub struct HumanSimConnector {
    connector: SimConnector,
    blood_connector: ClosedCircConnector<HumanBloodVessel>,
}

impl HumanSimConnector {
    pub fn new(connector: SimConnector, cc_connector: ClosedCircConnector<HumanBloodVessel>) -> HumanSimConnector {
        HumanSimConnector {
            connector: connector,
            blood_connector: cc_connector
        }
    }
}
