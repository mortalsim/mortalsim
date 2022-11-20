mod connector;
mod initializer;
use std::any::TypeId;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::event::Event;
use super::Sim;
pub use connector::SimConnector;
pub use initializer::SimModuleInitializer;

/// Trait to be used by any modules for Sim objects
pub trait SimModule {
    /// Initializes the module. Should register any `Event` objects to listen for
    /// and set initial state.
    /// 
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the module
    fn init(&mut self, initializer: &mut SimModuleInitializer);

    /// Used by the Sim to retrieve a mutable reference to this module's
    /// SimConnector, which tracks module interactions
    /// 
    /// ### returns
    /// SimConnector to interact with the rest of the simulation
    fn get_sim_connector(&mut self) -> &mut SimConnector;
    
    /// Runs an iteration of this module. Will be called anytime a `notify` registered
    /// `Event` changes on `Sim` state. All module logic should ideally occur within this
    /// call and all resulting `Event` objects scheduled for future emission.
    /// 
    /// Note that all `Event`s previously scheduled by this module which have not yet
    /// occurred will be unscheduled before `run` is executed.
    /// 
    /// ### Arguments
    /// * `connector` - Helper object for the module to interact with the rest of
    ///                 the simulation
    fn run(&mut self);
}

#[cfg(test)]
pub mod test {
    use std::sync::Arc;
    use crate::event::Event;
    use crate::event::test::{TestEventA, TestEventB};
    use super::SimModule;
    use super::{SimModuleInitializer, SimConnector};
    use uom::si::f64::{Length, AmountOfSubstance};
    use uom::si::length::meter;
    use uom::si::amount_of_substance::mole;

    pub struct TestModuleA {
        connector: SimConnector
    }
    impl TestModuleA {
        pub fn factory() -> Box<dyn SimModule> {
            Box::new(TestModuleA {
                connector: SimConnector::new()
            })
        }
    }
    impl SimModule for TestModuleA {
        fn get_sim_connector(&mut self) -> &mut SimConnector {
            &mut self.connector
        }
        fn init(&mut self, initializer: &mut SimModuleInitializer) {
            initializer.notify(TestEventA::new(Length::new::<meter>(1.0)));
            initializer.notify(TestEventB::new(AmountOfSubstance::new::<mole>(1.0)));
            initializer.transform(|evt: &mut TestEventA| {
                evt.len = Length::new::<meter>(3.0);
            });
        }
        fn run(&mut self) {
            let evt_a = self.connector.get::<TestEventA>().unwrap();
            assert_eq!(evt_a.len, Length::new::<meter>(3.0));

            log::debug!("Trigger Events: {:?}", self.connector.trigger_events().collect::<Vec<&Arc<dyn Event>>>());
        }
    }
    
    pub struct TestModuleB {
        connector: SimConnector
    }
    impl TestModuleB {
        pub fn factory() -> Box<dyn SimModule> {
            Box::new(TestModuleA {
                connector: SimConnector::new()
            })
        }
    }
    impl SimModule for TestModuleB {
        fn get_sim_connector(&mut self) -> &mut SimConnector {
            &mut self.connector
        }
        fn init(&mut self, initializer: &mut SimModuleInitializer) {
            initializer.notify(TestEventA::new(Length::new::<meter>(2.0)));
            initializer.notify(TestEventB::new(AmountOfSubstance::new::<mole>(2.0)));
        }
        fn run(&mut self) {
            let evt_a = self.connector.get::<TestEventA>().unwrap();
            assert_eq!(evt_a.len, Length::new::<meter>(3.0));

            log::debug!("Trigger Events: {:?}", self.connector.trigger_events().collect::<Vec<&Arc<dyn Event>>>());
        }
    }
}
