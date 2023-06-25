mod connector;
mod initializer;
use std::any::TypeId;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::event::Event;
use crate::sim::component::SimComponent;
use crate::sim::component::wrapper::ComponentWrapper;
pub use connector::CoreConnector;
pub use initializer::CoreComponentInitializer;

pub trait CoreComponent : SimComponent {
  /// Initializes the module. Should register any `Event` objects to listen for
  /// and set initial state.
  /// 
  /// ### Arguments
  /// * `initializer` - Helper object for initializing the module
  fn core_init(&mut self, initializer: &mut CoreComponentInitializer);

  /// Used by the Sim to retrieve a mutable reference to this module's
  /// CoreConnector, which tracks module interactions
  /// 
  /// ### returns
  /// CoreConnector to interact with the rest of the simulation
  fn core_connector(&mut self) -> &mut CoreConnector;
}

#[cfg(test)]
pub mod test {
    use std::sync::{Arc, Mutex};
    use crate::sim::SimState;
    use crate::event::Event;
    use crate::event::test::{TestEventA, TestEventB};
    use crate::sim::component::registry::ComponentRegistry;
    use super::{CoreComponent, SimComponent};
    use super::{CoreComponentInitializer, CoreConnector};
    use uom::si::f64::{Length, AmountOfSubstance};
    use uom::si::length::meter;
    use uom::si::amount_of_substance::mole;

    pub struct TestComponentA {
      connector: CoreConnector
    }
    impl TestComponentA {}
    impl CoreComponent for TestComponentA {
        fn core_connector(&mut self) -> &mut CoreConnector {
          &mut self.connector
        }
        fn core_init(&mut self, initializer: &mut CoreComponentInitializer) {
          initializer.notify(TestEventA::new(Length::new::<meter>(1.0)));
          initializer.notify(TestEventB::new(AmountOfSubstance::new::<mole>(1.0)));
          initializer.transform(|evt: &mut TestEventA| {
              evt.len = Length::new::<meter>(3.0);
          });
        }
    }

    impl SimComponent for TestComponentA {
      fn attach(self, registry: &mut ComponentRegistry) {
        registry.add_core_component("TestComponentA", self)
      }
      fn run(&mut self) {
        let evt_a = self.connector.get::<TestEventA>().unwrap();
        assert_eq!(evt_a.len, Length::new::<meter>(3.0));

        log::debug!("Trigger Events: {:?}", self.connector.trigger_events().collect::<Vec<&Arc<dyn Event>>>());
      }
    }
    
    pub struct TestComponentB {
      connector: CoreConnector
    }
    impl TestComponentB {
      pub fn factory() -> Box<dyn SimComponent> {
          Box::new(TestComponentA {
              connector: CoreConnector::new()
          })
      }

      pub fn transform_b(evt: &mut TestEventB) {
          evt.amt = evt.amt + AmountOfSubstance::new::<mole>(0.0);
      }
    }
    impl CoreComponent for TestComponentB {
      fn core_init(&mut self, initializer: &mut CoreComponentInitializer) {
          initializer.notify(TestEventA::new(Length::new::<meter>(2.0)));
          initializer.notify(TestEventB::new(AmountOfSubstance::new::<mole>(2.0)));
          initializer.transform(Self::transform_b);
      }
      fn core_connector(&mut self) -> &mut CoreConnector {
          &mut self.connector
      }
    }

    impl SimComponent for TestComponentB {
      fn attach(self, registry: &mut ComponentRegistry) {
        registry.add_core_component("TestComponentB", self)
      }
      fn run(&mut self) {
          let evt_a = self.connector.get::<TestEventA>().unwrap();
          assert_eq!(evt_a.len, Length::new::<meter>(3.0));

          log::debug!("Trigger Events: {:?}", self.connector.trigger_events().collect::<Vec<&Arc<dyn Event>>>());
      }
    }
}
