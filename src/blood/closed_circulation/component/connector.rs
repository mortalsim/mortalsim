use std::collections::HashMap;
use anyhow::Result;
use crate::core::sim::SimConnector;
use crate::substance::SubstanceStore;
use super::super::super::{BloodNode, BloodVessel, BloodManager};

pub struct ClosedCircConnector<'a, V: BloodVessel> {
    blood_manager: &'a BloodManager<V>,
}

impl<'a, V: BloodVessel> ClosedCircConnector<'a, V> {
    pub fn composition(&self, vessel: V) -> Option<&SubstanceStore> {
        self.blood_manager.composition(vessel)
    }
    
    pub fn composition_mut(&mut self, vessel: V) -> Option<&mut SubstanceStore> {
        self.blood_manager.composition_mut(vessel)
    }
}

pub struct ClosedCircSimConnector<'a, V: BloodVessel> {
    connector: SimConnector,
    cc_connector: ClosedCircConnector<'a, V>,
}
