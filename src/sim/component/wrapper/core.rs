use super::{
    super::{ComponentRegistry, SimComponent},
    ComponentWrapper,
};
use crate::sim::layer::core::component::{CoreComponent, CoreComponentInitializer, CoreConnector};

pub struct CoreComponentWrapper<T: CoreComponent + 'static>(pub T);

impl<T: CoreComponent> ComponentWrapper for CoreComponentWrapper<T> {
    fn is_core_component(&self) -> bool {
        true
    }
    fn is_closed_circ_component(&self) -> bool {
        false
    }
}

impl<T: CoreComponent> SimComponent for CoreComponentWrapper<T> {
    fn id(&self) -> &'static str {
        self.0.id()
    }
    fn attach(self, registry: &mut ComponentRegistry) {
        registry.add_core_component(self.0);
    }
    fn run(&mut self) {
        self.0.run();
    }
}

impl<T: CoreComponent> CoreComponent for CoreComponentWrapper<T> {
    fn core_init(&mut self, initializer: &mut CoreComponentInitializer) {
        self.0.core_init(initializer)
    }
    fn core_connector(&mut self) -> &mut CoreConnector {
        self.0.core_connector()
    }
}
