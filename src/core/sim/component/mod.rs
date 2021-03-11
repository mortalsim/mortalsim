mod connector;
pub use connector::BioConnector;

pub trait BioComponent {
    fn init(&mut self, connector: &mut BioConnector);
    fn trigger(&mut self, connector: &mut BioConnector);
}


#[cfg(test)]
pub mod test {
    use super::BioComponent;
    use super::BioConnector;

    pub struct TestComponent {}
    impl TestComponent {
        pub fn factory() -> Box<dyn BioComponent> {
            Box::new(TestComponent {})
        }
    }
    impl BioComponent for TestComponent {
        fn init(&mut self, _connector: &mut BioConnector) {}
        fn trigger(&mut self, _connector: &mut BioConnector) {}
    }
}