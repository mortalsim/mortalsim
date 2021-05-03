
use petgraph::graph::{Graph, Neighbors};
use super::{BloodVessel};

mod organism;
mod system;
mod component;
mod graph;
mod manager;

pub use system::ClosedCirculatorySystem;
pub use component::{ClosedCircSimComponent, ClosedCircComponentInitializer, ClosedCircInitializer, ClosedCircConnector, ClosedCircSimConnector};
pub use graph::{BloodEdge, BloodNode};
pub use manager::ClosedCirculationManager;
pub use organism::{ClosedCirculationOrganism, ClosedCirculationSimOrganism};

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
