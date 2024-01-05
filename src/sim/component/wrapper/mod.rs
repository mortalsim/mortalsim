pub mod core;
pub mod closed_circulation;
use crate::sim::{layer::{core::component::CoreComponent, closed_circulation::ClosedCircComponent}, organism::{Organism, generic::GenericSim}};

pub trait ComponentWrapper<O: Organism = GenericSim>: CoreComponent<O> + ClosedCircComponent<O> {
    fn is_core_component(&self) -> bool;
    fn is_closed_circ_component(&self) -> bool;
}
