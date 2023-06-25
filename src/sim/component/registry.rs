
use std::collections::HashMap;
use crate::sim::layer::core::component::CoreComponent;

use super::{wrapper::{ComponentWrapper, core::CoreComponentWrapper}, SimComponent};

pub struct ComponentRegistry<'a>(HashMap<&'a str, Box<dyn ComponentWrapper>>);

impl<'a> ComponentRegistry<'a> {
  pub fn add_core_component(&mut self, component: impl CoreComponent + 'static) {
    self.0.insert(component.id(), Box::new(CoreComponentWrapper(component)));
  }
  // TODO: create add_{system}Component methods for each combination of systems
}
