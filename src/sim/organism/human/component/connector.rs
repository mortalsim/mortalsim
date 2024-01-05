use super::super::HumanBloodVessel;
use super::initializer::HumanModuleInitializer;
use crate::sim::layer::core::CoreConnector;
use crate::sim::layer::closed_circulation::ClosedCircConnector;
use crate::sim::organism::human::HumanSim;
use crate::substance::{Substance, SubstanceStore};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

pub type HumanCircConnector = ClosedCircConnector<HumanSim>;

pub struct HumanSimConnector {
    core: CoreConnector,
    circ: HumanCircConnector,
}

impl HumanSimConnector {
    pub fn new(
        core: CoreConnector,
        circ: HumanCircConnector,
    ) -> HumanSimConnector {
        HumanSimConnector {
            core,
            circ,
        }
    }
}
