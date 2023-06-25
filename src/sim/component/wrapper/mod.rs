pub mod core;
use crate::sim::system::core::component::CoreComponent;

pub trait ComponentWrapper : CoreComponent {
  fn is_core_component(&self) -> bool;
  fn is_closed_circ_component(&self) -> bool;
}