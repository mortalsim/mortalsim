use super::graph::{BloodEdge, BloodNode};
use super::vessel::{
    BloodVessel, BloodVesselType,
    BloodVesselType::{Artery, Vein},
};
use anyhow::Result;
use petgraph::dot::{Config, Dot};
use petgraph::graph::{EdgeIndex, Graph, Neighbors, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
// use super::vessel::BloodVesselType::{Vein, Artery};

pub struct ClosedCircVesselIter<'a, 'b, V: BloodVessel> {
    graph: &'a Graph<BloodNode<V>, BloodEdge>,
    idx_iter: Option<Neighbors<'b, BloodEdge>>,
}

impl<'a, 'b, V: BloodVessel> Iterator for ClosedCircVesselIter<'a, 'b, V> {
    type Item = V;
    fn next(&mut self) -> Option<V> {
        Some(self.graph[self.idx_iter.as_mut()?.next()?].vessel)
    }
}

#[derive(Clone)]
pub struct ClosedCirculatorySystem<V: BloodVessel> {
    /// Graph structure representing circulation anatomy
    pub graph: Graph<BloodNode<V>, BloodEdge>,
    /// Mapping from vessel id to node index for rapid lookup
    pub node_map: HashMap<V, NodeIndex>,
}

impl<T: BloodVessel> ClosedCirculatorySystem<T> {
    // /// Loads a circulation graph and corresponding vessel->idx map from the given file
    // pub fn from_json_file(filename: &str) -> Result<ClosedCirculatorySystem<T>> {
    //     let contents = match fs::read_to_string(filename) {
    //         Err(err) => panic!(
    //             "Error loading Circulatory System file '{}': {}",
    //             filename, err
    //         ),
    //         Ok(contents) => contents,
    //     };

    //     let circ_json: Value = serde_json::from_str(&contents)?;
    //     Ok(Self::from_json_value(&circ_json))
    // }

    // /// Parses a circulation graph and corresponding vessel->idx map from the given json string
    // pub fn from_json_value(circ_json: &Value) -> ClosedCirculatorySystem<T> {
    //     let mut circ = ClosedCirculatorySystem {
    //         graph: Graph::new(),
    //         node_map: HashMap::new(),
    //     };

    //     // Get each tree from the circulation definition
    //     let arterial_tree = &circ_json["arterial"];
    //     let venous_tree = &circ_json["venous"];

    //     // Add veins first, since arteries reference them at the capillaries
    //     circ.add_veins(venous_tree);
    //     circ.add_arteries(arterial_tree);
    //     circ.set_edge_weights();
    //     circ
    // }

    // /// Adds the venous tree to the circulation graph
    // fn add_veins(&mut self, vein: &Value) {
    //     let root_idx = self.add_node(vein, Vein);
    //     self.add_vessels(vein, root_idx, Vein, 1);
    // }

    // /// Adds the arterial tree to the circulation graph
    // fn add_arteries(&mut self, artery: &Value) {
    //     let root_idx = self.add_node(artery, Artery);
    //     self.add_vessels(artery, root_idx, Artery, 1);
    // }

    // /// Recursive function which adds BloodNodes to the circulation graph based on the JSON definition
    // fn add_vessels(
    //     &mut self,
    //     vessel: &Value,
    //     vessel_idx: NodeIndex,
    //     vessel_type: BloodVesselType,
    //     depth: u32,
    // ) {
    //     let links = &vessel["links"];

    //     // Set the depth to the maximum
    //     if depth > self.depth {
    //         self.depth = depth;
    //     }

    //     // If there are more nodes in the tree, keep adding them recursively
    //     if links.is_array() {
    //         for link_vessel in links.as_array().unwrap() {
    //             let link_idx = self.add_node(link_vessel, vessel_type);
    //             self.add_vessels(link_vessel, link_idx, vessel_type, depth + 1);

    //             if vessel_type == Artery {
    //                 self.graph.add_edge(vessel_idx, link_idx, BloodEdge::new());
    //             } else {
    //                 self.graph.add_edge(link_idx, vessel_idx, BloodEdge::new());
    //             }
    //         }
    //     }

    //     let bridge = &vessel["bridge"];

    //     if bridge.is_array() {
    //         // Note this vessel as a pre_capillary
    //         self.pre_capillaries.insert(self.graph[vessel_idx].vessel);

    //         // Establish an edge between the current Node and the venous Node, which
    //         // we'll look up from the map
    //         for bridge_id in bridge.as_array().unwrap() {
    //             let bridge_vessel = match T::from_str(bridge_id.as_str().unwrap()) {
    //                 Err(_) => panic!(
    //                     "Invalid BloodVessel variant: '{}'",
    //                     bridge_id.as_str().unwrap()
    //                 ),
    //                 Ok(val) => val,
    //             };

    //             match self.node_map.get(&bridge_vessel) {
    //                 Some(bridge_idx) => {
    //                     self.graph
    //                         .add_edge(vessel_idx, bridge_idx.clone(), BloodEdge::new());

    //                     // Note the target as a post_capillary
    //                     self.post_capillaries.insert(bridge_vessel);
    //                 }
    //                 None => {
    //                     panic!(
    //                         "Invalid bridge vessel for Node '{}': '{}'",
    //                         &self.graph[vessel_idx], bridge_vessel
    //                     )
    //                 }
    //             }
    //         }
    //     }
    // }

    // /// Adds a single node to the circulation graph
    // fn add_node(&mut self, vessel: &Value, vessel_type: BloodVesselType) -> NodeIndex {
    //     let vessel = match T::from_str(vessel["id"].as_str().unwrap()) {
    //         Err(_) => panic!(
    //             "Invalid BloodVessel variant: '{}'",
    //             vessel["id"].as_str().unwrap()
    //         ),
    //         Ok(val) => val,
    //     };

    //     // Nodes can be defined multiple times if needed to establish multiple upstream connections
    //     // so we'll grab the existing value here if it already exists
    //     self.node_map
    //         .entry(vessel)
    //         .or_insert(self.graph.add_node(BloodNode::new(vessel, vessel_type)))
    //         .clone()
    // }

    // /// Sets edge weights for which incoming and outgoing are set evenly based on the number
    // /// of inputs/outputs connected to the Node
    // /// TODO: May want to allow this to be configurable in the future
    // fn set_edge_weights(&mut self) {
    //     for idx in self.graph.node_indices() {
    //         let incoming_edges: Vec<EdgeIndex> = self
    //             .graph
    //             .edges_directed(idx, Direction::Incoming)
    //             .map(|x| x.id())
    //             .collect();
    //         let incoming_count = incoming_edges.len() as f32;
    //         for edge_id in incoming_edges {
    //             self.graph[edge_id].incoming_pct = 1.0 / incoming_count;
    //         }

    //         let outgoing_edges: Vec<EdgeIndex> = self
    //             .graph
    //             .edges_directed(idx, Direction::Outgoing)
    //             .map(|x| x.id())
    //             .collect();
    //         let outgoing_count = outgoing_edges.len() as f32;
    //         for edge_id in outgoing_edges {
    //             self.graph[edge_id].outgoing_pct = 1.0 / outgoing_count;
    //         }
    //     }
    // }

    // /// function for retrieving a vessel connection iterator (upstream or downstream)
    // pub fn vessel_connections(&self, vessel: T, dir: Direction) -> ClosedCircVesselIter<T> {
    //     match self.node_map.get(&vessel) {
    //         Some(node_idx) => ClosedCircVesselIter {
    //             graph: &self.graph,
    //             idx_iter: Some(self.graph.neighbors_directed(node_idx.clone(), dir)),
    //         },
    //         None => ClosedCircVesselIter {
    //             graph: &self.graph,
    //             idx_iter: None,
    //         },
    //     }
    // }

    // /// Internal function for computing the map of vessel ids to `NodeIndex` items for
    // /// rapid lookup later
    // fn compute_node_map(&mut self) {
    //     for node_idx in self.graph.node_indices() {
    //         self.node_map.insert(self.graph[node_idx].vessel, node_idx);
    //     }
    // }

    pub fn digraph(&self) -> String {
        Dot::with_config(&self.graph, &[Config::EdgeNoLabel]).to_string()
    }

    pub fn digraph_edges(&self) -> String {
        Dot::new(&self.graph).to_string()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::ClosedCirculatorySystem;
//     use crate::human::{HumanBloodVessel, HUMAN_CIRCULATION_FILEPATH};
//     use petgraph::dot::{Config, Dot};
//     use petgraph::graphmap::DiGraphMap;
//     use petgraph::stable_graph::StableDiGraph;
//     use serde_json::to_string_pretty;
//     use std::rc::Rc;
//     use std::time::{Duration, Instant};

//     #[test]
//     fn test_load() {
//         let circ: ClosedCirculatorySystem<HumanBloodVessel> =
//             ClosedCirculatorySystem::from_json_file(HUMAN_CIRCULATION_FILEPATH).unwrap();
//         println!("{}", circ.digraph());
//         println!("depth: {}", circ.depth);
//     }
// }
