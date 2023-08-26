use super::{
    super::{ComponentRegistry, SimComponent},
    ComponentWrapper,
};
use crate::sim::layer::core::component::{CoreComponent, CoreComponentInitializer, CoreConnector};
use crate::sim::layer::closed_circulation::{component::{ClosedCircComponent, ClosedCircInitializer, ClosedCircConnector}, BloodVessel};

pub struct ClosedCircComponentWrapper<T: ClosedCircComponent + 'static>(pub T);

impl<T: ClosedCircComponent> ComponentWrapper for ClosedCircComponentWrapper<T> {
    fn is_core_component(&self) -> bool {
        true
    }
    fn is_closed_circ_component(&self) -> bool {
        false
    }
}

impl<T: ClosedCircComponent> SimComponent for ClosedCircComponentWrapper<T> {
    fn id(&self) -> &'static str {
        self.0.id()
    }
    fn attach(self, registry: &mut ComponentRegistry) {
        registry.add_closed_circulation_component(self.0);
    }
    fn run(&mut self) {
        self.0.run();
    }
}

impl<T: ClosedCircComponent> CoreComponent for ClosedCircComponentWrapper<T> {
    fn core_init(&mut self, _initializer: &mut CoreComponentInitializer) {
        panic!();
    }
    fn core_connector(&mut self) -> &mut CoreConnector {
        panic!();
    }
}

impl<T: ClosedCircComponent> ClosedCircComponent for ClosedCircComponentWrapper<T> {
    type VesselType = T::VesselType;

    fn cc_init(&mut self, initializer: &mut ClosedCircInitializer<Self::VesselType>) {
        self.0.cc_init(initializer)
    }
    fn cc_connector(&mut self) -> &mut ClosedCircConnector<Self::VesselType> {
        self.0.cc_connector()
    }
}
