use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::collections::hash_set;
use std::rc::Rc;
use std::string;
use petgraph::Direction;
use petgraph::graph::{Graph, NodeIndex, Neighbors};
use crate::substance::{SubstanceStore, Volume};
use super::super::{BloodVessel, BloodVesselType, VesselIter};
use super::{BloodNode, BloodEdge, ClosedCirculatorySystem, ClosedCircVesselIter, ClosedCircConnector, ClosedCircSimConnector};

pub struct ClosedCirculationManager<V: BloodVessel> {
    graph: Graph<BloodNode<V>, BloodEdge>,
    node_map: HashMap<V, NodeIndex>,
    pre_capillaries: HashSet<V>,
    post_capillaries: HashSet<V>,
    depth: u8,
}

impl<V: BloodVessel> ClosedCirculationManager<V> {
    /// Creates a ClosedCirculationManager from a Graph representing the circulatory structure
    pub fn new(circulation: ClosedCirculatorySystem<V>) -> ClosedCirculationManager<V> {
        ClosedCirculationManager {
            graph: circulation.graph,
            node_map: circulation.node_map,
            pre_capillaries: circulation.pre_capillaries,
            post_capillaries: circulation.post_capillaries,
            depth: circulation.depth,
        }
    }

    /// Internal function for retrieving a vessel connection iterator
    /// (upstream or downstream)
    fn vessel_connections(&self, vessel: V, dir: Direction) -> ClosedCircVesselIter<V> {
        match self.node_map.get(&vessel) {
            Some(node_idx) => {
                ClosedCircVesselIter {
                    graph: &self.graph,
                    idx_iter: Some(self.graph.neighbors_directed(node_idx.clone(), dir))
                }
            },
            None => {
                ClosedCircVesselIter {
                    graph: &self.graph,
                    idx_iter: None
                }
            }
        }
    }
}

impl<V: BloodVessel> ClosedCircConnector<V> for ClosedCirculationManager<V> {
    /// Retrieves the SubstanceStore for the given vessel
    fn composition(&self, vessel: V) -> &SubstanceStore {
        let node_idx = self.node_map.get(&vessel).unwrap();
        &self.graph[*node_idx].composition
    }

    /// Retrieves a mutable SubstanceStore for the given vessel
    fn composition_mut(&mut self, vessel: V) -> &mut SubstanceStore {
        let node_idx = self.node_map.get(&vessel).unwrap();
        &mut self.graph[*node_idx].composition
    }

}

impl<V: BloodVessel> ClosedCircSimConnector<V> for ClosedCirculationManager<V> {
    /// Retrieves the maximum depth of the circulation tree (from root to capillary)
    fn depth(&self) -> u32 {
        self.depth as u32
    }

    /// Returns the BloodVesselType for the given vessel. Panics if the vessel is invalid
    fn vessel_type(&self, vessel: V) -> BloodVesselType {
        let node_idx = self.node_map.get(&vessel).unwrap();
        self.graph[*node_idx].vessel_type
    }

    /// Determines whether the given vessel is a pre-capillary vessel
    /// (Artery with no more downstream arteries, only veins)
    fn is_pre_capillary(&self, vessel: &V) -> bool {
        self.pre_capillaries.contains(vessel)
    }
    
    /// Determines whether the given vessel is a post-capillary vessel
    /// (Vein with no more upstream veins, only arteries)
    fn is_post_capillary(&self, vessel: &V) -> bool {
        self.post_capillaries.contains(vessel)
    }

    /// Retrieves an iterator of pre-capillary vessels
    /// (Arteries with no more downstream arteries, only veins)
    fn pre_capillaries(&self) -> VesselIter<V> {
        self.pre_capillaries.iter().into()
    }
    
    /// Retrieves an iterator of post-capillary vessels
    /// (Veins with no more upstream veins, only arteries)
    fn post_capillaries(&self) -> VesselIter<V> {
        self.post_capillaries.iter().into()
    }

    /// Retrieves an iterator over all downstream vessels from
    /// the provided vessel
    fn downstream(&self, vessel: V) -> ClosedCircVesselIter<V> {
        self.vessel_connections(vessel, Direction::Outgoing)
    }
    
    /// Retrieves an iterator over all upstream vessels from
    /// the provided vessel
    fn upstream(&self, vessel: V) -> ClosedCircVesselIter<V> {
        self.vessel_connections(vessel, Direction::Incoming)
    }

}

#[cfg(test)]
mod tests {
    use super::ClosedCirculationManager;
    #[test]
    fn test_manager() {

    }
}
