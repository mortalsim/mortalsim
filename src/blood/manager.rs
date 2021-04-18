use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;
use std::string;
use petgraph::graph::{Graph, NodeIndex};
use super::{BloodNode, BloodEdge, BloodVesselType, VesselId};
use crate::substance::{SubstanceStore, Volume};
use super::circulation::CirculationDef;

pub struct BloodManager {
    graph: Graph<BloodNode, BloodEdge>,
    node_map: HashMap<VesselId, NodeIndex>,
    depth: u32,
}

impl BloodManager {
    /// Creates a BloodManager from a Graph representing the circulatory structure
    pub fn new(circulation: CirculationDef) -> BloodManager {
        BloodManager {
            graph: circulation.graph,
            node_map: circulation.node_map,
            depth: circulation.depth,
        }
    }

    /// Retrieves the maximum depth of the circulation tree (from root to capillary)
    pub fn depth(&self) -> u32 {
        self.depth
    }

    pub fn vessel_type(&self, vessel_id: &str) -> Option<BloodVesselType> {
        let node_idx = self.node_map.get(vessel_id)?;
        Some(self.graph[*node_idx].vessel_type)
    }
}