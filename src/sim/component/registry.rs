
use std::collections::HashMap;
use crate::sim::system::core::component::CoreComponent;

use super::{wrapper::{ComponentWrapper, core::CoreComponentWrapper}, SimComponent};

pub struct ComponentRegistry<'a>(HashMap<&'a str, Box<dyn ComponentWrapper>>);

impl<'a> ComponentRegistry<'a> {
  pub fn add_core_component(&mut self, component_name: &'static str, component: impl CoreComponent + 'static) {
    self.0.insert(component_name, Box::new(CoreComponentWrapper::new(component_name, component)));
  }
  // TODO: create add_{system}Component methods for each combination of systems
}
