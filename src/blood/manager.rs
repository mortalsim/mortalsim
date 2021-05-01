use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;
use std::string;
use petgraph::graph::{Graph, NodeIndex};
use super::{BloodNode, BloodEdge, BloodVessel, BloodVesselType};
use crate::substance::{SubstanceStore, Volume};
use super::ClosedCirculatorySystem;

pub struct BloodManager<T: BloodVessel> {
    graph: Graph<BloodNode<T>, BloodEdge>,
    node_map: HashMap<T, NodeIndex>,
    depth: u8,
}

impl<T: BloodVessel> BloodManager<T> {
    /// Creates a BloodManager from a Graph representing the circulatory structure
    pub fn new(circulation: ClosedCirculatorySystem<T>) -> BloodManager<T> {
        BloodManager {
            graph: circulation.graph,
            node_map: circulation.node_map,
            depth: circulation.depth,
        }
    }

    /// Retrieves the maximum depth of the circulation tree (from root to capillary)
    pub fn depth(&self) -> u32 {
        self.depth as u32
    }

    pub fn vessel_type(&self, vessel: T) -> Option<BloodVesselType> {
        let node_idx = self.node_map.get(&vessel)?;
        Some(self.graph[*node_idx].vessel_type)
    }

    pub fn composition(&self, vessel: T) -> Option<&SubstanceStore> {
        let node_idx = self.node_map.get(&vessel)?;
        Some(&self.graph[*node_idx].composition)
    }

    pub fn composition_mut(&mut self, vessel: T) -> Option<&mut SubstanceStore> {
        let node_idx = self.node_map.get(&vessel)?;
        Some(&mut self.graph[*node_idx].composition)
    }
}

#[cfg(test)]
mod tests {
    use super::BloodManager;
    #[test]
    fn test_manager() {

    }
}
