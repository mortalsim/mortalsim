mod connector;
mod initializer;
use std::any::TypeId;
use std::collections::HashSet;
use crate::event::Event;
pub use connector::BioConnector;
pub use initializer::BioComponentInitializer;

pub trait BioComponent {
    fn init(&mut self, initializer: &mut BioComponentInitializer);
    fn run(&mut self, connector: &mut BioConnector);
}

#[cfg(test)]
pub mod test {
    use crate::event::Event;
    use crate::event::test::{TestEventA, TestEventB};
    use super::BioComponent;
    use super::{BioComponentInitializer, BioConnector};
    use uom::si::f64::{Length, AmountOfSubstance};
    use uom::si::length::meter;
    use uom::si::amount_of_substance::mole;

    pub struct TestComponent {}
    impl TestComponent {
        pub fn factory() -> Box<dyn BioComponent> {
            Box::new(TestComponent {})
        }
    }
    impl BioComponent for TestComponent {
        fn init(&mut self, initializer: &mut BioComponentInitializer) {
            initializer.notify(TestEventA::new(Length::new::<meter>(1.0)));
            initializer.notify(TestEventB::new(AmountOfSubstance::new::<mole>(1.0)));
            initializer.transform(|evt: &mut TestEventA| {
                evt.len = Length::new::<meter>(3.0);
            });
        }
        fn run(&mut self, connector: &mut BioConnector) {
            let evt_a = connector.get::<TestEventA>().unwrap();
            assert_eq!(evt_a.len, Length::new::<meter>(3.0));
        }
    }
}