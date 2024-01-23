pub mod empty_wrapper;
pub mod core;
pub mod closed_circulation;
pub mod digestion;
pub mod nervous;

use crate::sim::layer::{DigestionComponent, NervousComponent};
use crate::sim::layer::core::component::CoreComponent;
use crate::sim::layer::closed_circulation::ClosedCircComponent;
use crate::sim::organism::Organism;

use super::registry::ComponentRegistry;

pub trait ComponentWrapper<O: Organism>: CoreComponent<O> + ClosedCircComponent<O> + DigestionComponent<O> + NervousComponent<O> {
    fn attach(self, registry: &mut ComponentRegistry<O>);
}
