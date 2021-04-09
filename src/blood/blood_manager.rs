
use std::collections::HashMap;
use serde_json::Value;
use indextree::{Arena, NodeId};
use super::BloodVesselType;
use crate::substance::SubstanceStore;

struct BloodNode<'a> {
    pub vessel_id: String,
    pub vessel_type: BloodVesselType,
    pub substance_stores: Vec<SubstanceStore<'a>>,
}

pub struct BloodManager<'a> {
    arena: Arena<BloodNode<'a>>,
    node_map: HashMap<String, NodeId>,
}

impl<'a> BloodManager<'a> {
    pub fn new(vessel_tree_def: &Value) -> BloodManager<'a> {
        let mut manager = BloodManager {
            arena: Arena::new(),
            node_map: HashMap::new(),
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
}