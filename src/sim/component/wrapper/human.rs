use crate::sim::{organism::human::{component::{HumanComponent, HumanCircInitializer, HumanCircConnector}, HumanBloodVessel}, layer::{core::{CoreComponent, CoreComponentInitializer, CoreConnector}, closed_circulation::ClosedCircComponent}};

use super::{
    super::{ComponentRegistry, SimComponent},
    ComponentWrapper,
};

pub struct HumanComponentWrapper<T: HumanComponent + 'static>(pub T);

impl<T: HumanComponent> ComponentWrapper for HumanComponentWrapper<T> {
    fn is_core_component(&self) -> bool {
        true
    }
    fn is_closed_circ_component(&self) -> bool {
        true
    }
}

impl<T: HumanComponent> SimComponent for HumanComponentWrapper<T> {
    fn id(&self) -> &'static str {
        self.0.id()
    }
    fn attach(self, registry: &mut ComponentRegistry) {
        registry.add_human_component(self.0);
    }
    fn run(&mut self) {
        self.0.run();
    }
}

impl<T: HumanComponent> CoreComponent for HumanComponentWrapper<T> {
    fn core_init(&mut self, initializer: &mut CoreComponentInitializer) {
        self.0.core_init(initializer)
    }
    fn core_connector(&mut self) -> &mut CoreConnector {
        self.0.core_connector()
    }
}

impl<T: HumanComponent<VesselType = HumanBloodVessel>> ClosedCircComponent for HumanComponentWrapper<T> {
    type VesselType = T::VesselType;

    fn cc_init(&mut self, initializer: &mut HumanCircInitializer) {
        self.0.cc_init(initializer)
    }
    fn cc_connector(&mut self) -> &mut HumanCircConnector {
        self.0.cc_connector()
    }
}
