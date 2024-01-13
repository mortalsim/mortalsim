use crate::sim::component::SimComponent;
use crate::sim::organism::Organism;

pub mod connector;
pub mod initializer;

use self::connector::DigestionConnector;
pub use self::initializer::DigestionInitializer;

pub trait DigestionComponent<O: Organism>: SimComponent<O> {
    /// Initializes the module. Currently not used.
    ///
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the module
    fn digestion_init(&mut self, _initializer: &mut DigestionInitializer) {}

    /// Used by the Sim to retrieve a mutable reference to this module's
    /// DigestionConnector, which tracks module interactions
    ///
    /// ### returns
    /// DigestionConnector to interact with the digestion layer
    fn digestion_connector(&mut self) -> &mut DigestionConnector;
}

// #[cfg(test)]
// pub mod test {
//     use super::{CoreComponent, SimComponent};
//     use super::{CoreComponentInitializer, CoreConnector};
//     use crate::event::test::{TestEventA, TestEventB};
//     use crate::event::Event;
//     use crate::sim::component::registry::ComponentRegistry;
//     use crate::sim::SimState;
//     use crate::units::base::Amount;
//     use crate::units::base::Distance;
//     use std::any::TypeId;
//     use std::sync::{Arc, Mutex};

//     pub struct TestComponentA {
//         connector: CoreConnector,
//     }
//     impl TestComponentA {
//         fn new() -> TestComponentA {
//             TestComponentA {
//                 connector: CoreConnector::new(),
//             }
//         }
//     }
//     impl CoreComponent for TestComponentA {
//         fn core_connector(&mut self) -> &mut CoreConnector {
//             &mut self.connector
//         }
//         fn core_init(&mut self, initializer: &mut CoreComponentInitializer) {
//             initializer.notify(TestEventA::new(Distance::from_m(1.0)));
//             initializer.notify(TestEventB::new(Amount::from_mol(1.0)));
//             initializer.transform(|evt: &mut TestEventA| {
//                 evt.len = Distance::from_m(3.0);
//             });
//         }
//     }

//     impl SimComponent for TestComponentA {
//         fn id(&self) -> &'static str {
//             "TestComponentA"
//         }
//         fn attach(self, registry: &mut ComponentRegistry) {
//             registry.add_core_component(self)
//         }
//         fn run(&mut self) {
//             let evt_a = self.connector.get::<TestEventA>().unwrap();
//             assert_eq!(evt_a.len, Distance::from_m(3.0));

//             log::debug!(
//                 "Trigger Events: {:?}",
//                 self.connector
//                     .trigger_events()
//                     .collect::<Vec<&TypeId>>()
//             );
//         }
//     }

//     pub struct TestComponentB {
//         connector: CoreConnector,
//     }
//     impl TestComponentB {
//         pub fn new() -> TestComponentB {
//             TestComponentB {
//                 connector: CoreConnector::new(),
//             }
//         }

//         pub fn transform_b(evt: &mut TestEventB) {
//             evt.amt = evt.amt + Amount::from_mol(0.0);
//         }
//     }
//     impl CoreComponent for TestComponentB {
//         fn core_init(&mut self, initializer: &mut CoreComponentInitializer) {
//             initializer.notify(TestEventA::new(Distance::from_m(2.0)));
//             initializer.notify(TestEventB::new(Amount::from_mol(2.0)));
//             initializer.transform(Self::transform_b);
//         }
//         fn core_connector(&mut self) -> &mut CoreConnector {
//             &mut self.connector
//         }
//     }

//     impl SimComponent for TestComponentB {
//         fn id(&self) -> &'static str {
//             "TestComponentB"
//         }
//         fn attach(self, registry: &mut ComponentRegistry) {
//             registry.add_core_component(self)
//         }
//         fn run(&mut self) {
//             let evt_a = self.connector.get::<TestEventA>().unwrap();
//             assert_eq!(evt_a.len, Distance::from_m(3.0));

//             log::debug!(
//                 "Trigger Events: {:?}",
//                 self.connector
//                     .trigger_events()
//                     .collect::<Vec<&TypeId>>()
//             );
//         }
//     }

//     #[test]
//     fn test_component() {
//         let mut component = TestComponentA::new();
//         let mut initializer = CoreComponentInitializer::new();
//         component.core_init(&mut initializer);

//         assert!(initializer.pending_notifies.len() == 2);
//         assert!(initializer.pending_transforms.len() == 1);
//     }
// }
