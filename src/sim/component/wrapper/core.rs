use super::{super::{SimComponent, ComponentRegistry}, ComponentWrapper};
use crate::sim::system::core::component::{CoreComponent, CoreComponentInitializer, CoreConnector};

pub struct CoreComponentWrapper<T: CoreComponent + 'static> {
  name: &'static str,
  inner: T,
}

impl<T: CoreComponent> CoreComponentWrapper<T> {
  pub fn new(name: &'static str, inner: T) -> CoreComponentWrapper<T> {
    CoreComponentWrapper {name, inner}
  }
}

impl<T: CoreComponent> ComponentWrapper for CoreComponentWrapper<T> {
  fn is_core_component(&self) -> bool {true}
  fn is_closed_circ_component(&self) -> bool {false}
}

impl<T: CoreComponent> SimComponent for CoreComponentWrapper<T> {
  fn attach(self, registry: &mut ComponentRegistry) {
    registry.add_core_component(self.name, self.inner);
  }
  fn run(&mut self) {
      self.inner.run();
  }
}

impl<T: CoreComponent> CoreComponent for CoreComponentWrapper<T> {
  fn core_init(&mut self, initializer: &mut CoreComponentInitializer) {
    self.inner.core_init(initializer)
  }
  fn core_connector(&mut self) -> &mut CoreConnector {
    self.inner.core_connector()
  }
}