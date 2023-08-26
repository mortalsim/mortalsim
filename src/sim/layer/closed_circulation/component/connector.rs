use super::super::vessel::{BloodVessel, BloodVesselType, VesselIter};
use crate::substance::{SubstanceConcentration, Substance, SubstanceStore};
use super::ClosedCircInitializer;
use anyhow::Result;
use petgraph::Direction;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct ClosedCircConnector<V: BloodVessel> {
    pub(crate) vessel_connections: HashMap<V, SubstanceStore>,
    pub(crate) substance_notifies: HashMap<V, HashMap<Substance, SubstanceConcentration>>,
}

impl<V: BloodVessel> ClosedCircConnector<V> {
    pub fn new(
        initializer: ClosedCircInitializer<V>,
    ) -> ClosedCircConnector<V> {
        ClosedCircConnector {
            vessel_connections: initializer.vessel_connections,
            substance_notifies: initializer.substance_notifies,
        }
    }

    pub fn blood_store(&self, vessel: &V) -> Option<&SubstanceStore> {
        self.vessel_connections.get(vessel)
    }
}
