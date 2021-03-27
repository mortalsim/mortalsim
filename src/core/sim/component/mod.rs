mod connector;
mod initializer;
use std::any::TypeId;
use std::collections::HashSet;
use crate::event::Event;
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
    /// * `initializer` - Helper object for initializing the component
    fn run(&mut self, connector: &mut SimConnector);
}

#[cfg(test)]
pub mod test {
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

            match connector.get_trigger_event() {
                None => {
                    log::debug!("No trigger event");
                },
                Some(evt) => {
                    log::debug!("Trigger event: {:?}", evt);
                }
            }
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

            match connector.get_trigger_event() {
                None => {
                    print!("No trigger event");
                },
                Some(evt) => {
                    print!("Trigger event: {:?}", evt);
                }
            }
        }
    }
}