use std::collections::{HashMap, HashSet};
use anyhow::Result;
use petgraph::Direction;
use crate::core::sim::SimConnector;
use crate::substance::{Substance, SubstanceStore, MolarConcentration};
use super::super::super::{BloodVessel, BloodVesselType, VesselIter};
use super::super::{BloodNode, ClosedCirculationManager, ClosedCircVesselIter, ClosedCirculatorySystem, ClosedCircInitializer};

pub struct ClosedCircConnector<V: BloodVessel> {
    pub(crate) system: Option<ClosedCirculatorySystem<V>>,
    pub(crate) stores: HashMap<V, SubstanceStore>,
    pub(crate) vessel_connections: HashSet<V>,
    pub(crate) substance_notifies: HashMap<V, HashMap<Substance, MolarConcentration>>
}

impl<V: BloodVessel> ClosedCircConnector<V> {
    pub fn new(initializer: ClosedCircInitializer<V>) -> ClosedCircConnector<V> {
        ClosedCircConnector {
            system: None,
            stores: HashMap::new(),
            vessel_connections: initializer.vessel_connections,
            substance_notifies: initializer.substance_notifies,
        }
    }

    fn get_system(&self) -> &ClosedCirculatorySystem<V> {
        match &self.system {
            None => panic!("ClosedCirculatorySystem not set for ClosedCircConnector before execution!"),
            Some(v) => v
        }
    }

    pub fn depth(&self) -> u32 {
        self.get_system().depth as u32
    }

    pub fn blood_store(&self, vessel: &V) -> Option<&SubstanceStore> {
        let system = self.get_system();
        let node_idx = system.node_map.get(vessel)?;
        Some(&system.graph[*node_idx].composition)
    }

    pub fn vessel_type(&self, vessel: V) -> BloodVesselType {
        let system = self.get_system();
        let node_idx = system.node_map.get(&vessel).unwrap();
        system.graph[*node_idx].vessel_type
    }

    pub fn is_pre_capillary(&self, vessel: &V) -> bool {
        self.get_system().pre_capillaries.contains(vessel)
    }
    
    pub fn is_post_capillary(&self, vessel: &V) -> bool {
        self.get_system().post_capillaries.contains(vessel)
    }

    pub fn pre_capillaries(&self) -> VesselIter<V> {
        self.get_system().pre_capillaries.iter().into()
    }
    
    pub fn post_capillaries(&self) -> VesselIter<V> {
        self.get_system().post_capillaries.iter().into()
    }

    pub fn downstream(&self, vessel: V) -> ClosedCircVesselIter<V> {
        self.get_system().vessel_connections(vessel, Direction::Outgoing)
    }
    
    pub fn upstream(&self, vessel: V) -> ClosedCircVesselIter<V> {
        self.get_system().vessel_connections(vessel, Direction::Incoming)
    }
}
