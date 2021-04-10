use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use std::collections::HashMap;
use serde_json::Value;
use petgraph::stable_graph::StableDiGraph;
use super::BloodVesselType;
use crate::substance::SubstanceStore;


#[derive(Debug)]
struct BloodNode<'a> {
    pub vessel_id: String,
    pub vessel_type: BloodVesselType,
    pub main_store: SubstanceStore<'a>,
    pub stores: Vec<SubstanceStore<'a>>,
}

impl<'a> BloodNode<'a> {
    fn new(vessel_id: String, vessel_type: BloodVesselType) -> BloodNode<'a> {
        BloodNode {
            vessel_id: vessel_id,
            vessel_type: vessel_type,
            main_store: SubstanceStore::new(),
            stores: Vec::new(),
        }
    }
}

impl<'a> Hash for BloodNode<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vessel_id.hash(state);
    }
}

// impl<'a> PartialEq for BloodNode<'a> {
//     fn eq(&self, other: &BloodNode) -> bool {
//         self.vessel_id.eq(&other.vessel_id)
//     }
// }

// impl<'a> PartialOrd for BloodNode<'a> {
//     fn partial_cmp(&self, other: &BloodNode) -> Option<Ordering> {
//         self.vessel_id.partial_cmp(&other.vessel_id)
//     }
// }

// impl<'a> Eq for BloodNode<'a> {}

// impl<'a> Ord for BloodNode<'a> {
//     fn cmp(&self, other: &BloodNode) -> Ordering {
//         self.vessel_id.cmp(&other.vessel_id)
//     }
// }

pub struct BloodManager<'a> {
    graph: DiGraphMap<BloodNode<'a>, u32>,
}

impl<'a> BloodManager<'a> {
    pub fn new(vessel_tree_def: &Value) -> BloodManager<'a> {
        let mut manager = BloodManager {
            graph: DiGraphMap::new(),
        };

        manager.setup(vessel_tree_def);
        manager
    }

    fn setup(&mut self, vessel_tree_def: &Value) {
        let arterial_tree = &vessel_tree_def["arterial"];
        let venous_tree = &vessel_tree_def["venous"];

        let vein_id = String::from(venous_tree[0]["id"].as_str().unwrap());
        let vein_node = self.arena.new_node(BloodNode {
            vessel_id: vein_id.clone(),
            vessel_type: BloodVesselType::Vein,
            substance_stores: Vec::new(),
        });

        self.node_map.insert(vein_id, vein_node);

        self.add_arteries(arterial_tree, &vein_node);
    }

    fn add_node(&mut self, vessel_def: &Value, upstream: &NodeId) -> NodeId {
        let vessel_id = String::from(vessel_def["id"].as_str().unwrap());

        let node = self.arena.new_node(BloodNode {
            vessel_id: vessel_id.clone(),
            vessel_type: BloodVesselType::Artery,
            substance_stores: Vec::new(),
        });

        upstream.append(node, &mut self.arena);
        self.node_map.insert(vessel_id, node);
        node
    }

    fn add_arteries(&mut self, vessel_def: &Value, upstream: &NodeId) {
        let node = self.add_node(vessel_def, upstream);

        let links = &vessel_def["links"];

        // If there are more nodes in the tree, keep adding them recursively
        if links.is_array() {
            self.add_arteries(links, &node);
        }

        let bridge = &vessel_def["bridge"];

        if bridge.is_array() {
            // create a node for each vein
            for vein_def in bridge.as_array().unwrap() {
                self.add_node(vein_def, &node);
            }
        }
    }

    fn add_veins(&mut self, vessel_def: &Value) {

    }
}