use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use anyhow::Result;
use petgraph::Direction;
use uom::si::volume::liter;
use crate::substance::Volume;
use crate::substance::{Substance, SubstanceStore, MolarConcentration};
use super::super::vessel::{BloodVessel, BloodVesselType, VesselIter};
use super::super::{BloodNode, ClosedCirculationSim, ClosedCircVesselIter, ClosedCirculatorySystem, ClosedCircInitializer};

pub struct ClosedCircConnector<V: BloodVessel> {
    pub(crate) system: Rc<ClosedCirculatorySystem<V>>,
    pub(crate) vessel_connections: HashMap<V, SubstanceStore>,
    pub(crate) substance_notifies: HashMap<V, HashMap<Substance, MolarConcentration>>,
}

impl<V: BloodVessel> ClosedCircConnector<V> {
    pub fn new(system: Rc<ClosedCirculatorySystem<V>>, initializer: ClosedCircInitializer<V>) -> ClosedCircConnector<V> {
        ClosedCircConnector {
            system: system,
            vessel_connections: initializer.vessel_connections,
            substance_notifies: initializer.substance_notifies,
        }
    }

    fn get_system(&self) -> &ClosedCirculatorySystem<V> {
        self.system.as_ref()
    }    

    pub fn depth(&self) -> u32 {
        self.get_system().depth as u32
    }

    pub fn blood_store(&self, vessel: &V) -> Option<&SubstanceStore> {
        self.vessel_connections.get(vessel)
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