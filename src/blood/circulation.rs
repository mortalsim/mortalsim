use std::fmt;
use std::fs;
use std::rc::Rc;
use std::sync::Mutex;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use petgraph::Direction;
use petgraph::visit::EdgeRef;
use petgraph::graph::{Graph, NodeIndex, EdgeIndex};
use petgraph::dot::{Dot, Config};
use super::{BloodNode, BloodEdge, BloodVesselType, VesselId};
use super::BloodVesselType::{Vein, Artery};

#[derive(Clone)]
pub struct CirculationDef {
    /// Graph structure representing circulation anatomy
    pub graph: Graph<BloodNode, BloodEdge>,
    /// Mapping from vessel id to node index for rapid lookup
    pub node_map: HashMap<VesselId, NodeIndex>,
    /// Maximum depth of the circulation from root node to capillary
    pub depth: u32,
}

impl CirculationDef {
    /// Loads a circulation graph and corresponding vessel->idx map from the given file
    pub fn from_json_file(filename: &str) -> Result<CirculationDef> {
        let contents = fs::read_to_string(filename).unwrap();
        let circ_json: Value = serde_json::from_str(&contents)?;
        Ok(Self::from_json_value(&circ_json))
    }

    /// Parses a circulation graph and corresponding vessel->idx map from the given json string
    pub fn from_json_value(circ_json: &Value) -> CirculationDef {
        let mut circ = CirculationDef {
            graph: Graph::new(),
            node_map: HashMap::new(),
            depth: 0,
        };
        
        // Get each tree from the circulation definition

        let arterial_tree = &circ_json["arterial"];
        let venous_tree = &circ_json["venous"];

        // Add veins first, since arteries reference them at the capillaries
        circ.add_veins(venous_tree);
        circ.add_arteries(arterial_tree);
        circ.set_edge_weights();
        circ
    }

    /// Adds the venous tree to the circulation graph
    fn add_veins(&mut self, vein: &Value) {
        let root_idx = self.add_node(vein, Vein);
        self.add_vessels(vein, root_idx, Vein, 1);
    }

    /// Adds the arterial tree to the circulation graph
    fn add_arteries(&mut self, artery: &Value) {
        let root_idx = self.add_node(artery, Artery);
        self.add_vessels(artery, root_idx, Artery, 1);
    }

    /// Recursive function which adds BloodNodes to the circulation graph based on the JSON definition
    fn add_vessels(&mut self, vessel: &Value, vessel_idx: NodeIndex, vessel_type: BloodVesselType, depth: u32) {
        let links = &vessel["links"];

        // Set the depth to the maximum
        if depth > self.depth {
            self.depth = depth;
        }

        // If there are more nodes in the tree, keep adding them recursively
        if links.is_array() {
            for link_vessel in links.as_array().unwrap() {
                let link_idx = self.add_node(link_vessel, vessel_type);
                self.add_vessels(link_vessel, link_idx, vessel_type, depth + 1);

                if vessel_type == Artery {
                    self.graph.add_edge(vessel_idx, link_idx, BloodEdge::new());
                }
                else {
                    self.graph.add_edge(link_idx, vessel_idx, BloodEdge::new());
                }
            }
        }

        let bridge = &vessel["bridge"];

        if bridge.is_array() {
            // Establish an edge between the current Node and the venous Node, which
            // we'll look up from the map
            for bridge_id in bridge.as_array().unwrap() {
                let bridge_str = Rc::new(String::from(bridge_id.as_str().unwrap()));
                match self.node_map.get(&bridge_str) {
                    Some(bridge_idx) => {
                        self.graph.add_edge(vessel_idx, bridge_idx.clone(), BloodEdge::new());
                    },
                    None => {
                        panic!("Invalid bridge value for Node '{}': '{}'", &self.graph[vessel_idx], bridge_str)
                    }
                }
            }
        }
    }

    /// Adds a single node to the circulation graph
    fn add_node(&mut self, vessel: &Value, vessel_type: BloodVesselType) -> NodeIndex {
        let vessel_id: VesselId = vessel["id"].as_str().unwrap().into();

        // Nodes can be defined multiple times if needed to establish multiple upstream connections
        // so we'll grab the existing value here if it already exists
        self.node_map.entry(vessel_id.clone()).or_insert(
            self.graph.add_node(BloodNode::new(vessel_id, vessel_type))
        ).clone()
    }


    /// Sets edge weights for which incoming and outgoing are set evenly based on the number
    /// of inputs/outputs connected to the Node
    /// TODO: May want to allow this to be configurable in the future
    fn set_edge_weights(&mut self) {
        for idx in self.graph.node_indices() {
            let incoming_edges: Vec<EdgeIndex> = self.graph.edges_directed(idx, Direction::Incoming).map(|x| x.id()).collect();
            let incoming_count = incoming_edges.len() as f32;
            for edge_id in incoming_edges {
                self.graph[edge_id].incoming_pct = 1.0 / incoming_count;
            }

            let outgoing_edges: Vec<EdgeIndex> = self.graph.edges_directed(idx, Direction::Outgoing).map(|x| x.id()).collect();
            let outgoing_count = outgoing_edges.len() as f32;
            for edge_id in outgoing_edges {
                self.graph[edge_id].outgoing_pct = 1.0 / outgoing_count;
            }
        }
    }

    /// Internal function for computing the map of vessel ids to `NodeIndex` items for
    /// rapid lookup later
    fn compute_node_map(&mut self) {
        for node_idx in self.graph.node_indices() {
            self.node_map.insert(self.graph[node_idx].vessel_id.clone(), node_idx);
        }
    }

    pub fn digraph(&self) -> String {
        Dot::with_config(&self.graph, &[Config::EdgeNoLabel]).to_string()
    }
    
    pub fn digraph_edges(&self) -> String {
        Dot::new(&self.graph).to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::time::{Duration, Instant};
    use petgraph::stable_graph::StableDiGraph;
    use petgraph::graphmap::DiGraphMap;
    use petgraph::dot::{Dot, Config};
    use serde_json::to_string_pretty;
    use super::CirculationDef;

    #[test]
    fn test_load() {
        let circ = CirculationDef::from_json_file("config/circulation/human_circulation.json").unwrap();
        println!("{}", circ.digraph());
        println!("depth: {}", circ.depth);
    }
}