mod connector;
mod initializer;
pub use connector::HumanSimConnector;
pub use initializer::HumanModuleInitializer;
use crate::sim::{component::SimComponent, layer::{core::{CoreComponentInitializer, CoreComponent}, closed_circulation::{ClosedCircConnector, ClosedCircInitializer, ClosedCircComponent}}};

use super::HumanBloodVessel;

pub type HumanCircConnector = ClosedCircConnector<HumanBloodVessel>;
pub type HumanCircInitializer = ClosedCircInitializer<HumanBloodVessel>;

pub trait HumanComponent: SimComponent + CoreComponent + ClosedCircComponent {}

// #[cfg(test)]
// mod tests {
//     use crate::sim::{layer::core::CoreConnector, component::SimComponent};
//     use crate::sim::component::registry::ComponentRegistry;

//     use super::HumanCircConnector;


//     pub struct TestModuleA {
//         core_connector: CoreConnector,
//         circ_connector: HumanCircConnector,
//     }
//     impl SimComponent for TestModuleA {
//         fn id(&self) -> &'static str {
//             "TestModuleA"
//         }
//         fn attach(self, registry: &mut ComponentRegistry) {
//             registry.
//         }
//         fn init(&mut self, initializer: &mut SimModuleInitializer) {
//             initializer.notify(TestEventA::new(Distance::from_m(1.0)));
//             initializer.notify(TestEventB::new(Amount::from_mol(1.0)));
//             initializer.transform(|evt: &mut TestEventA| {
//                 evt.len = Distance::from_m(3.0);
//             });
//         }
//         fn run(&mut self) {
//             let evt_a = self.connector.get::<TestEventA>().unwrap();
//             assert_eq!(evt_a.len, Distance::from_m(3.0));

//             log::debug!("Trigger Events: {:?}", self.connector.trigger_events().collect::<Vec<&Arc<dyn Event>>>());
//         }
//     }

//     impl HumanSimModule for TestModuleA {
//         fn as_sim_module(&mut self) -> &mut dyn SimModule {
//             self
//         }
//         fn init_human(&mut self, initializer: &mut super::HumanModuleInitializer) {

//         }

//         fn get_human_sim_connector(&mut self) -> &mut super::HumanSimConnector {
//             &mut self.human_connector
//         }
//     }

//     #[test]
//     fn test_human_sim_module() {
//         register_module("TestModuleA", TestModuleA::factory);

//         let module = TestModuleA::factory();
//         let module_ref: Box<&mut dyn SimModule> = Box::new(module.as_sim_module());

//         // assert_eq!(sim.get_time(), Time::from_s(0.0));
//         // sim.advance_by(Time::from_s(1.0));
//         // assert_eq!(sim.get_time(), Time::from_s(1.0));
//     }
// }
