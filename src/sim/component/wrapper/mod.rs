pub mod core;
pub mod closed_circulation;
use std::collections::HashSet;

use crate::sim::layer::SimLayer;
use crate::sim::layer::core::component::CoreComponent;
use crate::sim::layer::closed_circulation::ClosedCircComponent;
use crate::sim::organism::Organism;

use super::registry::ComponentRegistry;

pub trait ComponentWrapper<O: Organism>: CoreComponent<O> + ClosedCircComponent<O> {
    fn attach(self, registry: &mut ComponentRegistry<O>);
}
