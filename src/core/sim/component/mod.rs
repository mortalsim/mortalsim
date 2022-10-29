mod connector;
mod initializer;
use std::any::TypeId;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::event::Event;
use super::Sim;
pub use connector::SimConnector;
pub use initializer::SimComponentInitializer;

/// Trait to be used by any components for Sim objects
pub trait SimComponent {
    /// Initializes the component. Should register any `Event` objects to listen for
    /// and set initial state.
    /// 
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the component
    fn init(&mut self, initializer: &mut SimComponentInitializer);
    
    /// Runs an iteration of this component. Will be called anytime a `notify` registered
    /// `Event` changes on `Sim` state. All module logic should ideally occur within this
    /// call and all resulting `Event` objects scheduled for future emission.
    /// 
    /// Note that all `Event`s previously scheduled by this component which have not yet
    /// occurred will be unscheduled before `run` is executed.
    /// 
    /// ### Arguments
    /// * `connector` - Helper object for the component to interact with the rest of
    ///                 the simulation
    fn run(&mut self, connector: &mut SimConnector);
}

#[cfg(test)]
pub mod test {
    use std::sync::Arc;
    use crate::event::Event;
    use crate::event::test::{TestEventA, TestEventB};
    use super::SimComponent;
    use super::{SimComponentInitializer, SimConnector};
    use uom::si::f64::{Length, AmountOfSubstance};
    use uom::si::length::meter;
    use uom::si::amount_of_substance::mole;

    pub struct TestComponentA {}
    impl TestComponentA {
        pub fn factory() -> Box<dyn SimComponent> {
            Box::new(TestComponentA {})
        }
    }
    impl SimComponent for TestComponentA {
        fn init(&mut self, initializer: &mut SimComponentInitializer) {
            initializer.notify(TestEventA::new(Length::new::<meter>(1.0)));
            initializer.notify(TestEventB::new(AmountOfSubstance::new::<mole>(1.0)));
            initializer.transform(|evt: &mut TestEventA| {
                evt.len = Length::new::<meter>(3.0);
            });
        }
        fn run(&mut self, connector: &mut SimConnector) {
            let evt_a = connector.get::<TestEventA>().unwrap();
            assert_eq!(evt_a.len, Length::new::<meter>(3.0));

            log::debug!("Trigger Events: {:?}", connector.trigger_events().collect::<Vec<&Arc<dyn Event>>>());
        }
    }
    
    pub struct TestComponentB {}
    impl TestComponentB {
        pub fn factory() -> Box<dyn SimComponent> {
            Box::new(TestComponentB {})
        }
    }
    impl SimComponent for TestComponentB {
        fn init(&mut self, initializer: &mut SimComponentInitializer) {
            initializer.notify(TestEventA::new(Length::new::<meter>(2.0)));
            initializer.notify(TestEventB::new(AmountOfSubstance::new::<mole>(2.0)));
        }
        fn run(&mut self, connector: &mut SimConnector) {
            let evt_a = connector.get::<TestEventA>().unwrap();
            assert_eq!(evt_a.len, Length::new::<meter>(3.0));

            log::debug!("Trigger Events: {:?}", connector.trigger_events().collect::<Vec<&Arc<dyn Event>>>());
        }
    }
}
