mod connector;
mod initializer;
use std::any::TypeId;
use std::collections::HashSet;
use crate::event::Event;
pub use connector::{InitBioConnector, BioConnector};
pub use initializer::BioComponentInitializer;

pub trait BioComponent {
    fn init(&mut self, initializer: &mut BioComponentInitializer);
    fn run(&mut self, connector: &mut BioConnector);
}

#[cfg(test)]
pub mod test {
    use crate::event::Event;
    use super::BioComponent;
    use super::{BioComponentInitializer, BioConnector};

    pub struct TestComponent {}
    impl TestComponent {
        pub fn factory() -> Box<dyn BioComponent> {
            Box::new(TestComponent {})
        }
    }
    impl BioComponent for TestComponent {
        fn init(&mut self, _initializer: &mut BioComponentInitializer) {}
        fn run(&mut self, _connector: &mut BioConnector) {}
    }
}