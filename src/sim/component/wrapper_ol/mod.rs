pub mod empty_wrapper;
pub mod core;
pub mod circulation;
pub mod digestion;
pub mod nervous;

use crate::sim::layer::{DigestionComponent, NervousComponent};
use crate::sim::layer::core::component::CoreComponent;
use crate::sim::layer::circulation::CirculationComponent;
use crate::sim::organism::Organism;

use super::SimComponent;

pub trait ComponentWrapper<O: Organism>: SimComponent<O> + CoreComponent<O> + CirculationComponent<O> + DigestionComponent<O> + NervousComponent<O> {}
