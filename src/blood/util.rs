use std::fmt;
use std::fs;
use std::rc::Rc;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use petgraph::Direction;
use petgraph::visit::EdgeRef;
use petgraph::graph::{Graph, NodeIndex, EdgeIndex};
use super::{BloodNode, BloodEdge, BloodVesselType};
use super::BloodVesselType::{Vein, Artery};

/// Loads a circulation graph and corresponding vessel->idx map from the given file
pub fn load_circulation(filename: &str) -> Result<(Graph<BloodNode, BloodEdge>, HashMap<Rc<String>, NodeIndex>)> {
    let contents = fs::read_to_string(filename).unwrap();
    parse_circulation(&contents)
}

/// Parses a circulation graph and corresponding vessel->idx map from the given json string
pub fn parse_circulation(json: &str) -> Result<(Graph<BloodNode, BloodEdge>, HashMap<Rc<String>, NodeIndex>)> {
    let mut graph = Graph::new();
    let mut node_map = HashMap::new();

    // Get each tree from the circulation definition
    let circulation_def: Value = serde_json::from_str(json)?;

    let arterial_tree = &circulation_def["arterial"];
    let venous_tree = &circulation_def["venous"];

    // Add veins first, since arteries reference them at the capillaries
    add_veins(&mut graph, &mut node_map, venous_tree);
    add_arteries(&mut graph, &mut node_map, arterial_tree);
    set_edge_weights(&mut graph);

    Ok((graph, node_map))
}

/// Adds the venous tree to the circulation graph
fn add_veins(graph: &mut Graph<BloodNode, BloodEdge>, node_map: &mut HashMap<Rc<String>, NodeIndex>, vein: &Value) {
    let root_idx = add_node(graph, node_map, vein, Vein);
    add_vessels(graph, node_map, vein, root_idx, Vein);
}

/// Adds the arterial tree to the circulation graph
fn add_arteries(graph: &mut Graph<BloodNode, BloodEdge>, node_map: &mut HashMap<Rc<String>, NodeIndex>, artery: &Value) {
    let root_idx = add_node(graph, node_map, artery, Artery);
    add_vessels(graph, node_map, artery, root_idx, Artery);
}

/// Adds a single node to the circulation graph
fn add_node(graph: &mut Graph<BloodNode, BloodEdge>, node_map: &mut HashMap<Rc<String>, NodeIndex>, vessel: &Value, vessel_type: BloodVesselType) -> NodeIndex {
    let vessel_id = Rc::new(String::from(vessel["id"].as_str().unwrap()));

    // Nodes can be defined multiple times if needed to establish multiple upstream connections
    // so we'll grab the existing value here if it already exists
    node_map.entry(vessel_id.clone()).or_insert(graph.add_node(BloodNode {
        vessel_id: vessel_id,
        vessel_type: vessel_type,
    })).clone()
}

/// Recursive function which adds BloodNodes to the circulation graph based on the JSON definition
fn add_vessels(graph: &mut Graph<BloodNode, BloodEdge>, node_map: &mut HashMap<Rc<String>, NodeIndex>, vessel: &Value, vessel_idx: NodeIndex, vessel_type: BloodVesselType) {
    let links = &vessel["links"];

    // If there are more nodes in the tree, keep adding them recursively
    if links.is_array() {
        for link_vessel in links.as_array().unwrap() {
            let link_idx = add_node(graph, node_map, link_vessel, vessel_type);
            add_vessels(graph, node_map, link_vessel, link_idx, vessel_type);

            if vessel_type == Artery {
                graph.add_edge(vessel_idx, link_idx, BloodEdge::new());
            }
            else {
                graph.add_edge(link_idx, vessel_idx, BloodEdge::new());
            }
        }
    }

    let bridge = &vessel["bridge"];

    if bridge.is_array() {
        // Establish an edge between the current Node and the venous Node, which
        // we'll look up from the map
        for bridge_id in bridge.as_array().unwrap() {
            let bridge_str = Rc::new(String::from(bridge_id.as_str().unwrap()));
            match node_map.get(&bridge_str) {
                Some(bridge_idx) => {
                    graph.add_edge(vessel_idx, bridge_idx.clone(), BloodEdge::new());
                },
                None => {
                    panic!("Invalid bridge value for Node '{}': '{}'", &graph[vessel_idx], bridge_str)
                }
            }
        }
    }
}

fn set_edge_weights(graph: &mut Graph<BloodNode, BloodEdge>) {
    for idx in graph.node_indices() {
        let incoming_edges: Vec<EdgeIndex> = graph.edges_directed(idx, Direction::Incoming).map(|x| x.id()).collect();
        let incoming_count = incoming_edges.len() as f32;
        for edge_id in incoming_edges {
            graph[edge_id].incoming_pct = 1.0 / incoming_count;
        }

        let outgoing_edges: Vec<EdgeIndex> = graph.edges_directed(idx, Direction::Outgoing).map(|x| x.id()).collect();
        let outgoing_count = outgoing_edges.len() as f32;
        for edge_id in outgoing_edges {
            graph[edge_id].outgoing_pct = 1.0 / outgoing_count;
        }
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

    #[test]
    fn test_load() {
        let (graph, _) = super::load_circulation("config/circulation/human_circulation.json").unwrap();
        println!("{}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
        // println!("{}", Dot::new(&graph));
    }
}